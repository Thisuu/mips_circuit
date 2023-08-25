//! Module encapsulating the database interaction.
//! The essential part of this module is the trait that abstracts
//! the database interaction, so no real database is needed to run
//! the prover-server, which is required for tests.

// Built-in
use std::clone::Clone;
use serde_json::Value;
// Workspace uses
use storage::{ConnectionPool, StorageProcessor};
use storage::prover::records::StoredProof;
use types::BlockNumber;
// Local uses
use crate::DatabaseInterface;

const NUMBER_OF_STORED_ACCOUNT_TREE_CACHE: u32 = 300;

/// The actual database wrapper.
/// This structure uses `StorageProcessor` to interact with an existing database.
#[derive(Debug, Clone)]
pub struct Database {
    /// Connection to the database.
    db_pool: ConnectionPool,
}

impl Database {
    pub fn new(db_pool: ConnectionPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait::async_trait]
impl DatabaseInterface for Database {
    async fn acquire_connection(&self) -> anyhow::Result<StorageProcessor<'_>> {
        let connection = self.db_pool.access_storage().await?;

        Ok(connection)
    }

    async fn load_last_proof_block_number(
        &self,
        connection: &mut StorageProcessor<'_>,
    ) -> anyhow::Result<i64> {
        let number = connection.prover_schema().load_last_proof_block_number().await?;

        Ok(number)
    }

    async fn update_last_proof_block_number(
        &self,
        connection: &mut StorageProcessor<'_>,
        block_number: BlockNumber,
    ) -> anyhow::Result<()> {
        connection
            .prover_schema()
            .update_last_proof_block_number(block_number)
            .await?;

        Ok(())
    }

    async fn load_last_witness_block_number(
        &self,
        connection: &mut StorageProcessor<'_>,
    ) -> anyhow::Result<i64> {
        let number = connection.prover_schema().load_last_witness_block_number().await?;

        Ok(number)
    }

    async fn update_last_witness_block_number(
        &self,
        connection: &mut StorageProcessor<'_>,
        block_number: BlockNumber,
    ) -> anyhow::Result<()> {
        connection
            .prover_schema()
            .update_last_witness_block_number(block_number)
            .await?;

        Ok(())
    }

    async fn load_trace(
        &self,
        connection: &mut StorageProcessor<'_>,
        block_number: BlockNumber,
    ) -> anyhow::Result<Option<String>> {
        let trace = connection.prover_schema().load_trace(block_number).await?;

        Ok(trace)
    }

    async fn load_witness(
        &self,
        connection: &mut StorageProcessor<'_>,
        block_number: BlockNumber,
    ) -> anyhow::Result<Option<String>> {
        let witness = connection.prover_schema().get_witness(block_number).await?;

        Ok(witness)
    }

    async fn load_proof(
        &self,
        connection: &mut StorageProcessor<'_>,
        block_number: BlockNumber,
    ) -> anyhow::Result<Option<StoredProof>> {
        let proof = connection.prover_schema().load_proof(block_number).await?;

        Ok(proof)
    }

    async fn store_proof(
        &self,
        connection: &mut StorageProcessor<'_>,
        job_id: i32,
        block_number: BlockNumber,
        proof_str: String,
    ) -> anyhow::Result<()> {
        let proof_value = serde_json::from_str(proof_str.as_str()).unwrap();
        connection
            .prover_schema()
            .store_proof(job_id, block_number, proof_value)
            .await?;

        Ok(())
    }

    async fn store_witness(
        &self,
        connection: &mut StorageProcessor<'_>,
        block: BlockNumber,
        witness: String,
    ) -> anyhow::Result<()> {
        connection
            .prover_schema()
            .store_witness(block, witness)
            .await?;

        Ok(())
    }
}
