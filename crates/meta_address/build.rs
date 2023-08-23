use anyhow::Context;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde_json::Value;
use std::{
    env,
    fs::{read_to_string, write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

pub fn ident(s: impl AsRef<str>) -> Ident {
    let s = s.as_ref().trim();

    // Parse the ident from a str. If the string is a Rust keyword, stick an
    // underscore in front.
    syn::parse_str::<Ident>(s)
        .unwrap_or_else(|_| Ident::new(format!("_{s}").as_str(), Span::call_site()))
}

pub fn write_generated_file(content: TokenStream, out_file: &str) -> anyhow::Result<()> {
    let out_dir = env::var_os("OUT_DIR").context("failed to get OUT_DIR env var")?;
    let path = Path::new(&out_dir).join(out_file);
    let code = content.to_string();

    write(&path, code)?;

    // Try to format the output for debugging purposes.
    // Doesn't matter if rustfmt is unavailable.
    let _ = Command::new("rustfmt").arg(path).output();

    Ok(())
}

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=./static/token_address.json");
    let out_dir = env::var("OUT_DIR").unwrap();
    println!("OUT_DIR {:?}", out_dir);

    write_generated_file(build_enums()?, "token_enum.rs")
}

fn build_enums() -> anyhow::Result<TokenStream> {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let path: PathBuf = PathBuf::from_str(&cargo_manifest_dir).unwrap().join("static/token_address.json");

    let token_json_file = read_to_string(path).unwrap();
    let v: Value = serde_json::from_str(&token_json_file).unwrap();

    let enum_variants =
        v.as_object().unwrap().iter().map(|(key, _)| key.to_string()).collect::<Vec<String>>();
    println!("enum_variants {:?}", enum_variants);

    let prop_name_variants =
        enum_variants.iter().map(|name| ident(name.replace('.', "_"))).collect::<Vec<_>>();
    Ok(quote! {
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

            #(
                #[strum(ascii_case_insensitive)]
                #prop_name_variants,
            )*
        }

        impl Into<String> for Token {
            fn into(self) -> String {
                return self.to_string();
            }
        }

        
    })
}