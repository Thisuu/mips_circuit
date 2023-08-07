use std::time::Instant;
// Built-in
use std::{thread, time};
use std::collections::HashMap;
// External
use futures::channel::mpsc;
use tokio::time::sleep;
// Workspace deps
use crate::database_interface::DatabaseInterface;
use types::BlockNumber;
use utils::panic_notify::ThreadPanicNotify;

/// The essential part of this structure is `maintain` function
/// which runs forever and adds data to the database.
///
/// This will generate and store in db witnesses for blocks with indexes
/// start_block, start_block + block_step, start_block + 2*block_step, ...
pub struct WitnessGenerator<DB: DatabaseInterface> {
    /// Connection to the database.
    database: DB,
    /// Routine refresh interval.
    rounds_interval: time::Duration,

    start_block: BlockNumber,
    block_step: BlockNumber,
}

impl<DB: DatabaseInterface> WitnessGenerator<DB> {
    /// Creates a new `WitnessGenerator` object.
    pub fn new(
        database: DB,
        rounds_interval: time::Duration,
        start_block: BlockNumber,
        block_step: BlockNumber,
    ) -> Self {
        Self {
            database,
            rounds_interval,
            start_block,
            block_step,
        }
    }

    /// Starts the thread running `maintain` method.
    pub fn start(self, panic_notify: mpsc::Sender<bool>) {
        thread::Builder::new()
            .name("prover_server_pool".to_string())
            .spawn(move || {
                let _panic_sentinel = ThreadPanicNotify(panic_notify);
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Unable to build runtime for a witness generator");

                runtime.block_on(async move {
                    self.maintain().await;
                });
            })
            .expect("failed to start provers server");
    }

    async fn maintain(self) {
        vlog::info!(
            "preparing prover data routine started with start_block({}), block_step({})",
            *self.start_block,
            *self.block_step
        );

        // Initialize counters for cache hits/misses.
        metrics::register_counter!("witness_generator.cache_access", "type" => "hit");
        metrics::register_counter!("witness_generator.cache_access", "type" => "off_by_1");
        metrics::register_counter!("witness_generator.cache_access", "type" => "miss");

        let mut current_block = self.start_block;
        loop {
            sleep(self.rounds_interval).await;

            let next_block = BlockNumber(*current_block + *self.block_step);

            let mut block_args = HashMap::new(); // FIXME get specificed block args from db
            block_args.insert("input".to_string(), "/Users/bj89200ml/Documents/rust_workspace/src/github.com/zkMIPS/mips_circuit/core/lib/circuit/out".to_string());
            block_args.insert("abi-spec".to_string(), "/Users/bj89200ml/Documents/rust_workspace/src/github.com/zkMIPS/mips_circuit/core/lib/circuit/abi.json".to_string());
            block_args.insert("arguments".to_string(), r#"[
	[{
		"cycle": "0",
		"pc": "0",
		"nextPC": "0",
		"lo": "0",
		"hi": "0",
		"regs": ["0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0"],
		"preImageKey": ["0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0"],
		"preImageOffset": "0",
		"heap": "0",
		"exitCode": "0",
		"exited": false
	}, {
		"cycle": "0",
		"pc": "0",
		"nextPC": "0",
		"lo": "0",
		"hi": "0",
		"regs": ["0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0"],
		"preImageKey": ["0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0","0", "0", "0", "0"],
		"preImageOffset": "0",
		"heap": "0",
		"exitCode": "0",
		"exited": true
	}],
	[
		"0", "0", "0", "0"
	],
	"0", [
		"0", "0", "0", "0"
	]
]"#.to_string());
            let mut storage = self.database.acquire_connection().await.unwrap();
            let witness_str = circuit::witness::compute_witness(&block_args).unwrap();
            self.database
                .store_witness(
                    &mut storage,
                    next_block,
                    witness_str,
                )
                .await.unwrap();

            // Update current block.
            current_block = next_block;
        }
    }
}