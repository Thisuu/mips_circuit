use num::{rational::Ratio, BigUint};
// External uses
use serde::Deserialize;
// Workspace uses
use types::Address;
use utils::scaled_u64_to_ratio;
// Local uses
use crate::envy_load;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
pub enum TokenPriceSource {
    CoinGecko,
    CoinMarketCap,
}

/// Configuration for the fee ticker.
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TickerConfig {
    /// Indicator of the API to be used for getting token prices.
    pub token_price_source: TokenPriceSource,
    /// URL of CoinMarketCap API. Can be set to the mock server for local development.
    pub coinmarketcap_base_url: String,
    /// URL of CoinGecko API. Can be set to the mock server for local development.
    pub coingecko_base_url: String,
    /// Coefficient for scaling all fees in percent.
    pub scale_fee_percent: u32,
    /// Coefficient for the fee price for fast withdrawal requests.
    pub fast_processing_coeff: f64,
    /// Url to uniswap api
    pub uniswap_url: String,
    /// The volume of tokens to confirm their liquidity
    pub liquidity_volume: f64,
    /// Time when liquidity check results are valid
    pub available_liquidity_seconds: u64,
    /// List of the tokens that are unconditionally acceptable for paying fee in.
    pub unconditionally_valid_tokens: Vec<Address>,
    ///
    pub token_market_update_time: u64,
    /// Number of tickers for load balancing.
    pub number_of_ticker_actors: u8,
    /// Subsidized price for ChangePubKey in cents scaled by SUBSIDY_USD_AMOUNTS_SCALE
    pub subsidy_cpk_price_usd_scaled: u64,
}

impl TickerConfig {
    pub fn subsidy_cpk_price_usd(&self) -> Ratio<BigUint> {
        scaled_u64_to_ratio(self.subsidy_cpk_price_usd_scaled)
    }

    pub fn from_env() -> Self {
        envy_load!("fee_ticker", "FEE_TICKER_")
    }

    /// Returns the token price source type and the corresponding API URL.
    pub fn price_source(&self) -> (TokenPriceSource, String) {
        let url = match self.token_price_source {
            TokenPriceSource::CoinGecko => self.coingecko_base_url.clone(),
            TokenPriceSource::CoinMarketCap => self.coinmarketcap_base_url.clone(),
        };
        (self.token_price_source, url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configs::test_utils::{addr, set_env};

    fn expected_config() -> TickerConfig {
        TickerConfig {
            token_price_source: TokenPriceSource::CoinGecko,
            coinmarketcap_base_url: "http://127.0.0.1:9876".into(),
            coingecko_base_url: "http://127.0.0.1:9876".into(),
            scale_fee_percent: 100,
            fast_processing_coeff: 10.0f64,
            uniswap_url: "http://127.0.0.1:9975/graphql".to_string(),
            liquidity_volume: 100.0,
            available_liquidity_seconds: 1000,
            unconditionally_valid_tokens: vec![addr("0000000000000000000000000000000000000000")],
            token_market_update_time: 120,
            number_of_ticker_actors: 4,
            subsidy_cpk_price_usd_scaled: 100,
        }
    }

    #[test]
    fn from_env() {
        let config = r#"
FEE_TICKER_TOKEN_PRICE_SOURCE="CoinGecko"
FEE_TICKER_COINMARKETCAP_BASE_URL="http://127.0.0.1:9876"
FEE_TICKER_COINGECKO_BASE_URL="http://127.0.0.1:9876"
FEE_TICKER_FAST_PROCESSING_COEFF="10"
FEE_TICKER_UNISWAP_URL=http://127.0.0.1:9975/graphql
FEE_TICKER_AVAILABLE_LIQUIDITY_SECONDS=1000
FEE_TICKER_TOKEN_MARKET_UPDATE_TIME=120
FEE_TICKER_UNCONDITIONALLY_VALID_TOKENS="0x0000000000000000000000000000000000000000"
FEE_TICKER_LIQUIDITY_VOLUME=100
FEE_TICKER_NUMBER_OF_TICKER_ACTORS="4"
FEE_TICKER_SUBSIDIZED_TOKENS_LIMITS=156
FEE_TICKER_SCALE_FEE_PERCENT=100
FEE_TICKER_SUBSIDY_CPK_PRICE_USD_SCALED=100
        "#;
        set_env(config);

        let actual = TickerConfig::from_env();
        assert_eq!(actual, expected_config());
    }

    /// Checks the correctness of the config helper methods.
    #[test]
    fn methods() {
        const COINGECKO_URL: &str = "http://coingecko";
        const COINMARKETCAP_URL: &str = "http://coinmarketcap";

        let mut config = expected_config();

        config.coingecko_base_url = COINGECKO_URL.into();
        config.coinmarketcap_base_url = COINMARKETCAP_URL.into();

        config.token_price_source = TokenPriceSource::CoinGecko;
        assert_eq!(
            config.price_source(),
            (TokenPriceSource::CoinGecko, COINGECKO_URL.into())
        );

        config.token_price_source = TokenPriceSource::CoinMarketCap;
        assert_eq!(
            config.price_source(),
            (TokenPriceSource::CoinMarketCap, COINMARKETCAP_URL.into())
        );
    }
}
