use crate::{builder::*, AsSnake};

use syn::{parse::Result, Index};

struct QueryableRow;

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

struct QueryableFields;

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

        let model_with_id = input.parsed_struct.model_name_with_id();

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
