use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    Ident,
};

use crate::{attr::*, field::*, IdentExt};

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
