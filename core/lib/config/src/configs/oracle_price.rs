// External uses
use serde::Deserialize;
// Local uses
use crate::envy_load;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct OraclePriceConfig {
    provider_keys: Vec<String>,
}

impl OraclePriceConfig {
    pub fn from_env() -> Self {
        envy_load!("oracle_price", "ORACLE_PRICE_")
    }

    pub fn provider_keys(&self) -> Vec<String> {
        self.provider_keys.clone()
    }
}