use crate::{attr::*, IdentExt, camel_to_snake, r#struct::*, AsSnake};

use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token, Attribute, Ident, LitStr,
    Meta::*,
    MetaNameValue,
    Index,
};

pub struct Input {
    pub parsed_struct: Struct,
}

pub trait Builder<'i> {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream>;
}

pub struct InferredTableMacro;

impl<'i> Builder<'i> for InferredTableMacro {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let model_name = camel_to_snake(&input.parsed_struct.ident.to_string()[..]) + "s";
        let literal = LitStr::new(&model_name, Span::call_site());
        Ok(quote!(#[table_name = #literal]))
    }
}

pub struct CustomTableMacro<B>(pub B);

impl<'i, B: Builder<'i>> Builder<'i> for CustomTableMacro<B> {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        self.0.build(input)
    }
}

pub struct ModelWithId;

impl<'i> Builder<'i> for ModelWithId {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let model_name_with_id = input.parsed_struct.ident.append("WithId");
        let fields = ModelWithIdFields.build(input)?;

        Ok(quote! {
            pub struct #model_name_with_id {
                #fields
            }
        })
    }
}

pub struct ModelWithIdFields;

impl<'i> Builder<'i> for ModelWithIdFields {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let mut fields = Vec::new();
        fields.push(quote!(pub id: i32));

        let model_name = &input.parsed_struct.ident;
        fields.push(quote!(pub inner: #model_name));

        input.parsed_struct.fields.iter().for_each(|field| {
            if field.fk() {
                let ty = field.ty();
                let name = &field.name;
                fields.push(quote!(pub #name: #ty));
            }
        });

        Ok(quote!(#(#fields,)*))
    }
}

pub struct Model;

impl<'i> Builder<'i> for Model {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let model_name = &input.parsed_struct.ident;
        let fields = ModelFields.build(input)?;

        Ok(quote! {
            pub struct #model_name {
                #fields
            }
        })
    }
}

pub struct ModelFields;

impl<'i> Builder<'i> for ModelFields {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let mut fields = Vec::new();
        input.parsed_struct.fields.iter().for_each(|field| {
            if !field.fk() {
                let ty = field.ty();
                let name = &field.name;
                fields.push(quote!(pub #name: #ty));
            }
        });

        Ok(quote!(#(#fields,)*))
    }
}

pub struct Table;

impl<'i> Builder<'i> for Table {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let model = input.parsed_struct.as_snake_plural();
        Ok(quote!(crate::schema::#model))
    }
}

pub struct QueryableRow;

impl<'i> Builder<'i> for QueryableRow {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let mut fields = Vec::new();
        fields.push(quote!(i32));

        input.parsed_struct.fields.iter().for_each(|field| {
            let ty = field.ty();
            fields.push(quote!(#ty));
        });

        Ok(quote!(type Row = (#(#fields,)*);))
    }
}

pub struct QueryableFields;

impl<'i> Builder<'i> for QueryableFields {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let mut fields = Vec::new();
        let mut inner_fields = Vec::new();

        let model_name = input.parsed_struct.inner_model_name();

        let mut index = 0;

        // Push id
        let idx = Index::from(index);
        fields.push(quote!(id: row.#idx));

        input.parsed_struct.fields.iter().enumerate().for_each(|(i, field)| {
            let field_name = &field.name;
            if !field.fk() {
                index = i + 1;
                let idx = Index::from(index);
                inner_fields.push(quote!(#field_name: row.#idx));
            }
        });

        let generated_inner_fields = quote!({ #(#inner_fields,)* });

        // Push inner fields
        fields.push(quote!(inner: #model_name #generated_inner_fields));

        // Push remaining fields
        input.parsed_struct.fields.iter().for_each(|field| {
            if field.fk() {
                index = index + 1;
                let name = &field.name;
                let idx = Index::from(index);
                fields.push(quote!(#name: row.#idx));
            }
        });

        Ok(quote!(#(#fields,)*))
    }
}

pub struct Queryable;

impl<'i> Builder<'i> for Queryable {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let table = input.parsed_struct.as_snake_plural();
        let fields = QueryableFields.build(input)?;
        let row = QueryableRow.build(input)?;

        let model_with_id = input.parsed_struct.model_name();

        Ok(quote! {
            impl diesel::Queryable<#table::SqlType, diesel::pg::Pg> for #model_with_id {
                #row
                fn build(row: Self::Row) -> Self {
                    #model_with_id {
                        #fields
                    }
                }
            }
        })
    }
}


