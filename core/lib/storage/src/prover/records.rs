// External imports
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
// Workspace imports
// Local imports
use utils_macro_driver::UtilsMacro;
// Workspace imports
use crate::database_interface::UtilsMacro;

#[derive(Debug, FromRow)]
pub struct ActiveProver {
    pub id: i32,
    pub worker: String,
    pub created_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub block_size: i64,
}

#[derive(Debug, FromRow)]
pub struct NewProof {
    pub block_number: i64,
    pub proof: serde_json::Value,
}

#[derive(Debug, FromRow)]
pub struct StoredAggregatedProof {
    pub first_block: i64,
    pub last_block: i64,
    pub proof: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// Every time before a prover worker starts generating the proof, a prover run is recorded for monitoring purposes
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ProverRun {
    pub id: i32,
    pub block_number: i64,
    pub worker: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct IntegerNumber {
    pub integer_value: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct StorageBlockWitness {
    pub block: i64,
    pub witness: String,
}

#[derive(Debug, FromRow)]
pub struct StorageProverJobQueue {
    pub id: i32,
    pub job_status: i32,
    pub job_priority: i32,
    pub job_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_by: String,
    pub updated_at: DateTime<Utc>,
    pub first_block: i64,
    pub last_block: i64,
    pub job_data: serde_json::Value,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, UtilsMacro)]
pub struct StorageBlockWitnessCloud {
    pub f_id: i64,
    pub f_block: i64,
    pub f_version: i64,
    pub f_object_key: String,
    pub f_object_witness: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize, UtilsMacro)]
pub struct StoredProof {
    pub f_id: i64,
    pub f_block_number: i64,
    pub f_proof: serde_json::Value,
    pub f_created_at: DateTime<Utc>,

}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, UtilsMacro)]
pub struct StorageTrace {
    pub f_id: i64,
    pub f_trace: serde_json::Value,
    pub f_created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, UtilsMacro)]
pub struct StorageWitnessBlockNumber {
    pub f_id: i64,
    pub f_block: i64,
}