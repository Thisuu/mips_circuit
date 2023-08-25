// Built-in
use std::sync::Arc;
use std::thread;
use std::time::Duration;
// External
use actix_web::dev::ServiceRequest;
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web_httpauth::extractors::{
    bearer::{BearerAuth, Config},
    AuthenticationError,
};
use actix_web_httpauth::middleware::HttpAuthentication;

use jsonwebtoken::errors::Error as JwtError;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
// Workspace deps
use config::ProverConfig;
// Local deps
use self::database_interface::DatabaseInterface;
use self::scaler::ScalerOracle;
use tokio::task::JoinHandle;
use types::BlockNumber;
use utils::panic_notify::{spawn_panic_handler, ThreadPanicNotify};
use config::configs::api::ProverApiConfig;

pub mod database;
pub mod database_interface;
mod scaler;
mod witness_generator;

#[derive(Debug, Serialize, Deserialize)]
struct PayloadAuthToken {
    /// Subject (whom auth token refers to).
    sub: String,
    /// Expiration time (as UTC timestamp).
    exp: usize,
}

pub struct CreateBlockProof {
    pub block_number: BlockNumber,
    pub block_chunks_size: usize,
}

#[derive(Debug, Clone)]
struct AppState<DB: DatabaseInterface> {
    secret_auth: String,
    database: DB,
    scaler_oracle: Arc<RwLock<ScalerOracle<DB>>>,
}

impl<DB: DatabaseInterface> AppState<DB> {
    pub fn new(secret_auth: String, database: DB, idle_provers: u32) -> Self {
        let scaler_oracle = Arc::new(RwLock::new(ScalerOracle::new(
            database.clone(),
            idle_provers,
        )));

        Self {
            secret_auth,
            database,
            scaler_oracle,
        }
    }

    async fn access_storage(&self) -> actix_web::Result<storage::StorageProcessor<'_>> {
        self.database.acquire_connection().await.map_err(|e| {
            vlog::warn!("Failed to access storage: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })
    }
}

/// The structure that stores the secret key for checking JsonWebToken matching.
struct AuthTokenValidator<'a> {
    decoding_key: DecodingKey<'a>,
}

impl<'a> AuthTokenValidator<'a> {
    fn new(secret: &'a str) -> Self {
        Self {
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
        }
    }

    /// Checks whether the secret key and the authorization token match.
    fn validate_auth_token(&self, token: &str) -> Result<(), JwtError> {
        decode::<PayloadAuthToken>(token, &self.decoding_key, &Validation::default())?;

        Ok(())
    }

    async fn validator(
        &self,
        req: ServiceRequest,
        credentials: BearerAuth,
    ) -> actix_web::Result<ServiceRequest> {
        let config = req.app_data::<Config>().cloned().unwrap_or_default();

        self.validate_auth_token(credentials.token())
            .map_err(|_| AuthenticationError::from(config))?;

        Ok(req)
    }
}

async fn status() -> actix_web::Result<String> {
    Ok("alive".into())
}

/// Input of the `/scaler/replicas` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredReplicasInput {
    /// Amount of currently running prover entities.
    current_count: u32,
}

/// Output of the `/scaler/replicas` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredReplicasOutput {
    /// Amount of the prover entities required for server
    /// to run optimally.
    needed_count: u32,
}

pub fn run_prover_server<DB: DatabaseInterface>(
    database: DB,
    prover_api_opts: ProverApiConfig,
    prover_opts: ProverConfig,
) -> JoinHandle<()> {
    let witness_generator_opts = prover_opts.witness_generator;
    let core_opts = prover_opts.core;
    let (handler, panic_sender) = spawn_panic_handler();

    thread::Builder::new()
        .name("prover_server".to_string())
        .spawn(move || {
            let _panic_sentinel = ThreadPanicNotify(panic_sender.clone());
            let actix_runtime = actix_rt::System::new();

            actix_runtime.block_on(async move {

                let last_witness_block = {
                    let mut storage = database
                        .acquire_connection()
                        .await
                        .expect("Failed to access storage");

                    database
                        .load_last_witness_block_number(&mut storage)
                        .await
                        .expect("Failed to get last witness block number")
                        as usize
                };

                // Start pool maintainer threads.
                let start_block = last_witness_block as u32;
                let block_step = 1;
                vlog::info!(
                        "Starting witness generator ({},{})",
                        start_block,
                        block_step
                    );
                let pool_maintainer = witness_generator::WitnessGenerator::new(
                    database.clone(),
                    witness_generator_opts.prepare_data_interval(),
                    BlockNumber(start_block),
                    BlockNumber(block_step),
                );
                pool_maintainer.start(panic_sender.clone());
                // Start HTTP server.
                let secret_auth = prover_api_opts.secret_auth.clone();
                let idle_provers = core_opts.idle_provers;
                HttpServer::new(move || {
                    let app_state =
                        AppState::new(secret_auth.clone(), database.clone(), idle_provers);

                    let auth = HttpAuthentication::bearer(move |req, credentials| async {
                        let secret_auth = req
                            .app_data::<web::Data<AppState<DB>>>()
                            .expect("failed get AppState upon receipt of the authentication token")
                            .secret_auth
                            .clone();
                        AuthTokenValidator::new(&secret_auth)
                            .validator(req, credentials)
                            .await
                    });

                    // By calling `register_data` instead of `data` we're avoiding double
                    // `Arc` wrapping of the object.
                    App::new()
                        .wrap(auth)
                        .app_data(web::Data::new(app_state))
                        .route("/status", web::get().to(status))
                })
                    .bind(&prover_api_opts.bind_addr())
                    .expect("failed to bind")
                    .run()
                    .await
            })
        })
        .expect("failed to start prover server");

    handler
}