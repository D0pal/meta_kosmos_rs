use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Token, Type, Ident, PathArguments, GenericArgument};

pub struct KeywordArg<K, T> {
    pub key: K,
    pub value: T,
}

impl<K: Parse, T: Parse> Parse for KeywordArg<K, T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse::<K>()?;
        let _ = input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self { key, value })
    }
}

pub struct KeywordPunctuated<K, V> {
    pub value: Punctuated<V, Token![,]>,
    _p: std::marker::PhantomData<K>,
}

impl <K: Parse, V: Parse> Parse for KeywordPunctuated<K, V> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _ = input.parse::<K>()?;
        let content;
        let _ = syn::parenthesized!(content in input);
        let value = content.parse_terminated(V::parse)?;
        Ok(Self { value, _p: std::marker::PhantomData})
    }
}

pub fn compile_error(span: Span, message: &str) -> TokenStream {
    Error::new(span, message).to_compile_error()
}

pub fn snake_ident<T: ToTokens>(t: &T) -> Ident {
    format_ident!("{}", snake_case(t))
}

pub fn snake_case<T: ToTokens>(t: &T) -> String {
    quote!(#t).to_string().to_case(Case::Snake)
}

pub fn in_types(types: &Vec<Type>, ty: &Type) -> bool {
    types.contains(ty)
}

pub fn ident_generics_and_exclude(ty: &Type, excludes: &[&str]) -> Result<(Ident, Vec<Type>), Error> {
    let (ident, generics) = ident_and_generics(ty);
    if excludes.contains(&ident.to_string().as_str()) {
        if generics.is_empty() {
            Err(Error::new(ident.span(), format!("expect generic type for {}", ident)))
        } else {
            Ok(ident_and_generics(&generics[0]))
        }
    } else {
        Ok((ident, generics))
    }
}

pub fn snake_ident_and_generics(ty: &Type) -> (Ident, Vec<Type>) {
    let (ident, generics) = ident_and_generics(ty);
    (snake_ident(&ident), generics)
}

pub fn ident_and_generics(ty: &Type) -> (Ident, Vec<Type>) {
    match ty {
        Type::Path(ty) => {
            let seg = ty.path.segments.last().expect("at least one segment in type path");
            let seg_ident = seg.ident.clone();
            if let PathArguments::AngleBracketed(args) = &seg.arguments {
                let generics: Vec<Type> = args.args.iter()
                    .map(|arg| {
                        match arg {
                            GenericArgument::Type(ty) => Some(ty.clone()),
                            _ => None,
                        }
                    })
                    .filter(|option| option.is_some())
                    .map(|option| option.unwrap())
                    .collect();
                (seg_ident, generics)
            } else {
                (seg_ident, vec![])
            }
        }
        Type::Reference(ty_ref) => {
            ident_and_generics(&ty_ref.elem)
        }
        Type::Tuple(ty_tuple) => {
            if ty_tuple.elems.is_empty() {
                (format_ident!("void"), vec![])
            } else {
                (format_ident!("tuple"), ty_tuple.elems.clone().into_iter().collect())
            }
        }
         _ => panic!("ident_and_generics unsupported type `{}`", quote!(#ty))
    }
}
