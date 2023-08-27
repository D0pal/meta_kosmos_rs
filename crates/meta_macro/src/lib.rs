mod attr;
mod chain;
mod symbol;

use crate::attr::{
    ChainArgs, ChainContractArgs, ContractCacheArgs, ContractInstanceArgs, ContractVariantArgs,
    DelegateArgs, ManageArgs,
};
use crate::chain::{
    chain_enum, chains_contract_caches, chains_contract_enums, chains_contract_signer,
    contract_enum, derive_ident, extract_contract_args, parse_chain_args, variant_match,
    chain_manager
};
use crate::symbol::{ident_and_generics, snake_ident, snake_ident_and_generics};
use convert_case::{Case, Casing};
use quote::{format_ident, quote, ToTokens};
use std::collections::{HashMap, HashSet};
use std::{env, fs};
use syn::parse::ParseStream;
use syn::{FnArg, Ident, Item, ItemMod, ItemStruct, ItemTrait, LitInt, LitStr, TraitItem, Type};

const BUILDER: &'static str = "Builder";
const CONTRACT_SIGNER: &'static str = "ContractSigner";

#[proc_macro_attribute]
pub fn impl_contract_code(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_struct = syn::parse_macro_input!(item as ItemStruct);
    let struct_ident = &item_struct.ident;
    quote!(
        #item_struct
        impl ContractCode for #struct_ident {
            fn get_byte_code(&self) -> Option<&String> {
                self.byte_code.as_ref()
            }
            fn get_code_hash(&self) -> Option<&String> {
                self.code_hash.as_ref()
            }
        }

    )
    .into()
}

#[proc_macro_attribute]
pub fn delegate(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(attr as DelegateArgs);
    let item_trait = syn::parse_macro_input!(item as ItemTrait);
    let trait_ident = &item_trait.ident;
    let contract = args.contract.expect("should never panic");
    let (contract_ident, _) = ident_and_generics(&contract);
    let mut impl_fn_vec = vec![];
    for item in &item_trait.items {
        match item {
            TraitItem::Method(method) => {
                let sig_ident = &method.sig.ident;
                let inputs = &method.sig.inputs;
                let output = &method.sig.output;
                let mut call_args = vec![];
                for arg in inputs {
                    match arg {
                        FnArg::Receiver(_) => (),
                        FnArg::Typed(typed) => call_args.push(typed.pat.clone()),
                    }
                }
                let impl_fn = quote!(
                    fn #sig_ident(#inputs) #output {
                        let call = self.contract.#sig_ident(#(#call_args, )*);
                        self.delegator.call(self.address, call.tx.data().expect("should never panic").clone())
                    }
                );
                impl_fn_vec.push(impl_fn);
            }
            _ => (),
        }
    }
    quote!(
        #item_trait

        #[derive(Debug)]
        pub struct #contract_ident<M: Middleware, D: Delegator<M, T>, T> {
            address: ethers::prelude::Address,
            delegator: D,
            contract: #contract<M>,
            _p: PhantomData<T>,
        }

        impl <M: Middleware, D: Delegator<M, T>, T> #contract_ident<M, D, T> {

            pub fn new(address: ethers::prelude::Address, delegator: D) -> Self {
                let contract = #contract::new(address, delegator.middleware());
                Self { address, delegator, contract, _p: PhantomData }
            }

            pub fn address(&self) -> ethers::prelude::Address {
                self.address
            }

            pub fn delegator(&self) -> &D {
                &self.delegator
            }

            pub fn contract(&self) -> &#contract<M> {
                &self.contract
            }
        }

        impl <M: Middleware, D: Delegator<M, T>, T> Clone for #contract_ident<M, D, T> {
            fn clone(&self) -> Self {
                Self {
                    address: self.address.clone(),
                    delegator: self.delegator.clone(),
                    contract: self.contract.clone(),
                    _p: PhantomData
                }
            }
        }

        impl <M: Middleware, D: Delegator<M, T>, T> #trait_ident<M, T> for #contract_ident<M, D, T> {
            #(#impl_fn_vec)*
        }
    ).into()
}

#[proc_macro_attribute]
pub fn chain(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(attr as ChainArgs);
    let item_struct = syn::parse_macro_input!(item as ItemStruct);
    let provider = quote!(Provider<P>);
    let middleware = quote!(TransactionSubscriptionMiddleware<NonceManagerMiddleware<SignerMiddleware<Arc<#provider>, S>>>);
    let error = quote!(ChainError);
    let ident = item_struct.ident;
    let mut fields = vec![quote!(pub provider: Arc<#provider>)];
    let mut methods = vec![];
    let builder_ident = format_ident!("{}", BUILDER);
    let mut builder_fields = vec![];
    let mut builder_init_fields = vec![];
    let mut builder_methods = vec![];
    let mut build_assign_fields = vec![];
    let mut init_provider = quote!();
    let contract_signer_ident = format_ident!("{}", CONTRACT_SIGNER);
    let mut contract_signer_fields = vec![quote!(pub middleware: Arc<#middleware>)];
    let mut contract_signer_init_fields = vec![quote!(middleware)];
    let mut contract_signer_methods = vec![quote!(
        pub fn middleware(&self) -> Arc<#middleware> {
            self.middleware.clone()
        }
        pub fn signer(&self) -> &S {
            self.middleware.inner().inner().signer()
        }
        pub fn signer_address(&self) -> Address {
            self.middleware.inner().inner().signer().address()
        }
    )];
    let mut contract_enums = vec![];
    let mut contract_enum_aliases = vec![];
    let mut contract_caches = vec![];
    let mut contract_delegates = vec![];
    let http = &args.http;
    if !http.is_empty() {
        builder_fields.push(quote!(rpc_http_urls: Vec<String>));
        builder_init_fields.push(quote!(rpc_http_urls: strings(&vec![#(#http,)*])));
        builder_methods.push(quote!(
            pub fn http_rpc(mut self, http: &[&str]) -> Self {
                self.rpc_http_urls = strings(http);
                self
            }
        ));
        init_provider = quote!(MixProvider::http(urls(self.rpc_http_urls)?));
    }
    let ws = &args.ws;
    if !ws.is_empty() {
        builder_fields.push(quote!(rpc_ws_conns: Vec<ConnectionDetails>));
        builder_init_fields.push(quote!(rpc_ws_conns: ws_conns(&vec![#(#ws,)*])));
        builder_methods.push(quote!(
            pub fn ws_rpc(mut self, ws: &[&str]) -> Self {
                self.rpc_ws_conns = ws_conns(ws);
                self
            }
        ));
        init_provider =
            quote!(MixProvider::new(urls(self.rpc_http_urls)?, self.rpc_ws_conns).await?);
    }
    let flashbots = &args.flashbots;
    if !flashbots.is_empty() {
        fields.push(quote!(pub flashbots_builder_urls: Vec<Url>));
        builder_fields.push(quote!(flashbots_builder_urls: Vec<String>));
        builder_init_fields.push(quote!(flashbots_builder_urls: strings(&vec![#(#flashbots,)*])));
        builder_methods.push(quote!(
            pub fn flashbots_builder(mut self, flashbots: &[&str]) -> Self {
                self.flashbots_builder_urls = strings(flashbots);
                self
            }
        ));
        build_assign_fields
            .push(quote!(flashbots_builder_urls: urls(self.flashbots_builder_urls)?));
        methods.push(quote!(
            pub async fn bundle_manager<TS: Signer, SS: Signer>(&self, tx_signer: TS, searcher_signer: SS) -> Result<BundleManager<Arc<#provider>, TS, SS>, #error> {
                BundleManager::new(self.provider.clone(), tx_signer, searcher_signer, self.flashbots_builder_urls.clone()).await
            }
        ));
    }

    let poll_retries = args
        .poll
        .as_ref()
        .map(|p| p.retries.clone())
        .and_then(|o| o)
        .unwrap_or(LitInt::new("12", ident.span()));
    fields.push(quote!(pub poll_retries: usize));
    builder_fields.push(quote!(poll_retries: usize));
    builder_init_fields.push(quote!(poll_retries: #poll_retries));
    builder_methods.push(quote!(
        pub fn poll_retries(mut self, retries: usize) -> Self {
            self.poll_retries = retries;
            self
        }
    ));
    build_assign_fields.push(quote!(poll_retries: self.poll_retries));

    let poll_interval = args
        .poll
        .as_ref()
        .map(|p| p.interval.clone())
        .and_then(|o| o)
        .unwrap_or(LitInt::new("10", ident.span()));
    fields.push(quote!(pub poll_interval: std::time::Duration));
    builder_fields.push(quote!(poll_interval: std::time::Duration));
    builder_init_fields.push(quote!(poll_interval: std::time::Duration::from_secs(#poll_interval)));
    builder_methods.push(quote!(
        pub fn poll_interval(mut self, interval: std::time::Duration) -> Self {
            self.poll_interval = interval;
            self
        }
    ));
    build_assign_fields.push(quote!(poll_interval: self.poll_interval));

    let cancel_wait_secs = args
        .cancel
        .as_ref()
        .map(|c| c.wait.clone())
        .and_then(|o| o)
        .unwrap_or(LitInt::new("60", ident.span()));
    fields.push(quote!(pub cancel_wait_secs: u64));
    builder_fields.push(quote!(cancel_wait_secs: u64));
    builder_init_fields.push(quote!(cancel_wait_secs: #cancel_wait_secs));
    builder_methods.push(quote!(
        pub fn cancel_wait_secs(mut self, wait_secs: u64) -> Self {
            self.cancel_wait_secs = wait_secs;
            self
        }
    ));
    build_assign_fields.push(quote!(cancel_wait_secs: self.cancel_wait_secs));

    let cancel_gas_multiplier = args
        .cancel
        .as_ref()
        .map(|c| c.gas_multiplier.clone())
        .and_then(|o| o)
        .unwrap_or(LitInt::new("120", ident.span()));
    fields.push(quote!(pub cancel_gas_multiplier: u8));
    builder_fields.push(quote!(cancel_gas_multiplier: u8));
    builder_init_fields.push(quote!(cancel_gas_multiplier: #cancel_gas_multiplier));
    builder_methods.push(quote!(
        pub fn cancel_gas_multiplier(mut self, gas_multiplier: u8) -> Self {
            self.cancel_gas_multiplier = gas_multiplier;
            self
        }
    ));
    build_assign_fields.push(quote!(cancel_gas_multiplier: self.cancel_gas_multiplier));

    let cancel_max_gas_power = args
        .cancel
        .as_ref()
        .map(|c| c.max_gas_power.clone())
        .and_then(|o| o)
        .unwrap_or(LitInt::new("6", ident.span()));
    fields.push(quote!(pub cancel_max_gas_power: u8));
    builder_fields.push(quote!(cancel_max_gas_power: u8));
    builder_init_fields.push(quote!(cancel_max_gas_power: #cancel_max_gas_power));
    builder_methods.push(quote!(
        pub fn cancel_max_gas_power(mut self, max_gas_power: u8) -> Self {
            self.cancel_max_gas_power = max_gas_power;
            self
        }
    ));
    build_assign_fields.push(quote!(cancel_max_gas_power: self.cancel_max_gas_power));

    let cancel_warning = args
        .cancel
        .as_ref()
        .map(|c| c.warning.clone())
        .and_then(|o| o)
        .unwrap_or(LitInt::new("5", ident.span()));
    fields.push(quote!(pub cancel_warning: u8));
    builder_fields.push(quote!(cancel_warning: u8));
    builder_init_fields.push(quote!(cancel_warning: #cancel_warning));
    builder_methods.push(quote!(
        pub fn cancel_warning(mut self, warning: u8) -> Self {
            self.cancel_warning = warning;
            self
        }
    ));
    build_assign_fields.push(quote!(cancel_warning: self.cancel_warning));

    let cancel_missing = args
        .cancel
        .as_ref()
        .map(|c| c.missing.clone())
        .and_then(|o| o)
        .unwrap_or(LitInt::new("3", ident.span()));
    fields.push(quote!(pub cancel_missing: u8));
    builder_fields.push(quote!(cancel_missing: u8));
    builder_init_fields.push(quote!(cancel_missing: #cancel_missing));
    builder_methods.push(quote!(
        pub fn cancel_missing(mut self, missing: u8) -> Self {
            self.cancel_missing = missing;
            self
        }
    ));
    build_assign_fields.push(quote!(cancel_missing: self.cancel_missing));

    for contract in args.contracts {
        let ty = contract.ty.as_ref().expect("should never panic");
        let enum_ident = derive_ident(&ty, Case::UpperCamel);
        let method_ident = derive_ident(&ty, Case::Snake);
        let mut instances = contract.instances;
        for variant in &contract.variants {
            instances.extend(variant.instances.clone());
            let ty = variant.ty.as_ref().expect("should never panic");
            let (ty_ident, _) = snake_ident_and_generics(ty);
            let variant_name = variant.name.as_ref().unwrap_or(&ty_ident);
            let method_ident = format_ident!("{method_ident}_{variant_name}");
            let map_ident = format_ident!("{method_ident}_map");
            let variant_match = variant_match(&enum_ident, variant);
            fields.push(quote!(#map_ident: DashMap<#enum_ident, Arc<#ty<#provider>>>));
            build_assign_fields.push(quote!(#map_ident: DashMap::new()));
            methods.push(quote!(
                pub fn #method_ident(&self, name: #enum_ident) -> Option<Arc<#ty<#provider>>> {
                    match &name {
                        #variant_match => {
                            if !self.#map_ident.contains_key(&name) {
                                let value = Arc::new(#ty::new(name.address(), self.provider()));
                                self.#map_ident.insert(name, value);
                            }
                            Some(self.#map_ident.get(&name).expect("should never panic").clone())
                        },
                        _ => None,
                    }
                }
            ));
            contract_signer_fields.push(quote!(
                #map_ident: DashMap<#enum_ident, Arc<#ty<#middleware>>>
            ));
            contract_signer_init_fields.push(quote!(#map_ident: DashMap::new()));
            contract_signer_methods.push(quote!(
                pub fn #method_ident(&self, name: #enum_ident) -> Option<Arc<#ty<#middleware>>> {
                    match &name {
                        #variant_match => {
                            if !self.#map_ident.contains_key(&name) {
                                let value = Arc::new(#ty::new(name.address(), self.middleware()));
                                self.#map_ident.insert(name, value);
                            }
                            Some(self.#map_ident.get(&name).expect("should never panic").clone())
                        },
                        _ => None,
                    }
                }
            ));
        }
        contract_enums.push(contract_enum(&enum_ident, &instances, &contract.variants, &error));
        let map_ident = format_ident!("{method_ident}_map");
        fields.push(quote!(#map_ident: DashMap<#enum_ident, Arc<#ty<#provider>>>));
        build_assign_fields.push(quote!(#map_ident: DashMap::new()));
        methods.push(quote!(
            pub fn #method_ident(&self, name: #enum_ident) -> Arc<#ty<#provider>> {
                if !self.#map_ident.contains_key(&name) {
                    let value = Arc::new(#ty::new(name.address(), self.provider()));
                    self.#map_ident.insert(name, value);
                }
                self.#map_ident.get(&name).expect("should never panic").clone()
            }
        ));
        contract_signer_fields.push(quote!(
            #map_ident: DashMap<#enum_ident, Arc<#ty<#middleware>>>
        ));
        contract_signer_init_fields.push(quote!(#map_ident: DashMap::new()));
        contract_signer_methods.push(quote!(
            pub fn #method_ident(&self, name: #enum_ident) -> Arc<#ty<#middleware>> {
                if !self.#map_ident.contains_key(&name) {
                    let value = Arc::new(#ty::new(name.address(), self.middleware()));
                    self.#map_ident.insert(name, value);
                }
                self.#map_ident.get(&name).expect("should never panic").clone()
            }
        ));
        if !contract.caches.is_empty() {
            let method_name = format_ident!("{method_ident}_cache");
            let cache_ident =
                format_ident!("{}", method_name.to_string().to_case(Case::UpperCamel));
            let mut cache_fields = vec![quote!(provider: Arc<#provider>)];
            let mut cache_init_fields = vec![quote!(provider: provider.clone())];
            let mut cache_methods = vec![];
            for cache in &contract.caches {
                let name = &cache.name;
                let input = &cache.input;
                let output = &cache.output;
                let map_name = format_ident!("{name}_map");
                let (map_ty, key_stmt) = if input.is_empty() {
                    (quote!(DashMap<#enum_ident, #output>), quote!(let key = name;))
                } else {
                    let fmt = input
                        .iter()
                        .map(|i| format!("{{{}:?}}", i.pat))
                        .fold(String::new(), |acc, p| acc + &p);
                    let fmt_args = LitStr::new(&fmt, name.span());
                    (
                        quote!(DashMap<u64, #output>),
                        quote!(let key = hash_key(&format!(#fmt_args));),
                    )
                };
                let pats = input.iter().map(|i| &i.pat);
                cache_fields.push(quote!(#map_name: #map_ty));
                cache_init_fields.push(quote!(#map_name: DashMap::new()));
                cache_methods.push(quote!(
                    pub async fn #name(&self, name: CommonErc20 #(,#input)*) -> Result<#output, #error> {
                        #key_stmt
                        if !self.#map_name.contains_key(&key) {
                            let contract = #ty::new(name.address(), self.provider.clone());
                            let value = contract.#name(#(#pats,)*).call().await?;
                            self.#map_name.insert(key, value);
                        }
                        Ok(self.#map_name.get(&key).expect("should never panic").clone())
                    }
                ));
            }
            fields.push(quote!(pub #method_name: Arc<#cache_ident<P>>));
            build_assign_fields
                .push(quote!(#method_name: Arc::new(#cache_ident { #(#cache_init_fields,)* })));
            methods.push(quote!(
                pub fn #method_name(&self) -> Arc<#cache_ident<P>> {
                    self.#method_name.clone()
                }
            ));
            contract_caches.push(quote!(
                pub struct CommonErc20Cache<P: JsonRpcClient> {
                    #(#cache_fields,)*
                }

                impl <P: JsonRpcClient> CommonErc20Cache<P> {
                    #(#cache_methods)*
                }
            ));
        }
        if let Some(delegate) = &contract.delegate {
            let alias_ident = derive_ident(delegate, Case::UpperCamel);
            let method_ident = derive_ident(delegate, Case::Snake);
            contract_enum_aliases.push(quote!(pub type #alias_ident = #enum_ident;));
            contract_signer_methods.push(quote!(
                pub fn #method_ident<D: delegate::Delegator<#middleware, T>, T>(&self, name: #enum_ident, caller: D) -> #delegate<#middleware, D, T> {
                    #delegate::new(name.address(), caller)
                }
            ))
        }
    }

    for delegate in args.delegates {
        let name_ident = delegate.name.as_ref().expect("should never panic");
        let ty_ident = format_ident!("{}", name_ident.to_string().to_case(Case::UpperCamel));
        let caller = delegate.caller.as_ref().expect("should never panic");
        let output = delegate.output.as_ref().expect("should never panic");
        let mut fields = vec![quote!(caller: #caller<#middleware>)];
        let mut init_fields = vec![quote!(caller)];
        let mut methods = vec![];
        let caller_enum = derive_ident(&caller, Case::UpperCamel);
        for callee in &delegate.callees {
            let callee_enum = derive_ident(callee, Case::UpperCamel);
            let (method_ident, _) = snake_ident_and_generics(callee);
            let map_ident = format_ident!("{}_map", method_ident);
            fields.push(quote!(#map_ident: DashMap<#callee_enum, #callee<#middleware, #caller<#middleware>, #output>>));
            init_fields.push(quote!(#map_ident: DashMap::new()));
            methods.push(quote!(
                pub fn #method_ident(&self, name: #callee_enum) -> #callee<#middleware, #caller<#middleware>, #output> {
                    if !self.#map_ident.contains_key(&name) {
                        let value = #callee::new(name.address(), self.caller.clone());
                        self.#map_ident.insert(name, value);
                    }
                    self.#map_ident.get(&name).expect("should never panic").clone()
                }
            ))
        }
        contract_signer_methods.push(quote!(
            pub fn #name_ident(&self, caller: #caller_enum) -> #ty_ident<P, S> {
                let caller = #caller::new(caller.address(), self.middleware());
                #ty_ident { #(#init_fields,)* }
            }
        ));
        contract_delegates.push(quote!(
            pub struct #ty_ident<P: JsonRpcClient, S: Signer> { #(#fields,)* }

            impl <P: JsonRpcClient, S: Signer> #ty_ident<P, S> {
                #(#methods)*
            }
        ));
    }

    quote!(

        #(#contract_enums)*

        #(#contract_enum_aliases)*

        pub struct #ident<P: JsonRpcClient> {
            #(#fields,)*
        }

        impl <P: JsonRpcClient + 'static> #ident<P> {

            pub fn provider(&self) -> Arc<#provider> {
                return self.provider.clone()
            }

            pub fn contract_signer<S: Signer + Clone + 'static>(&self, signer: S) -> #contract_signer_ident<P, S> {
                let signer_address = signer.address();
                let cancel_tx = start_auto_cancel_task(
                    self.provider.clone(),
                    signer.clone(),
                    self.cancel_wait_secs,
                    self.cancel_gas_multiplier,
                    self.cancel_max_gas_power,
                    self.cancel_warning,
                    self.cancel_missing,
                );
                let middleware = Arc::new(TransactionSubscriptionMiddleware::new(
                    NonceManagerMiddleware::new(
                        SignerMiddleware::new(self.provider.clone(), signer),
                        signer_address
                    ),
                    vec![cancel_tx],
                    self.poll_retries,
                    self.poll_interval,
                ));
                #contract_signer_ident { #(#contract_signer_init_fields,)* }
            }

            #(#methods)*
        }

        pub struct #builder_ident {
            #(#builder_fields,)*
        }

        impl #builder_ident {
            pub fn new() -> Self {
                Self {
                    #(#builder_init_fields,)*
                }
            }

            #(#builder_methods)*

            pub async fn build(self) -> Result<#ident<MixProvider>, #error> {
                let provider = #init_provider;
                let provider = Arc::new(Provider::new(provider));
                Ok(#ident {
                    provider: provider.clone(),
                    #(#build_assign_fields,)*
                })
            }
        }

        pub struct #contract_signer_ident<P: JsonRpcClient, S: Signer> {
            #(#contract_signer_fields,)*
        }

        impl <P: JsonRpcClient, S: Signer> #contract_signer_ident<P, S> {
            #(#contract_signer_methods)*
        }

        #(#contract_caches)*

        #(#contract_delegates)*
    ).into()
}

#[proc_macro_attribute]
pub fn manage(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(attr as ManageArgs);
    let item_struct = syn::parse_macro_input!(item as ItemStruct);
    let ident = item_struct.ident;
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let mut all_chain_args: HashMap<Vec<Ident>, ChainArgs> = HashMap::new();
    for chain in args.chains {
        let path =
            &chain.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<String>>().join("/");
        let full_path = format!("{}/src/chain/{}.rs", manifest_dir, path);
        let s = fs::read_to_string(&full_path).expect(&format!("read module file: {}", full_path));
        let s = format!("mod adhoc {{ {} }}", s);
        let item = s.parse::<proc_macro::TokenStream>().expect(&format!("parse {}", full_path));
        let item_mod = syn::parse_macro_input!(item as ItemMod);
        if let Some((_, items)) = item_mod.content {
            for item in items {
                match item {
                    Item::Struct(item_struct) => {
                        for attr in item_struct.attrs {
                            if attr.path.to_token_stream().to_string() == "chain" {
                                let attr: proc_macro::TokenStream = attr.tokens.into();
                                let chain_args =
                                    syn::parse_macro_input!(attr with parse_chain_args);
                                let mut idents = chain
                                    .segments
                                    .iter()
                                    .map(|s| s.ident.clone())
                                    .collect::<Vec<Ident>>();
                                idents.push(item_struct.ident.clone());
                                all_chain_args.insert(idents, chain_args);
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    let error = quote!(ChainError);
    let provider = quote!(Provider<P>);
    let middleware = quote!(TransactionSubscriptionMiddleware<NonceManagerMiddleware<SignerMiddleware<Arc<#provider>, S>>>);
    let chains_count = all_chain_args.len();
    let chain_enum = chain_enum(&all_chain_args);
    let contract_args = extract_contract_args(&all_chain_args);
    let contract_enums = chains_contract_enums(contract_args.clone(), chains_count, &error);
    let (contract_caches, cache_elements) =
        chains_contract_caches(contract_args.clone(), chains_count, &error);
    let (contract_signer, contract_signer_ident, contract_elements) = chains_contract_signer(
        all_chain_args.keys(),
        contract_args,
        chains_count,
        &middleware,
        &error,
    );
    let chain_manager = chain_manager(
        ident,
        &all_chain_args,
        &cache_elements,
        &contract_signer_ident,
        &contract_elements,
        &provider,
        &error,
    );
    quote!(
        #chain_enum
        #contract_enums
        #contract_caches
        #contract_signer
        #chain_manager
    )
    .into()
}
