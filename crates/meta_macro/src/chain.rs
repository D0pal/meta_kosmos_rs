use std::{env, fs};
use std::collections::{HashMap, HashSet};
use convert_case::{Case, Casing};
use quote::{format_ident, quote, ToTokens};
use syn::{FnArg, Ident, Item, ItemMod, ItemStruct, ItemTrait, LitInt, LitStr, TraitItem, Type};
use syn::parse::ParseStream;
use crate::symbol::{ident_and_generics, snake_ident, snake_ident_and_generics};
use crate::attr::{ChainArgs, ContractVariantArgs, ContractInstanceArgs, DelegateArgs, ManageArgs, ChainContractArgs, ContractCacheArgs};

const CONTRACT_SIGNER: &'static str = "ContractSigner";
const BUILDER: &'static str = "Builder";

pub fn contract_enum(enum_ident: &Ident, instances: &[ContractInstanceArgs], variants: &[ContractVariantArgs], error: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let mut enum_variants = vec![];
    let mut enum_addresses = vec![];
    let mut enum_from_strs = vec![];
    let mut enum_to_strs = vec![];
    let mut enum_methods = vec![];
    for instance in instances {
        let name = instance.name.as_ref().expect("should never panic");
        let address = instance.address.as_ref().expect("should never panic");
        let bytes = parse_hex(address);
        let variant = format_ident!("{}", name.to_string().to_case(Case::UpperCamel));
        let name_str = LitStr::new(&variant.to_string().to_lowercase(), name.span());
        let address_str = LitStr::new(&address.value().to_lowercase(), address.span());
        enum_variants.push(variant.clone());
        enum_addresses.push(quote!(Self::#variant => Address::from_slice(&vec![#(#bytes,)*])));
        enum_from_strs.push(quote!( #name_str | #address_str => Ok(Self::#variant)));
        enum_to_strs.push(quote!( Self::#variant => #name_str));
    }
    for variant in variants {
        let ty = variant.ty.as_ref().expect("should never panic");
        let (ty_ident, _) = snake_ident_and_generics(ty);
        let method_name = format_ident!("is_{}", ty_ident);
        let variant_match = variant_match(enum_ident, variant);
        enum_methods.push(quote!(
            pub fn #method_name(&self) -> bool {
                match self {
                    #variant_match => true,
                    _ => false,
                }
            }
        ))
    }
    quote!(
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
        #[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
        pub enum #enum_ident {
            #(#enum_variants,)*
        }

        impl #enum_ident {
            pub fn address(&self) -> Address {
                match self {
                    #(#enum_addresses,)*
                }
            }

            #(#enum_methods)*
        }

        impl std::str::FromStr for #enum_ident {
            type Err = #error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    #(#enum_from_strs,)*
                    _ => Err(#error::UnknownContract(s.to_string()))
                }
            }
        }

        impl Display for #enum_ident {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let s = match self {
                    #(#enum_to_strs,)*
                };
                write!(f, "{}", s)
            }
        }

        impl <T: Display> TryConvert<#enum_ident> for T {
            type Error = #error;

            fn try_convert(self) -> Result<#enum_ident, Self::Error> {
                #enum_ident::from_str(&self.to_string())
            }
        }
    )
}

pub fn derive_ident(ty: &Type, case: Case) -> Ident {
    match ty {
        Type::Path(path) => {
            let segments: Vec<String> = path.path.segments.iter()
                .map(|seg| seg.ident.to_string()).collect();
            format_ident!("{}", segments.join("_").to_case(case))
        },
        _ => panic!("derive_enum_ident unsupported type `{}`", quote!(#ty))
    }
}

pub fn variant_match(enum_ident: &Ident, variant: &ContractVariantArgs) -> proc_macro2::TokenStream {
    variant.instances.iter().enumerate()
        .map(|(i, instance)| {
            let name = instance.name.as_ref()
                .map(|n| format_ident!("{}", n.to_string().to_case(Case::UpperCamel)));
            if i == 0 {
                quote!(#enum_ident::#name)
            } else {
                quote!(| #enum_ident::#name)
            }
        }).fold(quote!(), |acc, q| quote!(#acc #q))
}


pub fn parse_chain_args(input: ParseStream) -> syn::Result<ChainArgs> {
    let content;
    let _ = syn::parenthesized!(content in input);
    content.parse::<ChainArgs>()
}

pub fn chain_manager(
    ident: Ident,
    all_chain_args: &HashMap<Vec<Ident>, ChainArgs>,
    cache_elements: &HashMap<Ident, Vec<Ident>>,
    contract_signer_ident: &Ident,
    contract_elements: &HashMap<Ident, (Ident, Type, bool, Vec<Ident>)>,
    provider: &proc_macro2::TokenStream,
    error: &proc_macro2::TokenStream
) -> proc_macro2::TokenStream {
    let struct_def = chains_managed_struct(&ident, all_chain_args.keys());
    let builder_struct_def = chains_manager_builder(&ident, all_chain_args.keys(), error);
    let mut methods = vec![];
    let mut provider_matches = vec![];
    let mut chain_idents = vec![];
    for idents in all_chain_args.keys() {
        let chain_variant = chain_ident(idents, Case::UpperCamel);
        let chain_ident = chain_ident(idents, Case::Snake);
        let path = idents_to_path(idents);
        methods.push(quote!(
            pub fn #chain_ident(&self) -> Result<Arc<#path<P>>, #error> {
                self.#chain_ident.clone().ok_or(#error::ChainNotAvailable(Chain::#chain_variant.to_string()))
            }
        ));
        provider_matches.push(quote!(
            Chain::#chain_variant => {
                self.#chain_ident.as_ref()
                    .map(|chain| chain.provider.clone())
                    .ok_or(#error::ChainNotAvailable(Chain::#chain_variant.to_string()))
            },
        ));
        chain_idents.push(chain_ident);
    }
    for (cache_ident, chain_variants) in cache_elements {
        let cache_ident_snake = snake_ident(cache_ident);
        let mut assigns: Vec<proc_macro2::TokenStream> = chain_variants.iter()
            .map(|chain_variant| {
                let chain_ident = snake_ident(chain_variant);
                quote!(#chain_ident: self.#chain_ident.clone())
            }).collect();
        if chain_variants.len() < chain_idents.len() {
            assigns.push(quote!(..Default::default()));
        }
        methods.push(quote!(
            pub fn #cache_ident_snake(&self) -> #cache_ident<P> {
                #cache_ident {
                    #(#assigns,)*
                }
            }
        ))
    }
    for (method_name, (contract_ident, ty, is_variant, chain_variants)) in contract_elements {
        let mut method_matches: Vec<proc_macro2::TokenStream> = chain_variants.iter().map(|chain_variant| {
            let chain_ident = snake_ident(chain_variant);
            quote!(
                Chain::#chain_variant => {
                    if let Some(chain) = self.#chain_ident.as_ref() {
                        Ok(chain.#method_name(name.try_convert()?))
                    } else {
                        Err(#error::ChainNotAvailable(Chain::#chain_variant.to_string()))
                    }
                }
            )
        }).collect();
        if chain_variants.len() < chain_idents.len() {
            let method_name = LitStr::new(&method_name.to_string(), method_name.span());
            method_matches.push(quote!(_ => Err(#error::MethodNotAvailable(#method_name.to_string()))));
        }
        let ty = if *is_variant {
            quote!(Option<Arc<#ty<#provider>>>)
        } else {
            quote!(Arc<#ty<#provider>>)
        };
        methods.push(quote!(
            pub fn #method_name(&self, chain: Chain, name: #contract_ident) -> Result<#ty, #error> {
                match chain {
                    #(#method_matches)*
                }
            }
        ))
    }
    let contract_signer_ident_snake = snake_ident(contract_signer_ident);
    let assigns: Vec<proc_macro2::TokenStream> = chain_idents.iter()
        .map(|ident| quote!(#ident: self.#ident.as_ref().map(|c| Arc::new(c.contract_signer(signer.clone())))))
        .collect();
    methods.push(quote!(
        pub fn #contract_signer_ident_snake<S: Signer + Clone + 'static>(&self, signer: S) -> #contract_signer_ident<P, S> {
            #contract_signer_ident {
                #(#assigns,)*
            }
        }
    ));
    quote!(
        #struct_def
        #builder_struct_def

        impl<P: JsonRpcClient + 'static> #ident<P> {
            #(#methods)*

            pub fn provider(&self, chain: Chain) -> Result<Arc<#provider>, #error> {
                match chain {
                    #(#provider_matches)*
                }
            }
        }
    )
}

fn chains_manager_builder<'a>(ident: &Ident, idents_iter: impl Iterator<Item=&'a Vec<Ident>>, error: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let builder_ident = format_ident!("{}", BUILDER);
    let idents_vec: Vec<Vec<Ident>> = replace_last(idents_iter, &builder_ident);
    let fields = chains_managed_fields(idents_vec.iter(), quote!(), false);
    let mut methods = vec![];
    let mut build_stmts = vec![];
    let mut chain_idents = vec![];
    for idents in &idents_vec {
        let chain_ident = chain_ident(idents, Case::Snake);
        let path = idents_to_path(&idents);
        methods.push(quote!(
            pub fn #chain_ident(mut self, builder: #path) -> Self {
                self.#chain_ident = Some(builder);
                self
            }
        ));
        build_stmts.push(quote!(
            let #chain_ident = if let Some(builder) = self.#chain_ident {
                Some(Arc::new(builder.build().await?))
            } else {
                None
            };
        ));
        chain_idents.push(chain_ident);
    }
    quote!(
        #[derive(Default)]
        pub struct #builder_ident {
            #(#fields,)*
        }

        impl #builder_ident {
            #(#methods)*

            pub async fn build(self) -> Result<#ident<MixProvider>, #error> {
                #(#build_stmts)*

                Ok(#ident { #(#chain_idents,)* })
            }
        }
    )
}

pub fn chains_contract_signer<'a>(idents_iter: impl Iterator<Item=&'a Vec<Ident>>, contract_args: HashMap<Ident, Vec<(Vec<Ident>, ChainContractArgs)>>, chains_count: usize, middleware: &proc_macro2::TokenStream, error: &proc_macro2::TokenStream) -> (proc_macro2::TokenStream, Ident, HashMap<Ident, (Ident, Type, bool, Vec<Ident>)>) {
    let ident = format_ident!("{}", CONTRACT_SIGNER);
    let idents_vec = replace_last(idents_iter, &ident);
    let fields = chains_managed_fields(idents_vec.iter(), quote!(<P, S>), true);
    let mut methods = vec![];
    let mut all_contract_elements: HashMap<Ident, (Ident, Type, bool, Vec<Ident>)> = HashMap::new();
    for (contract_ident, contract_args) in contract_args {
        let (method, contract_elements) = chains_contract_signer_method(contract_ident, contract_args, chains_count, middleware, error);
        all_contract_elements.extend(contract_elements);
        methods.push(method);
    }
    let token_stream = quote!(
        #[derive(Clone)]
        pub struct #ident<P: JsonRpcClient, S: Signer> {
            #(#fields,)*
        }

        impl <P: JsonRpcClient, S: Signer> #ident<P, S> {
            #(#methods)*
        }
    );
    (token_stream, ident, all_contract_elements)
}

fn replace_last<'a>(idents_iter: impl Iterator<Item=&'a Vec<Ident>>, ident: &Ident) -> Vec<Vec<Ident>> {
    idents_iter.map(|v| {
        let mut v = v.clone();
        v.remove(v.len() - 1);
        v.push(ident.clone());
        v
    }).collect()
}

fn chains_contract_signer_method(contract_ident: Ident, contract_args: Vec<(Vec<Ident>, ChainContractArgs)>, chains_count: usize, middleware: &proc_macro2::TokenStream, error: &proc_macro2::TokenStream) -> (proc_macro2::TokenStream, HashMap<Ident, (Ident, Type, bool, Vec<Ident>)>) {
    let method_name = format_ident!("{}", contract_ident.to_string().to_case(Case::Snake));
    let mut method_matches: Vec<proc_macro2::TokenStream> = vec![];
    let mut variant_method_matches: HashMap<Ident, Vec<proc_macro2::TokenStream>> = HashMap::new();
    let mut variant_method_rt_types: HashMap<Ident, Type> = HashMap::new();
    let mut variant_methods = vec![];
    let mut chain_variants = vec![];
    let mut variant_chain_variants: HashMap<Ident, Vec<Ident>> = HashMap::new();
    let mut contract_elements: HashMap<Ident, (Ident, Type, bool, Vec<Ident>)> = HashMap::new();
    let mut ty = None;
    for (idents, contract) in contract_args {
        ty = contract.ty;
        let chain_variant = chain_ident(&idents, Case::UpperCamel);
        let chain_ident = chain_ident(&idents, Case::Snake);
        method_matches.push(quote!(
            Chain::#chain_variant => {
                if let Some(signer) = &self.#chain_ident {
                    let name = name.try_convert()?;
                    Ok(signer.#method_name(name))
                } else {
                    Err(#error::ChainNotAvailable(Chain::#chain_variant.to_string()))
                }
            }
        ));
        for variant in contract.variants {
            let ty = variant.ty.expect("should never panic");
            let (ty_ident, _) = snake_ident_and_generics(&ty);
            let variant_name = variant.name.as_ref().unwrap_or(&ty_ident);
            let method_name = format_ident!("{method_name}_{variant_name}");
            let method_matches = variant_method_matches.entry(method_name.clone()).or_insert(vec![]);
            variant_method_rt_types.insert(method_name.clone(), ty);
            method_matches.push(quote!(
                Chain::#chain_variant => {
                    if let Some(signer) = &self.#chain_ident {
                        let name = name.try_convert()?;
                        Ok(signer.#method_name(name))
                    } else {
                        Err(#error::ChainNotAvailable(Chain::#chain_variant.to_string()))
                    }
                }
            ));
            variant_chain_variants.entry(method_name).or_default().push(chain_variant.clone());
        }
        chain_variants.push(chain_variant);
    }
    if method_matches.len() < chains_count {
        let method_name = LitStr::new(&method_name.to_string(), method_name.span());
        method_matches.push(quote!(
            _ => Err(ChainError::MethodNotAvailable(#method_name.to_string()))
        ));
    }
    for (method_name, mut method_matches) in variant_method_matches {
        if method_matches.len() < chains_count {
            let method_name = LitStr::new(&method_name.to_string(), method_name.span());
            method_matches.push(quote!(
                _ => Err(ChainError::MethodNotAvailable(#method_name.to_string()))
            ));
        }
        let ty = variant_method_rt_types.remove(&method_name).expect("should never panic");
        variant_methods.push(quote!(
            pub fn #method_name(&self, chain: Chain, name: #contract_ident) -> Result<Option<Arc<#ty<#middleware>>>, #error> {
                match chain {
                    #(#method_matches)*
                }
            }
        ));
        let chain_variants = variant_chain_variants.remove(&method_name).expect("should never panic");
        contract_elements.insert(method_name, (contract_ident.clone(), ty.clone(), true, chain_variants));
    }
    let token_stream = quote!(
        pub fn #method_name(&self, chain: Chain, name: #contract_ident) -> Result<Arc<#ty<#middleware>>, #error> {
            match chain {
                #(#method_matches)*
            }
        }

        #(#variant_methods)*
    );
    contract_elements.insert(method_name, (contract_ident, ty.expect("should never panic"), false, chain_variants));
    (token_stream, contract_elements)
}

pub fn chains_contract_caches(contract_args: HashMap<Ident, Vec<(Vec<Ident>, ChainContractArgs)>>, chains_count: usize, error: &proc_macro2::TokenStream) -> (proc_macro2::TokenStream, HashMap<Ident, Vec<Ident>>) {
    let mut contract_caches = vec![];
    let mut cache_idents = HashMap::new();
    for (contract_ident, contract_args) in contract_args {
        let (contract_cache, cache_element) = chains_contract_cache(contract_ident, contract_args, chains_count, error);
        contract_caches.push(contract_cache);
        if let Some((cache_ident, chain_variants)) = cache_element {
            cache_idents.insert(cache_ident, chain_variants);
        }
    }
    (
        quote!(
            #(#contract_caches)*
        ),
        cache_idents
    )
}

fn chains_contract_cache(contract_ident: Ident, contract_args: Vec<(Vec<Ident>, ChainContractArgs)>, chains_count: usize, error: &proc_macro2::TokenStream) -> (proc_macro2::TokenStream, Option<(Ident, Vec<Ident>)>) {
    let cache_ident = format_ident!("{}Cache", contract_ident);
    let cache_ident_snake = format_ident!("{}", cache_ident.to_string().to_case(Case::Snake));
    let struct_def = chains_managed_struct(&cache_ident, contract_args.iter().map(|(i, _)| i));
    let mut cache_method_matches: HashMap<Ident, Vec<proc_macro2::TokenStream>> = HashMap::new();
    let mut cache_args_map: HashMap<Ident, ContractCacheArgs> = HashMap::new();
    let mut cache_methods = vec![];
    let mut chain_variants = vec![];
    for (idents, contract) in contract_args {
        let chain_variant = chain_ident(&idents, Case::UpperCamel);
        let chain_ident = chain_ident(&idents, Case::Snake);
        for cache in contract.caches {
            let method_name = cache.name.clone();
            let pats = cache.input.iter().map(|i| &i.pat);
            let methods_matches = cache_method_matches.entry(method_name.clone()).or_insert(vec![]);
            methods_matches.push(quote!(
                Chain::#chain_variant => {
                    if let Some(chain) = &self.#chain_ident {
                        let name = name.try_convert()?;
                        chain.#cache_ident_snake().#method_name(name #(,#pats)*).await
                    } else {
                        Err(#error::ChainNotAvailable(Chain::#chain_variant.to_string()))
                    }
                }
            ));
            cache_args_map.insert(method_name, cache);
        }
        chain_variants.push(chain_variant);
    }
    for (method_name, mut variant_match) in cache_method_matches {
        if variant_match.len() < chains_count {
            let method_name = LitStr::new(&method_name.to_string(), method_name.span());
            variant_match.push(quote!(
                _ => Err(ChainError::MethodNotAvailable(#method_name.to_string()))
            ));
        }
        let cache = cache_args_map.remove(&method_name).expect("should never panic");
        let input = cache.input;
        let output = cache.output;
        cache_methods.push(quote!(
            pub async fn #method_name(&self, chain: Chain, name: #contract_ident #(,#input)*) -> Result<#output, #error> {
                match chain {
                    #(#variant_match,)*
                }
            }
        ))
    }
    if cache_methods.is_empty() {
        (quote!(), None)
    } else {
        (
            quote!(
                #struct_def

                impl<P: JsonRpcClient + 'static> #cache_ident<P> {
                    #(#cache_methods)*
                }
            ),
            Some((cache_ident, chain_variants))
        )
    }
}

fn chains_managed_struct<'a>(ident: &Ident, idents_iter: impl Iterator<Item=&'a Vec<Ident>>) -> proc_macro2::TokenStream {
    let fields = chains_managed_fields(idents_iter, quote!(<P>), true);
    quote!(
        #[derive(Clone, Default)]
        pub struct #ident<P: JsonRpcClient> {
            #(#fields,)*
        }
    )
}

pub fn chains_managed_fields<'a>(idents_iter: impl Iterator<Item=&'a Vec<Ident>>, generics: proc_macro2::TokenStream, is_arc: bool) -> Vec<proc_macro2::TokenStream> {
    let mut fields = vec![];
    for idents in idents_iter {
        let field_name = chain_ident(idents, Case::Snake);
        let chain_path = idents_to_path(idents);
        let field_type = if is_arc {
            quote!(Option<Arc<#chain_path #generics>>)
        } else {
            quote!(Option<#chain_path #generics>)
        };
        fields.push(quote!(pub #field_name: #field_type))
    }
    fields
}

pub fn chain_enum(all_chain_args: &HashMap<Vec<Ident>, ChainArgs>) -> proc_macro2::TokenStream {
    let variants: Vec<Ident> =  all_chain_args.keys()
        .map(|idents| chain_ident(idents, Case::UpperCamel)).collect();
    quote!(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
        #[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
        pub enum Chain {
            #(#variants,)*
        }

        impl Display for Chain {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    )
}

pub fn chains_contract_enums(contract_args: HashMap<Ident, Vec<(Vec<Ident>, ChainContractArgs)>>, chains_count: usize, error: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let mut chains_enums = vec![];
    for (enum_ident, contract_args) in contract_args {
        chains_enums.push(chains_contract_enum(enum_ident, contract_args, chains_count, error));
    }
    quote!(
        #(#chains_enums)*
    )
}

pub fn extract_contract_args(all_chain_args: &HashMap<Vec<Ident>, ChainArgs>) -> HashMap<Ident, Vec<(Vec<Ident>, ChainContractArgs)>> {
    let mut contract_args: HashMap<Ident, Vec<(Vec<Ident>, ChainContractArgs)>> = HashMap::new();
    for (idents, args) in all_chain_args {
        for contract in &args.contracts {
            let ty = contract.ty.as_ref().expect("should never panic");
            let enum_ident = derive_ident(ty, Case::UpperCamel);
            let v = contract_args.entry(enum_ident).or_insert(vec![]);
            v.push((idents.clone(), contract.clone()));
        }
    }
    contract_args
}

fn chains_contract_enum(enum_ident: Ident, contract_args: Vec<(Vec<Ident>, ChainContractArgs)>, chains_count: usize, error: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let mut enum_variants = HashSet::new();
    let mut enum_from_strs = vec![];
    let mut enum_to_strs = vec![];
    let mut is_methods_match: HashMap<Ident, Vec<proc_macro2::TokenStream>> = HashMap::new();
    let mut enum_methods = vec![];
    for (idents, contract) in contract_args {
        let chain_variant = chain_ident(&idents, Case::UpperCamel);
        let mut instances = contract.instances;
        for variant in &contract.variants {
            instances.extend(variant.instances.clone());
            let ty = variant.ty.as_ref().expect("should never panic");
            let (ty_ident, _) = snake_ident_and_generics(ty);
            let method_name = format_ident!("is_{}", ty_ident);
            let variant_match = variant_match(&enum_ident, variant);
            let methods_matches = is_methods_match.entry(method_name).or_insert(vec![]);
            methods_matches.push(quote!(
                    Chain::#chain_variant => {
                        match self {
                            #variant_match => true,
                            _ => false,
                        }
                    }
                ));
        }
        for instance in instances {
            let name = instance.name.as_ref().expect("should never panic");
            let variant = format_ident!("{}", name.to_string().to_case(Case::UpperCamel));
            let name_str = LitStr::new(&variant.to_string().to_lowercase(), name.span());
            enum_variants.insert(variant.clone());
            enum_from_strs.push(quote!( #name_str => Ok(Self::#variant)));
            enum_to_strs.push(quote!( Self::#variant => #name_str));
        }
    }
    for (method_name, mut variant_match) in is_methods_match {
        if variant_match.len() < chains_count {
            variant_match.push(quote!(_ => false));
        }
        enum_methods.push(quote!(
            pub fn #method_name(&self, chain: Chain) -> bool {
                match chain {
                    #(#variant_match,)*
                }
            }
        ))
    }
    let enum_variants: Vec<Ident> = enum_variants.into_iter().collect();
    quote!(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
        #[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
        pub enum #enum_ident {
            #(#enum_variants,)*
        }

        impl #enum_ident {
            #(#enum_methods)*
        }

        impl std::str::FromStr for #enum_ident {
            type Err = #error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_lowercase().as_str() {
                    #(#enum_from_strs,)*
                    _ => Err(#error::UnknownContract(s.to_string()))
                }
            }
        }

        impl Display for #enum_ident {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let s = match self {
                    #(#enum_to_strs,)*
                };
                write!(f, "{}", s)
            }
        }

        impl <T: Display> TryConvert<#enum_ident> for T {
            type Error = #error;

            fn try_convert(self) -> Result<#enum_ident, Self::Error> {
                #enum_ident::from_str(&self.to_string())
            }
        }
    )
}

fn chain_ident(idents: &Vec<Ident>, case: Case) -> Ident {
    let len = idents.len();
    let v = idents.iter().enumerate()
        .map_while(|(i, ident)| if i == len - 1 { None } else { Some(ident.to_string()) })
        .collect::<Vec<String>>()
        .join("_").to_case(case);
    format_ident!("{}", v)
}

fn idents_to_path(idents: &Vec<Ident>) -> proc_macro2::TokenStream {
    let first = &idents[0];
    idents[1..].iter().fold(quote!(#first), |acc, i| quote!(#acc::#i))
}

fn parse_hex(address: &LitStr) -> Vec<LitInt> {
    let address_value = address.value();
    let data = if address_value.starts_with("0x") {
        &address_value[2..]
    } else {
        &address_value
    };
    match hex::decode(data) {
        Ok(v) => {
            v.into_iter().map(|x| LitInt::new(&x.to_string(), address.span())).collect()
        }
        Err(err) => {
            panic!("invalid hex address: {}, err: {:?}", address_value, err)
        }
    }
}

