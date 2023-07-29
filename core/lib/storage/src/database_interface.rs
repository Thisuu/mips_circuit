// Built-in deps
// External imports
// Workspace imports
// Local imports
use sqlx::FromRow;
 use sqlx::postgres::PgRow;
use crate::{StorageProcessor, QueryResult};

#[derive(Default)]
pub struct ExecuteResult {
    pub rows_affected: u64,
    // pub last_insert_id: u64,
}

pub trait UtilsMacro {
    fn get_type_name<'a>() -> &'a str;
    fn from_json_str(info: &str) -> Self;
}

#[async_trait::async_trait]
pub trait DatabaseInterface: Send + Sync + Clone + 'static {
    async fn acquire_connection(&self) -> anyhow::Result<StorageProcessor<'_>>;

    async fn execute_sql(
        &self,
        connection: &mut StorageProcessor<'_>,
        sql: &str,
    ) -> QueryResult<ExecuteResult>;

    async fn fetch_optional<'q, O>(
        &self,
        connection: &mut StorageProcessor<'_>,
        sql: &'q str,
    ) -> QueryResult<Option<O>>
        where
            O: for<'r> FromRow<'r, PgRow, > + Unpin + std::marker::Send + UtilsMacro;

    async fn fetch_all<'q, O>(
        &self,
        connection: &mut StorageProcessor<'_>,
        sql: &'q str,
    ) -> QueryResult<Vec<O>>
        where
            O: for<'r> FromRow<'r, PgRow, > + Unpin + std::marker::Send + UtilsMacro;
}