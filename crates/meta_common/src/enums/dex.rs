#[derive(Default, Clone, Debug, PartialEq, strum_macros::Display, strum_macros::EnumString)]
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