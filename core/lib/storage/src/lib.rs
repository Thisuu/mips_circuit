// `sqlx` macros result in these warning being triggered.
#![allow(clippy::toplevel_ref_arg, clippy::suspicious_else_formatting)]

// Built-in deps
use std::env;
use hex::{decode_to_slice, FromHexError};
// External imports
use sqlx::{postgres::Postgres, Connection, PgConnection, Transaction};
// Local imports
use crate::connection::{holder::ConnectionHolder, PooledConnection};
// Workspace imports
use types::{ActionType, BlockNumber};

pub mod chain;
pub mod connection;

pub use crate::connection::ConnectionPool;
pub use sqlx::types::BigDecimal;

pub type QueryResult<T, E = anyhow::Error> = Result<T, E>;

/// The maximum possible block number in the storage.
pub const MAX_BLOCK_NUMBER: BlockNumber = BlockNumber(u32::MAX);
/// The maximum possible index value in block in the storage.
pub const MAX_BLOCK_INDEX: u32 = i32::MAX as u32;

pub fn from_hex(input: &str) -> Result<Vec<u8>, FromHexError> {
    let input = if let Some(input) = input.strip_prefix("0x") {
        input
    } else {
        input
    };
    hex::decode(input)
}

pub fn from_hash_hex(input: &str) -> Result<[u8; 32], FromHexError> {
    let input = if let Some(input) = input.strip_prefix("0x") {
        input
    } else {
        input
    };
    let mut res: [u8; 32] = [0u8; 32];
    decode_to_slice(input, &mut res as &mut [u8])?;
    Ok(res)
}

/// Obtains the database URL from the environment variable.
pub fn get_database_replica_url() -> String {
    env::var("DATABASE_REPLICA_URL").unwrap_or_else(|_| get_database_url())
}

/// Obtains the database URL from the environment variable.
pub fn get_database_url() -> String {
    env::var("DATABASE_URL").expect("DATABASE_URL must be set")
}

/// Storage processor is the main storage interaction point.
/// It holds down the connection (either direct or pooled) to the database
/// and provide methods to obtain different storage schemas.
#[derive(Debug)]
pub struct StorageProcessor<'a> {
    conn: ConnectionHolder<'a>,
    in_transaction: bool,
}

#[derive(sqlx::Type, Debug, Clone, PartialEq, Eq)]
#[sqlx(type_name = "action_type")]
pub enum StorageActionType {
    COMMIT,
    VERIFY,
}

impl From<ActionType> for StorageActionType {
    fn from(action_type: ActionType) -> Self {
        match action_type {
            ActionType::COMMIT => StorageActionType::COMMIT,
            ActionType::VERIFY => StorageActionType::VERIFY,
        }
    }
}

impl<'a> StorageProcessor<'a> {
    /// Creates a `StorageProcessor` using an unique sole connection to the database.
    pub async fn establish_connection<'b>() -> QueryResult<StorageProcessor<'b>> {
        let database_url = get_database_url();
        let connection = PgConnection::connect(&database_url).await?;
        Ok(StorageProcessor {
            conn: ConnectionHolder::Direct(connection),
            in_transaction: false,
        })
    }

    pub async fn start_transaction<'c: 'b, 'b>(
        &'c mut self,
    ) -> Result<StorageProcessor<'b>, anyhow::Error> {
        let transaction = self.conn().begin().await?;

        let mut processor = StorageProcessor::from_transaction(transaction);
        processor.in_transaction = true;

        Ok(processor)
    }

    /// Checks if the `StorageProcessor` is currently within database transaction.
    pub fn in_transaction(&self) -> bool {
        self.in_transaction
    }

    pub fn from_transaction(conn: Transaction<'_, Postgres>) -> StorageProcessor<'_> {
        StorageProcessor {
            conn: ConnectionHolder::Transaction(conn),
            in_transaction: true,
        }
    }

    pub async fn commit(self) -> QueryResult<()> {
        if let ConnectionHolder::Transaction(transaction) = self.conn {
            transaction.commit().await?;
            Ok(())
        } else {
            panic!("StorageProcessor::commit can only be invoked after calling StorageProcessor::begin_transaction");
        }
    }

    /// Creates a `StorageProcessor` using a pool of connections.
    /// This method borrows one of the connections from the pool, and releases it
    /// after `drop`.
    pub fn from_pool(conn: PooledConnection) -> Self {
        Self {
            conn: ConnectionHolder::Pooled(conn),
            in_transaction: false,
        }
    }

    /// Gains access to the `Chain` schemas.
    pub fn chain(&mut self) -> chain::ChainIntermediator<'_, 'a> {
        chain::ChainIntermediator(self)
    }

    fn conn(&mut self) -> &mut PgConnection {
        match &mut self.conn {
            ConnectionHolder::Pooled(conn) => conn,
            ConnectionHolder::Direct(conn) => conn,
            ConnectionHolder::Transaction(conn) => conn,
        }
    }
}