use crate::symbol::{KeywordArg, KeywordPunctuated};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, LitStr, Path, Token, Type};

#[derive(Default)]
pub struct DelegateArgs {
    pub contract: Option<Type>,
}

impl Parse for DelegateArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::contract) {
                if args.contract.is_some() {
                    return Err(
                        input.error("expected only single `contract` argument for delegate")
                    );
                }
                let contract = input.parse::<KeywordArg<kw::contract, Type>>()?.value;
                args.contract = Some(contract);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(input.error("unexpected token for delegate"));
            }
        }
        if args.contract.is_none() {
            return Err(input.error("delegate `contract` argument is missing"));
        }
        Ok(args)
    }
}

#[derive(Default)]
pub struct ChainArgs {
    pub http: Vec<LitStr>,
    pub ws: Vec<LitStr>,
    pub flashbots: Vec<LitStr>,
    pub poll: Option<ChainPollArgs>,
    pub cancel: Option<ChainCancelArgs>,
    pub contracts: Vec<ChainContractArgs>,
    pub delegates: Vec<ChainDelegateArgs>,
}

impl Parse for ChainArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::http) {
                if !args.http.is_empty() {
                    return Err(input.error("expected only single `http` argument for chain"));
                }
                let http = input.parse::<KeywordPunctuated<kw::http, LitStr>>()?.value;
                args.http = http.into_iter().collect();
            } else if lookahead.peek(kw::ws) {
                if !args.ws.is_empty() {
                    return Err(input.error("expected only single `ws` argument for chain"));
                }
                let ws = input.parse::<KeywordPunctuated<kw::ws, LitStr>>()?.value;
                args.ws = ws.into_iter().collect();
            } else if lookahead.peek(kw::flashbots) {
                if !args.flashbots.is_empty() {
                    return Err(input.error("expected only single `flashbots` argument for chain"));
                }
                let flashbots = input.parse::<KeywordPunctuated<kw::flashbots, LitStr>>()?.value;
                args.flashbots = flashbots.into_iter().collect();
            } else if lookahead.peek(kw::poll) {
                let _ = input.parse::<kw::poll>();
                let content;
                let _ = syn::parenthesized!(content in input);
                let poll_arg = content.parse::<ChainPollArgs>()?;
                args.poll = Some(poll_arg);
            } else if lookahead.peek(kw::cancel) {
                let _ = input.parse::<kw::cancel>();
                let content;
                let _ = syn::parenthesized!(content in input);
                let cancel_arg = content.parse::<ChainCancelArgs>()?;
                args.cancel = Some(cancel_arg);
            } else if lookahead.peek(kw::contract) {
                let _ = input.parse::<kw::contract>();
                let content;
                let _ = syn::parenthesized!(content in input);
                let contract_arg = content.parse::<ChainContractArgs>()?;
                args.contracts.push(contract_arg);
            } else if lookahead.peek(kw::delegate) {
                let _ = input.parse::<kw::delegate>();
                let content;
                let _ = syn::parenthesized!(content in input);
                let delegate_args = content.parse::<ChainDelegateArgs>()?;
                args.delegates.push(delegate_args);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(input.error("unexpected token for chain"));
            }
        }
        if args.http.is_empty() && args.ws.is_empty() {
            return Err(input.error("chain `http` and `ws` argument is missing"));
        }
        Ok(args)
    }
}

#[derive(Default)]
pub struct ChainPollArgs {
    pub retries: Option<LitInt>,
    pub interval: Option<LitInt>,
}

impl Parse for ChainPollArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::retries) {
                if args.retries.is_some() {
                    return Err(
                        input.error("expected only single `retries` argument for poll of chain")
                    );
                }
                let retries = input.parse::<KeywordArg<kw::retries, LitInt>>()?.value;
                args.retries = Some(retries);
            } else if lookahead.peek(kw::interval) {
                if args.interval.is_some() {
                    return Err(
                        input.error("expected only single `interval` argument for poll of chain")
                    );
                }
                let interval = input.parse::<KeywordArg<kw::interval, LitInt>>()?.value;
                args.interval = Some(interval);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(input.error("unexpected token for poll of chain"));
            }
        }
        Ok(args)
    }
}

#[derive(Default)]
pub struct ChainCancelArgs {
    pub wait: Option<LitInt>,
    pub gas_multiplier: Option<LitInt>,
    pub max_gas_power: Option<LitInt>,
    pub warning: Option<LitInt>,
    pub missing: Option<LitInt>,
}

impl Parse for ChainCancelArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::wait) {
                if args.wait.is_some() {
                    return Err(
                        input.error("expected only single `wait` argument for cancel of chain")
                    );
                }
                let wait = input.parse::<KeywordArg<kw::wait, LitInt>>()?.value;
                args.wait = Some(wait);
            } else if lookahead.peek(kw::gas_multiplier) {
                if args.gas_multiplier.is_some() {
                    return Err(input.error(
                        "expected only single `gas_multiplier` argument for cancel of chain",
                    ));
                }
                let gas_multiplier = input.parse::<KeywordArg<kw::gas_multiplier, LitInt>>()?.value;
                args.gas_multiplier = Some(gas_multiplier);
            } else if lookahead.peek(kw::max_gas_power) {
                if args.max_gas_power.is_some() {
                    return Err(input.error(
                        "expected only single `max_gas_power` argument for cancel of chain",
                    ));
                }
                let max_gas_power = input.parse::<KeywordArg<kw::max_gas_power, LitInt>>()?.value;
                args.max_gas_power = Some(max_gas_power);
            } else if lookahead.peek(kw::warning) {
                if args.warning.is_some() {
                    return Err(
                        input.error("expected only single `warning` argument for cancel of chain")
                    );
                }
                let warning = input.parse::<KeywordArg<kw::warning, LitInt>>()?.value;
                args.warning = Some(warning);
            } else if lookahead.peek(kw::missing) {
                if args.missing.is_some() {
                    return Err(
                        input.error("expected only single `missing` argument for cancel of chain")
                    );
                }
                let missing = input.parse::<KeywordArg<kw::missing, LitInt>>()?.value;
                args.missing = Some(missing);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(input.error("unexpected token for cancel of chain"));
            }
        }
        Ok(args)
    }
}

#[derive(Default, Clone)]
pub struct ChainContractArgs {
    pub ty: Option<Type>,
    pub variants: Vec<ContractVariantArgs>,
    pub instances: Vec<ContractInstanceArgs>,
    pub caches: Vec<ContractCacheArgs>,
    pub delegate: Option<Type>,
}

impl Parse for ChainContractArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![type]) {
                if args.ty.is_some() {
                    return Err(
                        input.error("expected only single `type` argument for contract of chain")
                    );
                }
                let ty = input.parse::<KeywordArg<Token![type], Type>>()?.value;
                args.ty = Some(ty);
            } else if lookahead.peek(kw::variant) {
                let _ = input.parse::<kw::variant>();
                let content;
                let _ = syn::parenthesized!(content in input);
                let variant_args = content.parse::<ContractVariantArgs>()?;
                args.variants.push(variant_args);
            } else if lookahead.peek(kw::instance) {
                let _ = input.parse::<kw::instance>();
                let content;
                let _ = syn::parenthesized!(content in input);
                let instance_args = content.parse::<ContractInstanceArgs>()?;
                args.instances.push(instance_args);
            } else if lookahead.peek(kw::cache) {
                if !args.caches.is_empty() {
                    return Err(
                        input.error("expected only single `cache` argument for contract of chain")
                    );
                }
                let caches =
                    input.parse::<KeywordPunctuated<kw::cache, ContractCacheArgs>>()?.value;
                args.caches = caches.into_iter().collect();
            } else if lookahead.peek(kw::delegate) {
                if args.delegate.is_some() {
                    return Err(input
                        .error("expected only single `delegate` argument for contract of chain"));
                }
                let delegate = input.parse::<KeywordArg<kw::delegate, Type>>()?.value;
                args.delegate = Some(delegate);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(input.error("unexpected token for contract of chain"));
            }
        }
        if args.ty.is_none() {
            return Err(input.error("chain contract `type` argument is missing"));
        }
        if args.instances.is_empty() {
            return Err(input.error("chain contract `instance` argument is missing"));
        }
        Ok(args)
    }
}

#[derive(Default, Clone)]
pub struct ContractVariantArgs {
    pub name: Option<Ident>,
    pub ty: Option<Type>,
    pub instances: Vec<ContractInstanceArgs>,
}

impl Parse for ContractVariantArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![type]) {
                if args.ty.is_some() {
                    return Err(
                        input.error("expected only single `type` argument for variant of contract")
                    );
                }
                let ty = input.parse::<KeywordArg<Token![type], Type>>()?.value;
                args.ty = Some(ty);
            } else if lookahead.peek(kw::instance) {
                let _ = input.parse::<kw::instance>();
                let content;
                let _ = syn::parenthesized!(content in input);
                let instance_args = content.parse::<ContractInstanceArgs>()?;
                args.instances.push(instance_args);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(input.error("unexpected token for variant of contract"));
            }
        }
        if args.ty.is_none() {
            return Err(input.error("contract variant `type` argument is missing"));
        }
        if args.instances.is_empty() {
            return Err(input.error("contract variant `instance` argument is missing"));
        }
        Ok(args)
    }
}

#[derive(Default, Clone)]
pub struct ContractInstanceArgs {
    pub name: Option<Ident>,
    pub address: Option<LitStr>,
}

impl Parse for ContractInstanceArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::name) {
                if args.name.is_some() {
                    return Err(input
                        .error("expected only single `name` argument for instance of contract"));
                }
                let name = input.parse::<KeywordArg<kw::name, Ident>>()?.value;
                args.name = Some(name);
            } else if lookahead.peek(kw::address) {
                if args.address.is_some() {
                    return Err(input.error(
                        "expected only single `address` argument for instance of contract",
                    ));
                }
                let address = input.parse::<KeywordArg<kw::address, LitStr>>()?.value;
                args.address = Some(address);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(input.error("unexpected token for instance of contract"));
            }
        }
        if args.name.is_none() {
            return Err(input.error("contract instance `name` argument is missing"));
        }
        if args.address.is_none() {
            return Err(input.error("contract instance `address` argument is missing"));
        }
        Ok(args)
    }
}

#[derive(Clone)]
pub struct ContractCacheArgs {
    pub name: Ident,
    pub input: Vec<CachePatTypeArgs>,
    pub output: Type,
}

impl Parse for ContractCacheArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let content;
        let _ = syn::parenthesized!(content in input);
        let punctuated = content.parse_terminated::<_, Token![,]>(CachePatTypeArgs::parse)?;
        let _ = input.parse::<Token![->]>()?;
        let output = input.parse()?;
        Ok(Self { name, input: punctuated.into_iter().collect(), output })
    }
}

#[derive(Clone)]
pub struct CachePatTypeArgs {
    pub pat: Ident,
    pub colon: Token![:],
    pub ty: Type,
}

impl Parse for CachePatTypeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pat = input.parse()?;
        let colon = input.parse()?;
        let ty = input.parse()?;
        Ok(Self { pat, colon, ty })
    }
}

impl ToTokens for CachePatTypeArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.pat.to_tokens(tokens);
        self.colon.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

#[derive(Default)]
pub struct ChainDelegateArgs {
    pub name: Option<Ident>,
    pub caller: Option<Type>,
    pub callees: Vec<Type>,
    pub output: Option<Type>,
}

impl Parse for ChainDelegateArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::name) {
                if args.name.is_some() {
                    return Err(
                        input.error("expected only single `name` argument for delegate of chain")
                    );
                }
                let name = input.parse::<KeywordArg<kw::name, Ident>>()?.value;
                args.name = Some(name);
            } else if lookahead.peek(kw::caller) {
                if args.caller.is_some() {
                    return Err(
                        input.error("expected only single `caller` argument for delegate of chain")
                    );
                }
                let caller = input.parse::<KeywordArg<kw::caller, Type>>()?.value;
                args.caller = Some(caller);
            } else if lookahead.peek(kw::callee) {
                if !args.callees.is_empty() {
                    return Err(
                        input.error("expected only single `callee` argument for delegate of chain")
                    );
                }
                let callees = input.parse::<KeywordPunctuated<kw::callee, Type>>()?.value;
                args.callees = callees.into_iter().collect();
            } else if lookahead.peek(kw::output) {
                if args.output.is_some() {
                    return Err(
                        input.error("expected only single `output` argument for delegate of chain")
                    );
                }
                let output = input.parse::<KeywordArg<kw::output, Type>>()?.value;
                args.output = Some(output);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(input.error("unexpected token for delegate of chain"));
            }
        }
        if args.name.is_none() {
            return Err(input.error("chain delegate `name` argument is missing"));
        }
        if args.caller.is_none() {
            return Err(input.error("chain delegate `caller` argument is missing"));
        }
        if args.callees.is_empty() {
            return Err(input.error("chain delegate `callee` argument is missing"));
        }
        if args.output.is_none() {
            return Err(input.error("chain delegate `output` argument is missing"));
        }
        Ok(args)
    }
}

#[derive(Default)]
pub struct ManageArgs {
    pub chains: Vec<Path>,
}

impl Parse for ManageArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(Ident) {
                let path = input.parse()?;
                args.chains.push(path);
            } else if lookahead.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else {
                return Err(input.error("unexpected token for manage"));
            }
        }
        Ok(args)
    }
}

mod kw {
    syn::custom_keyword!(contract);
    syn::custom_keyword!(http);
    syn::custom_keyword!(ws);
    syn::custom_keyword!(flashbots);
    syn::custom_keyword!(poll);
    syn::custom_keyword!(retries);
    syn::custom_keyword!(interval);
    syn::custom_keyword!(cancel);
    syn::custom_keyword!(wait);
    syn::custom_keyword!(gas_multiplier);
    syn::custom_keyword!(max_gas_power);
    syn::custom_keyword!(warning);
    syn::custom_keyword!(missing);
    syn::custom_keyword!(variant);
    syn::custom_keyword!(instance);
    syn::custom_keyword!(name);
    syn::custom_keyword!(address);
    syn::custom_keyword!(cache);
    syn::custom_keyword!(delegate);
    syn::custom_keyword!(caller);
    syn::custom_keyword!(callee);
    syn::custom_keyword!(output);
}
