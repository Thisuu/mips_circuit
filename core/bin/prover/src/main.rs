// Workspace deps
use config::configs::ProverConfig as EnvProverConfig;
// Local deps
use prover::{prover_work_cycle, ShutdownRequest};
use storage::ConnectionPool;

#[tokio::main]
async fn main() {
    let mut _vlog_guard = Some(vlog::init());
    // used env
    let prover_options = EnvProverConfig::from_env();

    let shutdown_request = ShutdownRequest::new();

    // Handle termination requests.
    {
        let shutdown_request = shutdown_request.clone();
        ctrlc::set_handler(move || {
            vlog::info!(
                "Termination signal received. It will be handled after the currently working round"
            );

            if shutdown_request.get() {
                vlog::warn!("Second shutdown request received, shutting down without waiting for round to be completed");
                std::process::exit(0);
            }

            shutdown_request.set();
        })
            .expect("Failed to register ctrlc handler");
    }

    let connection_pool = ConnectionPool::new(None);
    prover_work_cycle(connection_pool.clone(),shutdown_request, prover_options).await;
}
