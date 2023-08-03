use futures::{channel::mpsc, executor::block_on, SinkExt, StreamExt};
use std::cell::RefCell;
use std::str::FromStr;

use structopt::StructOpt;

use serde::{Deserialize, Serialize};

use witness_generator::run_prover_server;

use tokio::task::JoinHandle;
use zksync_storage::ConnectionPool;

const DEFAULT_CHANNEL_CAPACITY: usize = 32_768;

#[derive(Debug, Clone, Copy)]
pub enum ServerCommand {
    Genesis,
    Launch,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum Component {
    // Api components
    RestApi,
    Web3Api,
    RpcApi,
    RpcWebSocketApi,

    // Core components
    Fetchers,
    EthSender,
    Core,
    WitnessGenerator,
    ForcedExit,

    // Additional components
    Prometheus,
    PrometheusPeriodicMetrics,
    RejectedTaskCleaner,
}

impl FromStr for Component {
    type Err = String;

    fn from_str(s: &str) -> Result<Component, String> {
        match s {
            "rest-api" => Ok(Component::RestApi),
            "web3-api" => Ok(Component::Web3Api),
            "rpc-api" => Ok(Component::RpcApi),
            "rpc-websocket-api" => Ok(Component::RpcWebSocketApi),
            "eth-sender" => Ok(Component::EthSender),
            "witness-generator" => Ok(Component::WitnessGenerator),
            "forced-exit" => Ok(Component::ForcedExit),
            "prometheus" => Ok(Component::Prometheus),
            "fetchers" => Ok(Component::Fetchers),
            "core" => Ok(Component::Core),
            "rejected-task-cleaner" => Ok(Component::RejectedTaskCleaner),
            "prometheus-periodic-metrics" => Ok(Component::PrometheusPeriodicMetrics),
            other => Err(format!("{} is not a valid component name", other)),
        }
    }
}

#[derive(Debug)]
struct ComponentsToRun(Vec<Component>);

impl Default for ComponentsToRun {
    fn default() -> Self {
        Self(vec![
            Component::RestApi,
            Component::Web3Api,
            Component::RpcApi,
            Component::RpcWebSocketApi,
            Component::EthSender,
            Component::WitnessGenerator,
            Component::ForcedExit,
            Component::Prometheus,
            Component::Core,
            Component::RejectedTaskCleaner,
            Component::Fetchers,
            Component::PrometheusPeriodicMetrics,
        ])
    }
}

impl FromStr for ComponentsToRun {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split(',')
                .map(|x| Component::from_str(x.trim()))
                .collect::<Result<Vec<Component>, String>>()?,
        ))
    }
}

#[derive(StructOpt)]
#[structopt(name = "zkm operator node", author = "ZKM")]
struct Opt {
    /// Generate genesis block for the first contract deployment
    #[structopt(long)]
    genesis: bool,
    /// comma-separated list of components to launch
    #[structopt(
        long,
        default_value = "rest-api,web3-api,rpc-api,rpc-websocket-api,eth-sender,witness-generator,forced-exit,prometheus,core,rejected-task-cleaner,fetchers,prometheus-periodic-metrics"
    )]
    components: ComponentsToRun,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let mut _vlog_guard = None;
    let server_mode = if opt.genesis {
        _vlog_guard = Some(vlog::init());
        ServerCommand::Genesis
    } else {
        _vlog_guard = Some(vlog::init());
        ServerCommand::Launch
    };

    if let ServerCommand::Genesis = server_mode {
        vlog::info!("Performing the server genesis initialization",);
        // let config = ChainConfig::from_env();
        // genesis_init(&config).await;
        return Ok(());
    }

    // It's a `ServerCommand::Launch`, perform the usual routine.
    vlog::info!("Running the zkm server");
    run_server(&opt.components).await;

    Ok(())
}

async fn run_server(components: &ComponentsToRun) {
    let connection_pool = ConnectionPool::new(None);
    let read_only_connection_pool = ConnectionPool::new_readonly_pool(None);
    let (stop_signal_sender, mut stop_signal_receiver) = mpsc::channel(256);
    let (alternative_tx_stop_signal_sender, mut alternative_tx_stop_signal_receiver) = mpsc::channel(256);

    let mut tasks = vec![];

    if components.0.contains(&Component::WitnessGenerator) {
        tasks.push(run_witness_generator(connection_pool.clone()))
    }

    {
        let stop_signal_sender = RefCell::new(stop_signal_sender.clone());
        ctrlc::set_handler(move || {
            let mut sender = stop_signal_sender.borrow_mut();
            block_on(sender.send(true)).expect("Ctrl+C signal send");
        })
        .expect("Error setting Ctrl+C handler");
    }

    tokio::select! {
        _ = async { wait_for_tasks(tasks).await } => {
            panic!("One if the actors is not supposed to finish its execution")
        },
        _ = async { stop_signal_receiver.next().await } => {
            vlog::warn!("Stop signal received, shutting down");
        }
        _ = async { alternative_tx_stop_signal_receiver.next().await } => {
            vlog::warn!("Alternative tx Stop signal received, shutting down");
        }
    };
}


pub fn run_witness_generator(connection_pool: ConnectionPool) -> JoinHandle<()> {
    vlog::info!("Starting the Prover server actors");
    let prover_api_config = ProverApiConfig::from_env();
    let prover_config = ProverConfig::from_env();
    let database = zksync_witness_generator::database::Database::new(connection_pool);
    run_prover_server(database, prover_api_config, prover_config)
}