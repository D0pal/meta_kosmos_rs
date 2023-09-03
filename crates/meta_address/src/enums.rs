use crate::Token;
use serde::Deserialize;
use strum::{AsRefStr, Display, EnumCount, EnumIter, EnumString, EnumVariantNames};

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsRefStr,         // AsRef<str>, fmt::Display and serde::Serialize
    EnumVariantNames, // Chain::VARIANTS
    EnumString,       // FromStr, TryFrom<&str>
    EnumIter,         // Chain::iter
    EnumCount,        // Chain::COUNT
    // TryFromPrimitive, // TryFrom<u64>
    Deserialize,
    Display,
)]
pub enum Asset {
    #[strum(ascii_case_insensitive, serialize = "ETH")]
    ETH,

    #[strum(ascii_case_insensitive, serialize = "ARB")]
    ARB,

    #[strum(ascii_case_insensitive, serialize = "BTC")]
    BTC,

    #[strum(ascii_case_insensitive, serialize = "USD")]
    USD,
}

impl Into<String> for Asset {
    fn into(self) -> String {
        return self.to_string();
    }
}

impl Into<Token> for Asset {
    fn into(self) -> Token {
        match self {
            Asset::BTC => Token::BTC,
            Asset::ARB => Token::ARB,
            Asset::ETH => Token::WETH,
            Asset::USD => Token::USDC,
        }
    }
}

impl Default for Asset {
    fn default() -> Self {
        Self::USD
    }
}
