use std::collections::HashMap;
// Built-in deps
use std::sync::{
    atomic::{AtomicBool, AtomicI32, Ordering},
    Arc,
};
use tokio::sync::oneshot;
// Workspace deps
use config::ProverConfig as EnvProverConfig;
use storage::ConnectionPool;
use types::BlockNumber;
use witness_generator::database_interface::DatabaseInterface;
use zokrates_common::constants;

const ABSENT_PROVER_ID: i32 = -1;

#[derive(Debug, Clone)]
pub struct ShutdownRequest {
    shutdown_requested: Arc<AtomicBool>,
    prover_id: Arc<AtomicI32>,
}

impl Default for ShutdownRequest {
    fn default() -> Self {
        let prover_id = Arc::new(AtomicI32::from(ABSENT_PROVER_ID));

        Self {
            shutdown_requested: Default::default(),
            prover_id,
        }
    }
}

impl ShutdownRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_prover_id(&self, id: i32) {
        self.prover_id.store(id, Ordering::SeqCst);
    }

    pub fn prover_id(&self) -> i32 {
        self.prover_id.load(Ordering::SeqCst)
    }

    pub fn set(&self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);
    }

    pub fn get(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }
}

pub async fn prover_work_cycle(
    connection_pool: ConnectionPool,
    shutdown: ShutdownRequest,
    prover_options: EnvProverConfig,
) {
    vlog::info!("Running worker cycle");
    let database = witness_generator::database::Database::new(connection_pool);
    let mut new_job_poll_timer = tokio::time::interval(prover_options.prover.cycle_wait());
    let mut current_block = BlockNumber(1);
    loop {
        new_job_poll_timer.tick().await;

        if shutdown.get() {
            break;
        }

        let mut block_args = HashMap::new();
        let circuit_path = std::env::var("CIRCUIT_FILE_PATH").unwrap();
        let proving_key_path = std::env::var("CIRCUIT_PROVING_KEY_PATH").unwrap();
        block_args.insert("input".to_string(), circuit_path);
        let mut storage = database.acquire_connection().await.unwrap();
        current_block = BlockNumber(database.load_last_proof_block_number(&mut storage).await.unwrap() as u32);
        let next_block = BlockNumber(*current_block + 1);
        let witness_str = database
            .load_witness(&mut storage, current_block)
            .await
            .unwrap();
        if witness_str.is_none() {
            continue;
        }

        block_args.insert("backend".to_string(), constants::ARK.to_string());
        block_args.insert("proving-scheme".to_string(), constants::G16.to_string());
        block_args.insert("witness".to_string(), witness_str.unwrap());
        block_args.insert("proving-key-path".to_string(), proving_key_path);
        let proof_str = circuit::proof::generate_proof(&block_args).unwrap();
        let mut transaction = storage.start_transaction().await.unwrap();
        database
            .store_proof(&mut transaction,
                         1,
                         current_block,
                         proof_str)
            .await
            .unwrap();
        database
            .update_last_proof_block_number(
                &mut transaction,
                next_block
            )
            .await.unwrap();
        transaction.commit().await.unwrap();

        // Update current block.
        current_block = next_block;
    }
}