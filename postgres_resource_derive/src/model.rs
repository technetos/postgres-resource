use crate::builder::{Input, Builder};

use syn::parse::Result; 

pub struct ModelWithId;

impl<'i> Builder<'i> for ModelWithId {
    fn build(self, input: &'i Input) -> Result<proc_macro2::TokenStream> {
        let model_name_with_id = input.parsed_struct.model_name_with_id();
        let fields = ModelWithIdFields.build(input)?;

        Ok(quote! {
            pub struct #model_name_with_id {
                #fields
            }
        })
    }
}

struct ModelWithIdFields;

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

struct ModelFields;

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

