// Built-in uses
use std::time::Duration;
// External uses
use serde::Deserialize;
// Local uses
use crate::envy_load;

/// Configuration for the prover application and part of the server that interact with it.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct VerifierConfig {
    pub chain_url: String,
    pub contract_address: String,
    pub account: String,
    pub abi_path: String,
}

impl VerifierConfig {
    pub fn from_env() -> Self {
        envy_load!("verifier", "VERIFIER_")
    }
}

