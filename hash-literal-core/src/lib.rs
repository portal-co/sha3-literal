use std::mem::take;

use proc_macro2::Span;
use syn::parse::discouraged::Speculative;
use syn::parse::{ParseStream, Parser};
use syn::{ExprArray, LitByte, LitByteStr, LitInt, Token};
use syn::{LitStr, Macro, parse::Parse};

pub use digest::Digest;
pub use proc_macro2;
pub use quote;
pub use syn;

/// A macro to generate literal and hex_literal proc macros for a specific hash algorithm.
/// 
/// # Example
/// ```ignore
/// hash_literal_core::literals!(sha3 => sha3::Sha3_256);
/// hash_literal_core::literals!(sha3_512 => sha3::Sha3_512);
/// ```
/// 
/// Note: The consuming crate must have `paste` as a dependency.
#[macro_export]
macro_rules! literals {
    ($a:ident => $t:ty) => {
        ::paste::paste! {
            #[proc_macro]
            pub fn [< $a _literal>](a: proc_macro::TokenStream) -> proc_macro::TokenStream {
                let a = $crate::syn::parse_macro_input!(a as $crate::HashLiteral).emit::<$t>();
                $crate::quote::quote! {#a}.into()
            }
            #[proc_macro]
            pub fn [< $a _hex_literal>](a: proc_macro::TokenStream) -> proc_macro::TokenStream {
                let a = $crate::syn::parse_macro_input!(a as $crate::HashLiteral).emit_hex::<$t>();
                $crate::quote::quote! {#a}.into()
            }
        }
    };
}

pub struct HashLiteral {
    pub lit: (Vec<u8>, Span),
    pub cb: Option<Macro>,
}

impl Parse for HashLiteral {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let a = parse_bytes(input)?;
        let mut this = Self { lit: a, cb: None };
        if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            this.cb = Some(input.parse()?);
        }
        Ok(this)
    }
}

pub fn parse_bytes(input: ParseStream) -> syn::Result<(Vec<u8>, Span)> {
    use quote::quote;
    
    let fork = input.fork();
    if let Ok(l) = fork.parse::<LitStr>() {
        input.advance_to(&fork);
        return Ok((l.value().into_bytes(), l.span()));
    }
    let fork = input.fork();
    if let Ok(l) = fork.parse::<LitByteStr>() {
        input.advance_to(&fork);
        return Ok((l.value(), l.span()));
    }
    let fork = input.fork();
    if let Ok(l) = fork.parse::<LitByte>() {
        input.advance_to(&fork);
        return Ok((vec![l.value()], l.span()));
    }
    let fork = input.fork();
    if let Ok(l) = fork.parse::<LitInt>() {
        if let Ok(v) = l.base10_parse() {
            input.advance_to(&fork);
            return Ok((vec![v], l.span()));
        }
    }
    let fork = input.fork();
    if let Ok(a) = fork.parse::<ExprArray>() {
        if let Ok(x) = a
            .elems
            .iter()
            .map(|a| parse_bytes.parse2(quote! {#a}))
            .collect::<syn::Result<Vec<_>>>()
        {
            input.advance_to(&fork);
            let (x, y) = x.into_iter().collect::<(Vec<_>, Vec<_>)>();
            return Ok((
                x.into_iter().flatten().collect(),
                y.into_iter()
                    .map(Some)
                    .reduce(|a, b| a?.join(b?))
                    .flatten()
                    .unwrap_or_else(|| Span::call_site()),
            ));
        }
    }
    let fork = input.fork();
    if let Ok(a) = fork.parse::<Macro>() {
        if let Some(s) = a.path.get_ident().map(|i| i.to_string()) {
            match s.as_str() {
                "include_bytes" | "include_str" => {
                    let l: LitStr = a.parse_body()?;
                    input.advance_to(&fork);
                    let r = match std::fs::read(l.value()) {
                        Ok(r) => r,
                        Err(e) => return Err(syn::Error::new(l.span(), e)),
                    };
                    return Ok((r, l.span()));
                }
                "include" => {
                    let l: LitStr = a.parse_body()?;
                    input.advance_to(&fork);
                    let r = match std::fs::read_to_string(l.value()) {
                        Ok(r) => r,
                        Err(e) => return Err(syn::Error::new(l.span(), e)),
                    };
                    let r: HashLiteral = syn::parse_str(&r)?;
                    return Ok((r.lit.0, l.span()));
                }
                _ => {}
            }
        }
    }
    Err(input.error("expected a hashable thing"))
}

impl HashLiteral {
    pub fn emit<D: Digest>(&self) -> proc_macro2::TokenStream {
        use quote::{quote, quote_spanned};
        
        let s = D::digest(&self.lit.0);
        let a = quote_spanned! { self.lit.1 =>
            [#(#s),*]
        };
        match self.cb.clone() {
            None => a,
            Some(mut c) => {
                let t = take(&mut c.tokens);
                c.tokens = quote! {#a #t};
                quote! {#c}
            }
        }
    }
    
    pub fn emit_hex<D: Digest>(&self) -> proc_macro2::TokenStream {
        use quote::{quote, quote_spanned};
        
        let s = D::digest(&self.lit.0);
        let s = hex::encode(s);
        let a = quote_spanned! { self.lit.1 =>
            #s
        };
        match self.cb.clone() {
            None => a,
            Some(mut c) => {
                let t = take(&mut c.tokens);
                c.tokens = quote! {#a #t};
                quote! {#c}
            }
        }
    }
}
