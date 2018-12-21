use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident,
};

use crate::{attr::*, camel_to_snake, field::*, AsSnake, IdentExt};

#[derive(Debug)]
pub struct Struct {
    pub attrs: Attrs,
    pub ident: syn::Ident,
    pub fields: Punctuated<Field, Token![,]>,
}

impl Parse for Struct {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attrs::parse)?;
        let content;
        let _: Token![struct] = input.parse()?;
        let ident = input.parse()?;
        let _ = braced!(content in input);
        let fields = content.parse_terminated(Field::parse)?;
        Ok(Struct { attrs, ident, fields })
    }
}

impl Struct {
    pub fn model_name_with_id(&self) -> Ident {
        self.ident.append("WithId")
    }

    pub fn inner_model_name(&self) -> Ident {
        self.ident.clone()
    }

    pub fn controller_name(&self) -> Ident {
        self.ident.append("Controller")
    }
}

impl AsSnake for Struct {
    fn as_snake(&self) -> Ident {
        let snake = camel_to_snake(&self.ident.to_string()[..]);
        Ident::new(&snake[..], Span::call_site())
    }

    fn as_snake_plural(&self) -> Ident {
        let plural_snake = camel_to_snake(&(self.ident.to_string() + "s")[..]);
        Ident::new(&plural_snake[..], Span::call_site())
    }
}
