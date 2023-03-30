use strum::{AsRefStr, EnumCount, EnumIter, EnumString, EnumVariantNames};
use serde::{Deserialize};

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
pub enum Bot {
    #[strum(ascii_case_insensitive, serialize = "FLASHBOT_ROUTER")]
    FLASHBOT_ROUTER,
}
