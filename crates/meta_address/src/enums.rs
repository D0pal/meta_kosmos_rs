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

    #[strum(ascii_case_insensitive, serialize = "BNB")]
    BNB,

    #[strum(ascii_case_insensitive, serialize = "BTC")]
    BTC,

    #[strum(ascii_case_insensitive, serialize = "USD")]
    USD,

    #[strum(ascii_case_insensitive, serialize = "ARB")]
    ARB,

    #[strum(ascii_case_insensitive, serialize = "LEO")]
    LEO,

    #[strum(ascii_case_insensitive, serialize = "ETHW")]
    ETHW,

    #[strum(ascii_case_insensitive, serialize = "APT")]
    APT,

    #[strum(ascii_case_insensitive, serialize = "UST")]
    UST,

    #[strum(ascii_case_insensitive, serialize = "USDT")]
    USDT,

    #[strum(ascii_case_insensitive, serialize = "UDC")]
    UDC,

    #[strum(ascii_case_insensitive, serialize = "OMG")]
    OMG,

    #[strum(ascii_case_insensitive, serialize = "BOBA")]
    BOBA,

    // bitfinex margin
    #[strum(ascii_case_insensitive, serialize = "USTF0")]
    USTF0,
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
            _ => {
                let token_str: String = self.to_string();
                let asset = token_str.parse::<Token>().expect("cannot parse string to Token");
                asset
            }
        }
    }
}

impl From<Token> for Asset {
    fn from(val: Token) -> Asset {
        match val {
            Token::BTC => Asset::BTC,
            Token::ARB => Asset::ARB,
            Token::WETH => Asset::ETH,
            Token::USDC => Asset::USD,
            _ => {
                let token_str: String = val.into();
                let asset = token_str.parse::<Asset>().unwrap();
                asset
            }
        }
    }
}

impl Default for Asset {
    fn default() -> Self {
        Self::USD
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_asset_from_str() {
        let output = "ETH".parse::<Asset>().unwrap();
        assert_eq!(output, Asset::ETH);
    }
}
