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
// Local deps
use self::database_interface::DatabaseInterface;
use self::scaler::ScalerOracle;
use tokio::task::JoinHandle;
use types::BlockNumber;
use utils::panic_notify::{spawn_panic_handler, ThreadPanicNotify};

pub mod database;
mod database_interface;
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
