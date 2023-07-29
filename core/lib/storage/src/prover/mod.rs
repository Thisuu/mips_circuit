// Built-in deps
use std::time::Instant;
// External imports
use anyhow::format_err;
// Workspace imports
use types::BlockNumber;
// Local imports
use crate::{QueryResult, StorageProcessor};
use crate::prover::records::StorageBlockWitnessCloud;

pub mod records;

/// Prover schema is capable of handling the prover-related informations,
/// such as started prover jobs, registered provers and proofs for blocks.
#[derive(Debug)]
pub struct ProverSchema<'a, 'c>(pub &'a mut StorageProcessor<'c>);

impl<'a, 'c> ProverSchema<'a, 'c> {

    /// Stores witness for a block
    pub async fn store_witness(
        &mut self,
        block: BlockNumber,
        witness_str: String,
    ) -> QueryResult<()> {
        let start = Instant::now();
        let key = format!("{}", block);

        sqlx::query!(
            r#"
            INSERT INTO t_block_witness_cloud (f_block, f_version, f_object_key, f_object_witness)
            VALUES($1, 0, $2, $3)
            "#,
            i64::from(*block),
            key,
            witness_str
        )
            .execute(self.0.conn())
            .await?;

        metrics::histogram!("sql", start.elapsed(), "prover" => "store_witness");
        Ok(())
    }

    /// Gets stored witness for a block.
    pub async fn get_witness(
        &mut self,
        block_number: BlockNumber,
    ) -> QueryResult<Option<String>> {
        let start = Instant::now();
        let block_witness = sqlx::query_as!(
            StorageBlockWitnessCloud,
            "SELECT * FROM t_block_witness_cloud WHERE f_block = $1",
            i64::from(*block_number),
        )
            .fetch_optional(self.0.conn())
            .await?;

        metrics::histogram!("sql", start.elapsed(), "prover" => "get_witness");
        if let Some(w) = block_witness {
            let witness_string: String = w.f_object_witness;

            return Ok(Some(witness_string));

        }
        return Ok(None);
    }
}