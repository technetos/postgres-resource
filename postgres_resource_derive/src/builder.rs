use crate::{camel_to_snake, r#struct::*, AsSnake};

use proc_macro2::Span;
use syn::{parse::Result, LitStr};

pub struct Input {
    pub parsed_struct: Struct,
}

pub trait Builder<'i> {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream>;
}

struct InferredTableMacro;

impl<'i> Builder<'i> for InferredTableMacro {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let model_name = camel_to_snake(&input.parsed_struct.ident.to_string()[..]) + "s";
        let literal = LitStr::new(&model_name, Span::call_site());
        Ok(quote!(#[table_name = #literal]))
    }
}

pub struct TableMacro;

impl<'i> Builder<'i> for TableMacro {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        if let Some(ref table) = input.parsed_struct.attrs.table {
            Ok(quote!(#[table_name = #table]))
        } else {
            Ok(InferredTableMacro.build(input)?)
        }
    }
}

pub struct Schema;

impl<'i> Builder<'i> for Schema {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let model = input.parsed_struct.as_snake_plural();
        Ok(quote!(crate::schema::#model))
    }
}
