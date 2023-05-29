use strum::{AsRefStr, EnumCount, EnumIter, EnumString, EnumVariantNames, Display};
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
    Display,
)]
// #[serde(rename_all = "snake_case")]
pub enum Network {

    #[strum(ascii_case_insensitive, serialize = "ETH")]
    ETH,

    #[strum(ascii_case_insensitive, serialize = "ETH_GOERLI")]
    ETH_GOERLI,

    #[strum(ascii_case_insensitive, serialize = "BSC")]
    BSC,

    #[strum(ascii_case_insensitive, serialize = "BSC_TEST")]
    BSC_TEST,

    #[strum(ascii_case_insensitive, serialize = "ZK_SYNC_ERA")]
    ZK_SYNC_ERA,

    #[strum(ascii_case_insensitive, serialize = "ZK_SYNC_ERA_TEST")]
    ZK_SYNC_ERA_TEST,
}

// This must be implemented manually so we avoid a conflict with `TryFromPrimitive` where it treats
// the `#[default]` attribute as its own `#[num_enum(default)]`
impl Default for Network {
    fn default() -> Self {
        Self::BSC
    }
}

// impl Serialize for Network {
//     fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         s.serialize_str(self.as_ref())
//     }
// }

#[test]
fn test_default_chain() {
    assert_eq!(Network::default(), Network::BSC);
}