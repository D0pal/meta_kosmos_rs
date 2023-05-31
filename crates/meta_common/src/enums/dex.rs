use serde::Deserialize;
use strum::{AsRefStr, EnumCount, EnumIter, EnumString, EnumVariantNames, Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoolVariant {
    UniswapV2,
    UniswapV3,
}

#[derive(
    Default,
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
pub enum DexExchange {
    #[default]
    None,

    #[strum(ascii_case_insensitive, serialize = "PANCAKE")]
    PANCAKE,

    #[strum(ascii_case_insensitive, serialize = "BISWAP")]
    BISWAP,

    #[strum(ascii_case_insensitive, serialize = "SYNC_SWAP")]
    SYNC_SWAP,

    #[strum(ascii_case_insensitive, serialize = "MUTE_SWITCH")]
    MUTE_SWITCH,

    #[strum(ascii_case_insensitive, serialize = "UNISWAP_V2")]
    UNISWAP_V2,
}

impl Into<String> for DexExchange {
    fn into(self) -> String {
        return self.to_string();
    }
}

// impl From<&str> for DexExchange {
//     fn from(input: &str) -> DexExchange {
//         return input.parse().expect("unable to convert string into Dex");
//     }
// }

#[derive(
    Clone, Debug, Eq, PartialEq, Hash, strum_macros::Display, Deserialize, strum_macros::EnumString,
)]
// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Deserialize, EnumVariantNames)]
pub enum ContractType {
    #[strum(ascii_case_insensitive, serialize = "UNI_V2_FACTORY")]
    UNI_V2_FACTORY,

    #[strum(ascii_case_insensitive, serialize = "UNI_V2_ROUTER")]
    UNI_V2_ROUTER,

    #[strum(ascii_case_insensitive, serialize = "UNI_V3_FACTORY")]
    UNI_V3_FACTORY,
}

impl Into<String> for ContractType {
    fn into(self) -> String {
        return self.to_string();
    }
}

#[cfg(test)]
mod test_dex {
    use super::DexExchange;
    #[test]
    fn should_str_into_dex_enum() {
        let dex: DexExchange = "MUTE_SWITCH".try_into().unwrap();
        assert_eq!(dex, DexExchange::MUTE_SWITCH);
    }

    #[test]
    fn should_str_split() {
        let dex: Vec<&str> = "MUTE_SWITCH,PANCAKE".split(',').collect();
        println!("{:?}", dex);
    }
}
