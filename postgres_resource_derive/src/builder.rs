use crate::{attr::*, IdentExt, camel_to_snake, r#struct::*};

use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token, Attribute, Ident, LitStr,
    Meta::*,
    MetaNameValue,
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

pub struct ModelWithId<B>(pub B);

impl<'i, B: Builder<'i>> Builder<'i> for ModelWithId<B> {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let model_name_with_id = input.parsed_struct.ident.append("WithId");
        let fields = self.0.build(input)?;

        Ok(quote! {
            pub struct #model_name_with_id {
                #fields
            }
        })
    }
}

pub struct Fields;

impl<'i> Builder<'i> for Fields {
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

pub struct Model<B>(pub B);

impl<'i, B: Builder<'i>> Builder<'i> for Model<B> {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let model_name = &input.parsed_struct.ident;
        let fields = self.0.build(input)?;

        Ok(quote! {
            pub struct #model_name {
                #fields
            }
        })
    }
}

pub struct InnerFields;

impl<'i> Builder<'i> for InnerFields {
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
        let model = camel_to_snake(&input.parsed_struct.ident.to_string()[..]) + "s";
        quote!(crate::schema::#model)
    }
}
