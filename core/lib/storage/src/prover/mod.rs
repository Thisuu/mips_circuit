// Built-in deps
use std::time::Instant;
// External imports
use anyhow::format_err;
// Workspace imports
use types::BlockNumber;
// Local imports
use crate::{QueryResult, StorageProcessor};
use crate::prover::records::{StorageBlockWitnessCloud, StoredProof, StorageTrace, StorageWitnessBlockNumber};

pub mod records;

#[derive(Debug, Clone)]
pub enum ProverJobStatus {
    Idle = 0,
    InProgress = 1,
    Done = 2,
}

impl ProverJobStatus {
    pub fn to_number(&self) -> i32 {
        match self {
            ProverJobStatus::Idle => 0,
            ProverJobStatus::InProgress => 1,
            ProverJobStatus::Done => 2,
        }
    }

    pub fn from_number(num: i32) -> anyhow::Result<Self> {
        Ok(match num {
            0 => Self::Idle,
            1 => Self::InProgress,
            2 => Self::Done,
            _ => anyhow::bail!("Incorrect ProverJobStatus number: {}", num),
        })
    }
}

#[derive(Debug, Clone)]
pub enum ProverJobType {
    SingleProof,
    AggregatedProof,
}

impl ToString for ProverJobType {
    fn to_string(&self) -> String {
        match self {
            ProverJobType::SingleProof => String::from("SINGLE_PROOF"),
            ProverJobType::AggregatedProof => String::from("AGGREGATED_PROOF"),
        }
    }
}

/// Prover schema is capable of handling the prover-related informations,
/// such as started prover jobs, registered provers and proofs for blocks.
#[derive(Debug)]
pub struct ProverSchema<'a, 'c>(pub &'a mut StorageProcessor<'c>);

impl<'a, 'c> ProverSchema<'a, 'c> {
    pub async fn load_last_witness_block_number(
        &mut self,
    ) -> QueryResult<i64> {
        let start = Instant::now();

        let number = sqlx::query_as!(
            StorageWitnessBlockNumber,
            "SELECT * FROM t_witness_block_number ORDER BY f_block DESC LIMIT 1",
        )
            .fetch_optional(self.0.conn())
            .await?;

        metrics::histogram!("sql", start.elapsed(), "prover" => "load_last_witness_block_number");

        if let Some(n) = number {
            return Ok(n.f_block);
        }
        return Ok(1);
    }

    pub async fn update_last_witness_block_number(
        &mut self,
        block: BlockNumber,
    ) -> QueryResult<()> {
        let start = Instant::now();

        sqlx::query!(
            r#"
            INSERT INTO t_witness_block_number (f_id,f_block)
            VALUES(1, $1)
            ON CONFLICT (f_id) DO UPDATE SET f_block = $1
            "#,
            i64::from(*block)
        )
            .execute(self.0.conn())
            .await?;

        metrics::histogram!("sql", start.elapsed(), "prover" => "update_last_witness_block_number");
        Ok(())
    }

    /// Load mips traces
    pub async fn load_trace(
        &mut self,
        block: BlockNumber,
    ) -> QueryResult<Option<String>> {
        let start = Instant::now();

        let trace = sqlx::query_as!(
            StorageTrace,
            "SELECT * FROM f_traces WHERE f_id = $1",
            i64::from(*block),
        )
            .fetch_optional(self.0.conn())
            .await?;

        metrics::histogram!("sql", start.elapsed(), "prover" => "load_trace");

        if let Some(t) = trace {
            let trace: String = serde_json::to_string(&t.f_trace).unwrap();

            return Ok(Some(trace));
        }
        return Ok(None);
    }

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

    /// Stores the proof for a block.
    pub async fn store_proof(
        &mut self,
        job_id: i32,
        block_number: BlockNumber,
        proof: serde_json::Value,
    ) -> QueryResult<()> {
        let start = Instant::now();
        let mut transaction = self.0.start_transaction().await?;

//         let updated_rows = sqlx::query!(
//     r#"
// UPDATE t_prover_job_queue_cloud
//             SET f_updated_at = now(), f_job_status = $1, f_updated_by = 'server_finish_job'
//             WHERE f_id = $2 AND f_job_type = $3
//     "#,
//             ProverJobStatus::Done.to_number(),
//             job_id as i64,
//             ProverJobType::SingleProof.to_string()
// )
//             .execute(transaction.conn())
//             .await?
//             .rows_affected();
//
//         if updated_rows != 1 {
//             return Err(format_err!("Missing job for stored proof"));
//         }

        sqlx::query!(
    r#"
INSERT INTO t_proofs (f_block_number, f_proof)
            VALUES ($1, $2)
    "#,
            i64::from(*block_number),
            proof
)
            .execute(transaction.conn())
            .await?;

        transaction.commit().await?;
        metrics::histogram!("sql", start.elapsed(), "prover" => "store_proof");

        Ok(())
    }

    /// Gets the stored proof for a block.
    pub async fn load_proof(
        &mut self,
        block_number: BlockNumber,
    ) -> QueryResult<Option<StoredProof>> {
        let start = Instant::now();

        let proof = sqlx::query_as!(
        StoredProof,
            "SELECT * FROM t_proofs WHERE f_block_number = $1",
            i64::from(*block_number),
)
            .fetch_optional(self.0.conn())
            .await?;

        metrics::histogram!("sql", start.elapsed(), "prover" => "load_proof");
        Ok(proof)
    }
}