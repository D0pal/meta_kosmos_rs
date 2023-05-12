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
pub enum Token {
    #[strum(ascii_case_insensitive, serialize = "EMPTY")]
    EMPTY,

    #[strum(ascii_case_insensitive, serialize = "WBNB")]
    WBNB,

    #[strum(ascii_case_insensitive, serialize = "WETH")]
    WETH,

    #[strum(ascii_case_insensitive, serialize = "USDC")]
    USDC,

    #[strum(ascii_case_insensitive, serialize = "DAI")]
    DAI,

    #[strum(ascii_case_insensitive, serialize = "BUSD")]
    BUSD,

    #[strum(ascii_case_insensitive, serialize = "SFP")]
    SFP,

    #[strum(ascii_case_insensitive, serialize = "CAKE")]
    CAKE,

    #[strum(ascii_case_insensitive, serialize = "TWT")]
    TWT,

    #[strum(ascii_case_insensitive, serialize = "C98")]
    C98,
}

impl Into<String> for Token {
    fn into(self) -> String {
        return self.to_string();
    }
}


impl Default for Token {
    fn default() -> Self {
        Self::WETH
    }
}