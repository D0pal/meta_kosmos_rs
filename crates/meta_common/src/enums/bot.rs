#![allow(non_camel_case_types)]

use serde::Deserialize;
use strum::{AsRefStr, EnumCount, EnumIter, EnumString, EnumVariantNames};

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
)]
pub enum BotType {
    #[strum(ascii_case_insensitive, serialize = "ATOMIC_SWAP_ROUTER")]
    ATOMIC_SWAP_ROUTER,

    #[strum(ascii_case_insensitive, serialize = "SANDWIDTH_HUFF")]
    SANDWIDTH_HUFF,

    #[strum(ascii_case_insensitive, serialize = "BRAIN_DANCE_SOL")]
    BRAIN_DANCE_SOL,
}
