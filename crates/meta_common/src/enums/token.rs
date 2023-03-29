
#[derive(Default, Clone, Debug, PartialEq, strum_macros::Display, strum_macros::EnumString)]
pub enum Token {
    #[default]
    None,

    #[strum(ascii_case_insensitive, serialize = "WBNB")]
    WBNB,

    #[strum(ascii_case_insensitive, serialize = "WETH")]
    WETH,

    #[strum(ascii_case_insensitive, serialize = "USDC")]
    USDC,

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