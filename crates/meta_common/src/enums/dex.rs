use serde::Deserialize;
use strum::{AsRefStr, Display, EnumCount, EnumIter, EnumString, EnumVariantNames};

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

    #[strum(ascii_case_insensitive, serialize = "AGNI")]
    AGNI,

    #[strum(ascii_case_insensitive, serialize = "IZUMI")]
    IZUMI,

    #[strum(ascii_case_insensitive, serialize = "SUSHISWAP")]
    SUSHISWAP,

    #[strum(ascii_case_insensitive, serialize = "SyncSwap")]
    SyncSwap,

    #[strum(ascii_case_insensitive, serialize = "MuteSwitch")]
    MuteSwitch,

    #[strum(ascii_case_insensitive, serialize = "UniswapV2")]
    UniswapV2,

    #[strum(ascii_case_insensitive, serialize = "UniswapV3")]
    UniswapV3,
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
    #[strum(ascii_case_insensitive, serialize = "UniV2Factory")]
    UniV2Factory,

    #[strum(ascii_case_insensitive, serialize = "UniV2RouterV2")]
    UniV2RouterV2,

    #[strum(ascii_case_insensitive, serialize = "UniV3Factory")]
    UniV3Factory,

    #[strum(ascii_case_insensitive, serialize = "UniV3SwapRouterV2")]
    UniV3SwapRouterV2,

    #[strum(ascii_case_insensitive, serialize = "UniV3Nft")]
    UniV3Nft,

    #[strum(ascii_case_insensitive, serialize = "UniV3QuoterV2")]
    UniV3QuoterV2,

    #[strum(ascii_case_insensitive, serialize = "IzumiLiquidityManager")]
    IzumiLiquidityManager,

    #[strum(ascii_case_insensitive, serialize = "IzumiLimitOrderManager")]
    IzumiLimitOrderManager,

    #[strum(ascii_case_insensitive, serialize = "IzumiSwapRouter")]
    IzumiSwapRouter,
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
        let dex: DexExchange = "MuteSwitch".try_into().unwrap();
        assert_eq!(dex, DexExchange::MuteSwitch);
    }

    #[test]
    fn should_str_split() {
        let dex: Vec<&str> = "MuteSwitch,PANCAKE".split(',').collect();
        println!("{:?}", dex);
    }
}
