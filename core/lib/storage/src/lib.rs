pub mod connection;

// Local imports
use crate::connection::{holder::ConnectionHolder, PooledConnection};

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

    /// Gains access to the `Config` schema.
    pub fn config_schema(&mut self) -> config::ConfigSchema<'_, 'a> {
        config::ConfigSchema(self)
    }

    fn conn(&mut self) -> &mut PgConnection {
        match &mut self.conn {
            ConnectionHolder::Pooled(conn) => conn,
            ConnectionHolder::Direct(conn) => conn,
            ConnectionHolder::Transaction(conn) => conn,
        }
    }
}