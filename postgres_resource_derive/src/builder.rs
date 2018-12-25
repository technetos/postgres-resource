use crate::{r#struct::*, IdentExt};

use proc_macro2::Span;
use syn::{Ident, parse::Result, LitStr};
use heck::CamelCase;

pub struct Input {
    pub parsed_struct: Struct,
}

pub trait Builder<'i> {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream>;
}

struct InferredTableMacro;

impl<'i> Builder<'i> for InferredTableMacro {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let model_name = input.parsed_struct.ident.append("s").snake_case().to_string();
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
        let model = input.parsed_struct.ident.append("s").snake_case();
        Ok(quote!(crate::schema::#model))
    }
}

struct DefaultDatabaseConnection;

impl<'i> Builder<'i> for DefaultDatabaseConnection {
    fn build(self, _: &'i Input) -> Result<proc_macro2::TokenStream> {
        Ok(quote!(&self.connection()))
    }
}
        
pub struct DatabaseConnection;

impl<'i> Builder<'i> for DatabaseConnection {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        if let Some(ref env_var) = input.parsed_struct.attrs.db_conn {
            Ok(quote!(&self.connection_string(#env_var)))
        } else {
            DefaultDatabaseConnection.build(input)
        }
    }
}

pub struct BelongsToMacro;

impl<'i> Builder<'i> for BelongsToMacro {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let mut macros = Vec::new();
        input.parsed_struct.fields
            .iter()
            .filter(|field| field.fk())
            .for_each(|field| {
                let mut name = field.name.to_string();
                name.truncate(name.len() - 2);
                let model_name = Ident::new(&name.to_camel_case(), Span::call_site());
                macros.push(quote!(#[belongs_to(#model_name)]));
            });
        Ok(quote!(#(#macros)*))
    }
}
