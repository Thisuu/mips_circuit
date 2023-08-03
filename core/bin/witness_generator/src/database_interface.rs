//! Module encapsulating the database interaction.
//! The essential part of this module is the trait that abstracts
//! the database interaction, so no real database is needed to run
//! the prover-server, which is required for tests.

// Built-in
use std::clone::Clone;
use std::marker::{Send, Sync};
use serde_json::Value;
use storage::prover::records::StoredProof;
// Workspace uses
use storage::StorageProcessor;
use types::BlockNumber;

/// Abstract database access trait.
#[async_trait::async_trait]
pub trait DatabaseInterface: Send + Sync + Clone + 'static {
    /// Returns connection to the database.
    async fn acquire_connection(&self) -> anyhow::Result<StorageProcessor<'_>>;

    /// Returns stored witness for a block.
    async fn load_witness(
        &self,
        connection: &mut StorageProcessor<'_>,
        block_number: BlockNumber,
    ) -> anyhow::Result<Option<String>>;

    async fn load_proof(
        &self,
        connection: &mut StorageProcessor<'_>,
        block_number: BlockNumber,
    ) -> anyhow::Result<Option<StoredProof>>;

    async fn store_proof(
        &self,
        connection: &mut StorageProcessor<'_>,
        job_id: i32,
        block_number: BlockNumber,
        proof: &StoredProof,
    ) -> anyhow::Result<()>;

    async fn store_witness(
        &self,
        connection: &mut StorageProcessor<'_>,
        block: BlockNumber,
        witness: String,
    ) -> anyhow::Result<()>;
}
