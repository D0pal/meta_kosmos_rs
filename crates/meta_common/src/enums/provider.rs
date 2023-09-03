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
// #[serde(rename_all = "snake_case")]
pub enum RpcProvider {
    #[strum(ascii_case_insensitive, serialize = "quick")]
    Quick,

    #[strum(ascii_case_insensitive, serialize = "ankr")]
    Ankr,

    #[strum(ascii_case_insensitive, serialize = "custom")]
    Custom,

    #[strum(ascii_case_insensitive, serialize = "official")]
    Official,
}

impl Default for RpcProvider {
    fn default() -> Self {
        RpcProvider::Official
    }
}
