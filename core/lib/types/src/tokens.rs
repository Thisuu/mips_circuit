use chrono::{DateTime, Utc};
use num::{rational::Ratio, BigUint};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, fs::read_to_string, path::PathBuf, str::FromStr};
use thiserror::Error;
use basic_types::{AccountId, Address, H256, Log, TokenId, U256};

/// ID of the ETH token in zkSync network.
use utils::{parse_env, UnsignedRatioSerializeAsDecimal};

#[derive(Debug, Error)]
pub enum NewTokenEventParseError {
    #[error("Cannot parse log for New Token Event {0:?}")]
    ParseError(Log),
}

/// ERC-20 standard token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// Address (prefixed with 0x)
    pub address: Address,
    /// Powers of 10 in 1.0 token (18 for default ETH-like tokens)
    pub decimals: u8,
    /// Token symbol
    pub symbol: String,
}

impl TokenInfo {
    pub fn new(address: Address, symbol: &str, decimals: u8) -> Self {
        Self {
            address,
            symbol: symbol.to_string(),
            decimals,
        }
    }
}

/// Tokens that added through a contract.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewTokenEvent {
    pub eth_block_number: u64,
    pub address: Address,
    pub id: TokenId,
}

impl TryFrom<Log> for NewTokenEvent {
    type Error = NewTokenEventParseError;

    fn try_from(event: Log) -> Result<NewTokenEvent, NewTokenEventParseError> {
        // `event NewToken(address indexed token, uint16 indexed tokenId)`
        //  Event has such a signature, so let's check that the number of topics is equal to the number of parameters + 1.
        if event.topics.len() != 3 {
            return Err(NewTokenEventParseError::ParseError(event));
        }

        let eth_block_number = match event.block_number {
            Some(block_number) => block_number.as_u64(),
            None => {
                return Err(NewTokenEventParseError::ParseError(event));
            }
        };

        Ok(NewTokenEvent {
            eth_block_number,
            address: Address::from_slice(&event.topics[1].as_fixed_bytes()[12..]),
            id: TokenId(U256::from_big_endian(&event.topics[2].as_fixed_bytes()[..]).as_u32()),
        })
    }
}

// Hidden as it relies on the filesystem structure, which can be different for reverse dependencies.
#[doc(hidden)]
pub fn get_genesis_token_list(network: &str) -> Result<Vec<TokenInfo>, GetGenesisTokenListError> {
    let mut file_path = parse_env::<PathBuf>("ZKSYNC_HOME");
    file_path.push("etc");
    file_path.push("tokens");
    file_path.push(network);
    file_path.set_extension("json");
    Ok(serde_json::from_str(&read_to_string(file_path)?)?)
}

pub fn get_token_id_by_symbol(network: &str, symbol: &str) -> TokenId {
    let tokens = get_genesis_token_list(network).unwrap();
    for (id, token) in (1..).zip(tokens) {
        if token.symbol == symbol {
            return TokenId(id);
        }
    }
    return TokenId(0);
}

pub fn get_token_info_by_symbol(network: &str, symbol: &str) -> Option<TokenInfo> {
    let tokens = get_genesis_token_list(network).unwrap();
    for token in tokens {
        if token.symbol == symbol {
            return Some(token);
        }
    }
    return None;
}

/// Token price known to the zkSync network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPrice {
    #[serde(with = "UnsignedRatioSerializeAsDecimal")]
    pub usd_price: Ratio<BigUint>,
    pub last_updated: DateTime<Utc>,
}

/// Token price known to the zkSync network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMarketVolume {
    #[serde(with = "UnsignedRatioSerializeAsDecimal")]
    pub market_volume: Ratio<BigUint>,
    pub last_updated: DateTime<Utc>,
}

/// NFT supported in zkSync protocol
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NFT {
    /// id is used for tx signature and serialization
    pub id: TokenId,
    /// id for enforcing uniqueness token address
    pub serial_id: u32,
    /// id of nft creator
    pub creator_address: Address,
    /// id of nft creator
    pub creator_id: AccountId,
    /// L2 token address
    pub address: Address,
    /// token symbol
    pub symbol: String,
    /// hash of content for nft token
    pub content_hash: H256,
}

impl NFT {
    pub fn new(
        token_id: TokenId,
        serial_id: u32,
        creator_id: AccountId,
        creator_address: Address,
        address: Address,
        symbol: Option<String>,
        content_hash: H256,
    ) -> Self {
        let symbol = symbol.unwrap_or_else(|| format!("NFT-{}", token_id));
        Self {
            id: token_id,
            serial_id,
            creator_address,
            creator_id,
            address,
            symbol,
            content_hash,
        }
    }
}

#[derive(Debug, Error, PartialEq)]
#[error("Incorrect ProverJobStatus number: {0}")]
pub struct IncorrectProverJobStatus(pub i32);

#[derive(Debug, Error)]
pub enum GetGenesisTokenListError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_fee_type_deserialize_old_type() {
        let deserialized: TxFeeTypes =
            serde_json::from_str(r#"{ "ChangePubKey": { "onchainPubkeyAuth": true }}"#).unwrap();

        assert_eq!(
            deserialized,
            TxFeeTypes::ChangePubKey(ChangePubKeyFeeTypeArg::PreContracts4Version {
                onchain_pubkey_auth: true,
            })
        );

        let deserialized: TxFeeTypes =
            serde_json::from_str(r#"{ "ChangePubKey": { "onchainPubkeyAuth": false }}"#).unwrap();
        assert_eq!(
            deserialized,
            TxFeeTypes::ChangePubKey(ChangePubKeyFeeTypeArg::PreContracts4Version {
                onchain_pubkey_auth: false,
            })
        );
    }

    #[test]
    fn tx_fee_type_deserialize() {
        let deserialized: TxFeeTypes =
            serde_json::from_str(r#"{ "ChangePubKey": "Onchain" }"#).unwrap();

        assert_eq!(
            deserialized,
            TxFeeTypes::ChangePubKey(ChangePubKeyFeeTypeArg::ContractsV4Version(
                ChangePubKeyType::Onchain
            ))
        );

        let deserialized: TxFeeTypes =
            serde_json::from_str(r#"{ "ChangePubKey": "ECDSA" }"#).unwrap();

        assert_eq!(
            deserialized,
            TxFeeTypes::ChangePubKey(ChangePubKeyFeeTypeArg::ContractsV4Version(
                ChangePubKeyType::ECDSA
            ))
        );

        let deserialized: TxFeeTypes =
            serde_json::from_str(r#"{ "ChangePubKey": "CREATE2" }"#).unwrap();

        assert_eq!(
            deserialized,
            TxFeeTypes::ChangePubKey(ChangePubKeyFeeTypeArg::ContractsV4Version(
                ChangePubKeyType::CREATE2
            ))
        );
    }

    #[test]
    fn token_like_is_eth() {
        let tokens = vec![
            TokenLike::Address(Address::zero()),
            TokenLike::Id(TokenId(0)),
            TokenLike::Symbol("ETH".into()),
            TokenLike::Symbol("eth".into()),
        ];

        for token in tokens {
            assert!(token.is_eth());
        }
    }

    #[test]
    fn token_like_to_case_insensitive() {
        assert_eq!(
            TokenLike::Symbol("ETH".into()).to_lowercase(),
            TokenLike::Symbol("eth".into())
        );
    }
}
