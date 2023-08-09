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
    loop {
        new_job_poll_timer.tick().await;

        if shutdown.get() {
            break;
        }

        // FIXME
        let mut block_args = HashMap::new();
        block_args.insert("input".to_string(), "/Users/bj89200ml/Documents/rust_workspace/src/github.com/zkMIPS/mips_circuit/core/lib/circuit/out".to_string());
        let block_number = BlockNumber(3);
        let mut storage = database.acquire_connection().await.unwrap();
        let witness_str = database
            .load_witness(&mut storage, block_number)
            .await
            .unwrap()
            .unwrap();
        block_args.insert("backend".to_string(), constants::ARK.to_string());
        block_args.insert("proving-scheme".to_string(), constants::G16.to_string());
        block_args.insert("witness".to_string(), witness_str);
        block_args.insert("proving-key-path".to_string(), "/Users/bj89200ml/Documents/rust_workspace/src/github.com/zkMIPS/mips_circuit/core/lib/circuit/proving.key".to_string());
        let proof_str = circuit::proof::generate_proof(&block_args).unwrap();
        database
            .store_proof(&mut storage,
                         1,
                         block_number,
                         proof_str)
            .await
            .unwrap();
    }
}