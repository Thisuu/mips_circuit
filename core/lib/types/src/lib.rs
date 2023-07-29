use serde::{Deserialize, Serialize};

pub use basic_types::*;

pub type SerialId = u64;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub enum ActionType {
    COMMIT,
    VERIFY,
}

impl std::string::ToString for ActionType {
    fn to_string(&self) -> String {
        match self {
            ActionType::COMMIT => "COMMIT".to_owned(),
            ActionType::VERIFY => "VERIFY".to_owned(),
        }
    }
}

impl std::str::FromStr for ActionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "COMMIT" => Ok(Self::COMMIT),
            "VERIFY" => Ok(Self::VERIFY),
            _ => Err("Should be either: COMMIT or VERIFY".to_owned()),
        }
    }
}
