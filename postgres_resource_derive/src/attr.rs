use proc_macro2::{Span, TokenTree::Literal};
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token, Attribute, Ident, LitStr,
    Meta::*,
    MetaNameValue,
    Error,
    Lit,
};

#[derive(Debug)]
pub struct Attrs {
    db_conn: Option<LitStr>,
    table: Option<LitStr>,
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let mut db_conn: Option<LitStr> = None;
        let mut table: Option<LitStr> = None;

        for attr in &attrs {
            if attr.path.is_ident("table") {
                table = Self::parse_attr(attr, "table")?;
            }
            if attr.path.is_ident("env_var") {
                db_conn = Self::parse_attr(attr, "env_var")?;
            }
        }

        println!("{:#?}", table);

        Ok(Attrs { db_conn, table })
    }
}

impl Attrs {
    fn parse_attr(attr: &Attribute, expected: &str) -> Result<Option<LitStr>> {
        match attr.parse_meta()? {
            NameValue(MetaNameValue { lit: Lit::Str(lit_str), .. }) => {
                lit_str.parse().map(Some)
            }
            _ => {
                let error_span = attr.bracket_token.span;
                let message = &format!("expected #[{} = \"...\"]", expected);
                Err(syn::Error::new(error_span, message))
            }
        }
    }
}

