//! The declaration of the most primitive types used in zkSync network.
//! Most of them are just re-exported from the `web3` crate.

#[macro_use]
mod macros;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::ParseIntError;
use std::ops::{Add, Deref, DerefMut, Sub};
use std::str::FromStr;

pub use web3::types::{Address, Log, TransactionReceipt, H160, H256, U128, U256};

pub type Amount = u64;
pub type AssetId = u16;
pub type FundingIndex = u64;
pub type IsBuyingSynthetic = u8;

basic_type!(
    /// Unique identifier of the token in the zkSync network.
    TokenId,
    u32
);

basic_type!(
    /// Unique identifier of the account in the zkSync network.
    AccountId,
    u32
);

basic_type!(
    /// zkSync network block sequential index.
    BlockNumber,
    u32
);

basic_type!(
    /// zkSync account nonce.
    Nonce,
    u32
);

basic_type!(
    /// Unique identifier of the priority operation in the zkSync network.
    PriorityOpId,
    u64
);

basic_type!(
    /// Block number in the Ethereum network.
    EthBlockId,
    u64
);

basic_type!(
    /// Unique identifier of the zkSync event.
    EventId,
    u64
);
basic_type!(
    /// Shared counter for L1 and L2  transactions
    /// This counter is used for total txs/priority ops ordering.
    /// It is required because we generally consider L1 and L2 operations different entities and
    /// store them separately.
    SequentialTxId,
    u64
);

pub type Timestamp = u32;

pub type Price = u64;

pub type ExternalPriceStr = String;

pub type ExternalPrice = u128;

pub type FPX32_32 = u64;