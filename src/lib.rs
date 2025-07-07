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
#[proc_macro]
pub fn sha3_hex_literal(a: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let a = parse_macro_input!(a as Sha3Literal);
    let a = Sha3HexLiteral(a);
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
                    let r: Sha3Literal = syn::parse_str(&r)?;
                    return Ok((r.lit.0, l.span()));
                }
                "sha3_literal" => {
                    let (b, c) = a.parse_body_with(parse_bytes)?;
                    input.advance_to(&fork);
                    let b = sha3::Sha3_256::digest(b).into_iter().collect();
                    return Ok((b, c));
                }
                "sha3_hex_literal" => {
                    let (b, c) = a.parse_body_with(parse_bytes)?;
                    input.advance_to(&fork);
                    let b = sha3::Sha3_256::digest(b);
                    let b = hex::encode(b);
                    let b = b.into_bytes();
                    return Ok((b, c));
                }
                _ => {}
            }
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
struct Sha3HexLiteral(Sha3Literal);
impl ToTokens for Sha3HexLiteral {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let s = sha3::Sha3_256::digest(&self.0.lit.0);
        let s = hex::encode(s);
        let a = quote_spanned! { self.lit.1 =>
            #s
        };
        tokens.extend(match self.0.cb.clone() {
            None => a,
            Some(mut c) => {
                let t = take(&mut c.tokens);
                c.tokens = quote! {#a #t};
                quote! {#c}
            }
        });
    }
}
