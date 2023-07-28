// Built-in deps
use std::{fmt, time::Duration, time::Instant};
// External imports
use async_trait::async_trait;
use deadpool::managed::{Manager, PoolConfig, RecycleResult, Timeouts};
use deadpool::Runtime;
use sqlx::{Connection, Error as SqlxError, PgConnection};
use tokio::time;
// Local imports
// use self::recoverable_connection::RecoverableConnection;
use crate::{get_database_replica_url, get_database_url, StorageProcessor};
use zksync_utils::parse_env;

pub mod holder;

type Pool = deadpool::managed::Pool<DbPool>;

pub type PooledConnection = deadpool::managed::Object<DbPool>;

pub const DB_CONNECTION_RETRIES: u32 = 3;

#[derive(Clone)]
pub struct DbPool {
    url: String,
}

impl DbPool {
    fn create(url: impl Into<String>, max_size: usize) -> Pool {
        let pool_config = PoolConfig {
            max_size,
            timeouts: Timeouts::wait_millis(20_000), // wait 20 seconds before returning error
            runtime: Runtime::Tokio1,
        };
        Pool::from_config(DbPool { url: url.into() }, pool_config)
    }
}

#[async_trait]
impl Manager for DbPool {
    type Type = PgConnection;
    type Error = SqlxError;
    async fn create(&self) -> Result<PgConnection, SqlxError> {
        PgConnection::connect(&self.url).await
    }
    async fn recycle(&self, obj: &mut PgConnection) -> RecycleResult<SqlxError> {
        Ok(obj.ping().await?)
    }
}

/// `ConnectionPool` is a wrapper over a `diesel`s `Pool`, encapsulating
/// the fixed size pool of connection to the database.
///
/// The size of the pool and the database URL are configured via environment
/// variables `DATABASE_POOL_SIZE` and `DATABASE_URL` respectively.
#[derive(Clone)]
pub struct ConnectionPool {
    pool: Pool,
}

impl fmt::Debug for ConnectionPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Recoverable connection")
    }
}

impl ConnectionPool {
    /// Establishes a pool of the connections to the database and
    /// creates a new `ConnectionPool` object.
    /// pool_max_size - number of connections in pool, if not set env variable "DATABASE_POOL_SIZE" is going to be used.
    pub fn new(pool_max_size: Option<u32>) -> Self {
        let database_url = get_database_url();
        let max_size = pool_max_size.unwrap_or_else(|| parse_env("DATABASE_POOL_SIZE"));

        let pool = DbPool::create(database_url, max_size as usize);

        Self { pool }
    }

    /// Establishes a pool of the connections to the replica of database and
    /// creates a new `ConnectionPool` object.
    /// pool_max_size - number of connections in pool,
    /// if not set env variable "DATABASE_POOL_SIZE" is going to be used.
    pub fn new_readonly_pool(pool_max_size: Option<u32>) -> Self {
        let database_url = get_database_replica_url();
        let max_size = pool_max_size.unwrap_or_else(|| parse_env("DATABASE_POOL_SIZE"));

        let pool = DbPool::create(database_url, max_size as usize);

        Self { pool }
    }
    /// Creates a `StorageProcessor` entity over a recoverable connection.
    /// Upon a database outage connection will block the thread until
    /// it will be able to recover the connection (or, if connection cannot
    /// be restored after several retries, this will be considered as
    /// irrecoverable database error and result in panic).
    ///
    /// This method is intended to be used in crucial contexts, where the
    /// database access is must-have (e.g. block committer).
    pub async fn access_storage(&self) -> Result<StorageProcessor<'_>, SqlxError> {
        let start = Instant::now();
        let connection = self.get_pooled_connection().await;
        metrics::histogram!("sql.connection_acquire", start.elapsed());

        Ok(StorageProcessor::from_pool(connection))
    }

    async fn get_pooled_connection(&self) -> PooledConnection {
        let mut retry_count = 0;

        let mut one_second = time::interval(Duration::from_secs(1));

        while retry_count < DB_CONNECTION_RETRIES {
            let connection = self.pool.get().await;

            match connection {
                Ok(connection) => return connection,
                Err(_) => retry_count += 1,
            }

            // Backing off for one second if facing an error
            vlog::warn!("Failed to get connection to db. Backing off for 1 second");
            one_second.tick().await;
        }

        // Attempting to get the pooled connection for the last time
        self.pool.get().await.unwrap()
    }
}
