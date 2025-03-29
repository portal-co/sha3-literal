use std::mem::take;

use proc_macro2::Span;
use quote::{ToTokens, quote, quote_spanned};
use sha3::Digest;
use syn::parse::discouraged::Speculative;
use syn::parse::{ParseStream, Parser};
use syn::{ExprArray, LitByte, LitByteStr, LitInt, Token, bracketed};
use syn::{ExprMacro, LitStr, Macro, parse::Parse, parse_macro_input, token::Token};

#[proc_macro]
pub fn sha3_literal(a: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let a = parse_macro_input!(a as Sha3Literal);
    quote! {#a}.into()
}
struct Sha3Literal {
    lit: (Vec<u8>, Span),
    cb: Option<Macro>,
}
impl Parse for Sha3Literal {
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
fn parse_bytes(input: ParseStream) -> syn::Result<(Vec<u8>, Span)> {
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
    return Err(input.error("expected a hashable thing"));
}
impl ToTokens for Sha3Literal {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let s = sha3::Sha3_256::digest(&self.lit.0);
        let a = quote_spanned! { self.lit.1 =>
            [#(#s),*]
        };
        tokens.extend(match self.cb.clone() {
            None => a,
            Some(mut c) => {
                let t = take(&mut c.tokens);
                c.tokens = quote! {#a #t};
                quote! {#c}
            }
        });
    }
}
