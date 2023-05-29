use serde::Deserialize;

#[derive(
    Default, Clone, Debug, PartialEq, strum_macros::Display, strum_macros::EnumString, Deserialize,
)]
pub enum Dex {
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

impl Into<String> for Dex {
    fn into(self) -> String {
        return self.to_string();
    }
}

// impl From<&str> for Dex {
//     fn from(input: &str) -> Dex {
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
}

impl Into<String> for ContractType {
    fn into(self) -> String {
        return self.to_string();
    }
}

#[cfg(test)]
mod test_dex {
    use super::Dex;
    #[test]
    fn should_str_into_dex_enum() {
        let dex: Dex = "MUTE_SWITCH".try_into().unwrap();
        assert_eq!(dex, Dex::MUTE_SWITCH);
    }

    #[test]
    fn should_str_split() {
        let dex: Vec<&str> = "MUTE_SWITCH,PANCAKE".split(',').collect();
        println!("{:?}", dex);
    }
}
