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
use zokrates_proof_systems::{Proof,G16};
use zokrates_field::Bn128Field;


/// The essential part of this structure is `maintain` function
/// which runs forever and adds data to the database.
///
/// This will generate and store in db witnesses for blocks with indexes
/// start_block, start_block + block_step, start_block + 2*block_step, ...
pub struct VerifierGenerator<DB: DatabaseInterface> {
    /// Connection to the database.
    database: DB,
    /// Routine refresh interval.
    rounds_interval: time::Duration,

    start_block: BlockNumber,
    block_step: BlockNumber,

    chain_url: String,
    contract_address: String,
    abi_path: String,
    account: String,

}

impl<DB: DatabaseInterface> VerifierGenerator<DB> {
    /// Creates a new `WitnessGenerator` object.
    pub fn new(
        database: DB,
        rounds_interval: time::Duration,
        start_block: BlockNumber,
        block_step: BlockNumber,
        chain_url: String,
        contract_address: String,
        abi_path: String,
        account: String,
    ) -> Self {
        Self {
            database,
            rounds_interval,
            start_block,
            block_step,
            chain_url,
            contract_address,
            abi_path,
            account,
        }
    }

    /// Starts the thread running `maintain` method.
    pub fn start(self, panic_notify: mpsc::Sender<bool>) {
        thread::Builder::new()
            .name("verifier_server_pool".to_string())
            .spawn(move || {
                let _panic_sentinel = ThreadPanicNotify(panic_notify);
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Unable to build runtime for a verifier generator");

                runtime.block_on(async move {
                    self.maintain().await;
                });
            })
            .expect("failed to start verifier server");
    }

    async fn maintain(self) {
        vlog::info!(
            "preparing verifier data routine started with start_block({}), block_step({})",
            *self.start_block,
            *self.block_step
        );

        // Initialize counters for cache hits/misses.

        let mut current_block = self.start_block;
        loop {
            sleep(self.rounds_interval).await;

            let mut storage = self.database.acquire_connection().await.unwrap();
            let next_block = BlockNumber(*current_block + *self.block_step);
            let proof_storage = self
                .database
                .load_proof(&mut storage,current_block+1)
                .await
                .unwrap();

            if proof_storage.is_none() {
                continue;
            }

            let proof: Proof<Bn128Field, G16> = serde_json::from_value(proof_storage.unwrap().f_proof).unwrap();


            let result = circuit::proof::call_verify(proof, self.chain_url.as_str(), self.contract_address.as_str(),self.abi_path.as_str(),self.account.as_str()).await;
            if result {
                let mut transaction = storage.start_transaction().await.unwrap();
                self.database
                    .update_last_verified_proof_block_number(
                        &mut transaction,
                        next_block,
                    )
                    .await.unwrap();

                transaction.commit().await.unwrap();

                // Update current block.
                current_block = next_block;
            }
        }
    }
}