#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;

use proc_macro2::Span;
use syn::Ident;

fn concat(string: &str, suffix: &str) -> syn::Ident {
    Ident::new(&format!("{}{}", string, suffix), Span::call_site())
}

#[proc_macro]
pub fn resource_controller(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let string = &input.to_string()[..];
    let model = Ident::new(string, Span::call_site());

    let controller = concat(string, "Controller");
    let model_with_id = concat(string, "WithId");

    let gen = quote_spanned! {Span::call_site()=>
        pub struct #controller;

        impl ResourceWithId for #controller {
            type ModelWithId = #model_with_id;
        }

        impl Resource for #controller {
            type Model = #model;
        }

        impl ResourceTable for #controller {
            type DBTable = table;
        }

        impl ResourceSql for #controller {
            type SQLType = SqlType;
        }

        use crate::db::establish_connection as connection;

        impl ResourceController for #controller {
            fn create(&self, model: &Self::Model) -> Result<Self::ModelWithId, Error> {
                Ok(insert_into(table)
                   .values(model)
                   .get_result(&connection())?)
            }

            fn get_one(&self, by: Expr<table>) -> Result<Self::ModelWithId, Error> {
                Ok(table
                   .filter(by)
                   .get_result::<Self::ModelWithId>(&connection())?)
            }

            fn get_all(&self, by: Expr<table>) -> Result<Vec<Self::ModelWithId>, Error> {
                Ok(table
                   .filter(by)
                   .get_results::<Self::ModelWithId>(&connection())?)
            }

            fn update(&self, model: &Self::Model, by: Expr<table>) -> Result<Self::ModelWithId, Error> {
                Ok(update(table)
                   .filter(by)
                   .set(model)
                   .get_result::<Self::ModelWithId>(&connection())?)
            }
        }
    };

    gen.into()
}
