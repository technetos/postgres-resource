#![recursion_limit = "128"]

#[macro_use]
extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;

type TokenStream = proc_macro::TokenStream;

mod attr;
mod builder;
mod field;
mod r#struct;

use crate::{attr::*, builder::*, field::*, r#struct::*};

use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token, Attribute, Ident,
};

trait IdentExt {
    fn append(&self, string: &str) -> Ident;
}

impl IdentExt for syn::Ident {
    fn append(&self, string: &str) -> Ident {
        Ident::new(&format!("{}{}", self, string), self.span())
    }
}

fn camel_to_snake(string: &str) -> String {
    let mut result = String::with_capacity(string.len());
    result.push_str(&string[..1].to_lowercase());
    for character in string[1..].chars() {
        if character.is_uppercase() {
            result.push('_');
            for lowercase in character.to_lowercase() {
                result.push(lowercase);
            }
        } else {
            result.push(character);
        }
    }
    result
}

impl Input {
    ////    fn db_connection(&self) -> proc_macro2::TokenStream {
    ////        let mut connection = quote!(&self.connection());
    ////        let key = String::from("database");
    ////        let attrs = &self.input.attrs;
    ////        if attrs.contains_key(&key) {
    ////            connection = attrs.get(&key).unwrap().clone();
    ////        }
    ////        connection
    ////    }
    //
    //    fn model_with_id_fields(&self) -> Vec<proc_macro2::TokenStream> {
    //        let mut fields = Vec::new();
    //        fields.push(quote!(pub id: i32));
    //
    //        let model_name = &self.input.ident;
    //        let model_name_lower = Ident::new(&camel_to_snake(&self.input.ident.to_string()), Span::call_site());
    //
    //        fields.push(quote!(pub #model_name_lower: #model_name));
    //
    //        self.input.fields.iter().for_each(|field| {
    //            if field.fk() {
    //                let ty = field.ty();
    //                let name = &field.name;
    //                fields.push(quote!(pub #name: #ty));
    //            }
    //        });
    //        fields
    //    }
    //
    //    fn model_fields(&self) -> Vec<proc_macro2::TokenStream> {
    //    }
    //
    //    fn queryable_row(&self) -> proc_macro2::TokenStream {
    //        let mut fields = Vec::new();
    //        fields.push(quote!(i32));
    //
    //        self.input.fields.iter().for_each(|field| {
    //            let ty = field.ty();
    //            fields.push(quote!(#ty));
    //        });
    //
    //        quote!(type Row = (#(#fields,)*);)
    //    }
    //
    //    fn queryable_inner_fields(&self) -> Vec<proc_macro2::TokenStream> {
    //        let mut fields = Vec::new();
    //        let mut inner_fields = Vec::new();
    //
    //        let model_name = &self.input.ident;
    //        let model_name_lower = Ident::new(&camel_to_snake(&self.input.ident.to_string()), Span::call_site());
    //
    //        let mut index = 0;
    //
    //        // Push id
    //        let idx = syn::Index::from(index);
    //        fields.push(quote!(id: row.#idx));
    //
    //        self.input.fields.iter().enumerate().for_each(|(i, field)| {
    //            let field_name = &field.name;
    //            if !field.fk() {
    //                index = i + 1;
    //                let idx = syn::Index::from(index);
    //                inner_fields.push(quote!(#field_name: row.#idx));
    //            }
    //        });
    //
    //        let generated_inner_fields = quote!({ #(#inner_fields,)* });
    //
    //        // Push inner fields
    //        fields.push(quote!(#model_name_lower: #model_name #generated_inner_fields));
    //
    //        // Push remaining fields
    //        self.input.fields.iter().for_each(|field| {
    //            if field.fk() {
    //                index = index + 1;
    //                let name = &field.name;
    //                let idx = syn::Index::from(index);
    //                fields.push(quote!(#name: row.#idx));
    //            }
    //        });
    //
    //        fields
    //    }
    //
    ////    fn gen_queryable(&self) -> proc_macro2::TokenStream {
    ////        let model_with_id = self.model_with_id_ident();
    ////        let sql_type = self.input.sql_type();
    ////        let row = self.queryable_row();
    ////        let queryable_fields = self.queryable_inner_fields();
    ////
    ////        quote! {
    ////            impl diesel::Queryable<#sql_type, diesel::pg::Pg> for #model_with_id {
    ////                #row
    ////                fn build(row: Self::Row) -> Self {
    ////                    #queryable_fields
    ////                }
    //            }
    //        }
    //    }
    //

    fn gen_model(&self) -> Result<proc_macro2::TokenStream> {
        let table_macro = InferredTableMacro.build(&self)?;
        let model_with_id = ModelWithId(Fields).build(&self)?;
        let model = Model(InnerFields).build(&self)?;

        Ok(quote! {
            #[derive(Serialize, Deserialize, FromSqlRow, Associations, Identifiable, Debug, PartialEq)]
            #table_macro
            #model_with_id

            #[derive(Serialize, Deserialize, FromSqlRow, Insertable, AsChangeset, Debug, PartialEq)]
            #table_macro
            #model
        })
    }
    
 //   fn gen_controller(&self) -> proc_macro2::TokenStream {
 //       let model_with_id = self.parsed_struct.model_name();
 //       let model = self.parsed_struct.inner_model_name();
 //       let controller = self.parsed_struct.controller_name();

 //       let table = self.input.table();
 //       let sql_type = self.input.sql_type();
 //       //        let connection = self.db_connection();

 //       let connection = quote!();

 //       quote! {
 //           pub struct #controller;

 //           impl ResourceDB for #controller {}

 //           impl ResourceWithId for #controller {
 //               type ModelWithId = #model_with_id;
 //           }

 //           impl Resource for #controller {
 //               type Model = #model;
 //           }

 //           impl ResourceTable for #controller {
 //               type DBTable = #table;
 //           }

 //           impl ResourceSql for #controller {
 //               type SQLType = #sql_type;
 //           }

 //           impl ResourceController for #controller {
 //               fn create(&self, model: &Self::Model) -> Result<Self::ModelWithId, Error> {
 //                   Ok(insert_into(#table)
 //                      .values(model)
 //                      .get_result(#connection)?)
 //               }

 //               fn get_one(&self, by: Expr<#table>) -> Result<Self::ModelWithId, Error> {
 //                   Ok(#table
 //                      .filter(by)
 //                      .get_result::<Self::ModelWithId>(#connection)?)
 //               }

 //               fn get_all(&self, by: Expr<#table>) -> Result<Vec<Self::ModelWithId>, Error> {
 //                   Ok(#table
 //                      .filter(by)
 //                      .get_results::<Self::ModelWithId>(#connection)?)
 //               }

 //               fn update(&self, model: &Self::Model, by: Expr<#table>) -> Result<Self::ModelWithId, Error> {
 //                   Ok(update(#table)
 //                      .filter(by)
 //                      .set(model)
 //                      .get_result::<Self::ModelWithId>(#connection)?)
 //               }

 //               fn delete(&self, by: Expr<#table>) -> Result<usize, Error> {
 //                   Ok(delete(#table).filter(by).execute(#connection)?)
 //               }
 //           }
 //       }
 //   }
}
//
///
/// ### Model Definition
/// ```
/// #[resource]
/// struct Account {
///     #[optional]
///     uuid: Uuid,
///
///     #[optional]
///     username: String,
///
///     #[optional]
///     password: String,
///
///     #[optional]
///     email: String,
///
///     #[optional]
///     #[fk]
///     verification_id: i32,
/// }
/// ```
///
/// ### Generated result
/// ```
/// #[derive(Serialize, Deserialize, FromSqlRow, Associations, Identifiable, Debug, PartialEq)]
/// #[table_name = "accounts"]
/// pub struct AccountWithId {
///     pub id: i32,
///     pub account: Account,
///     pub verification_id: Option<i32>,
/// }
/// #[derive(Serialize, Deserialize, FromSqlRow, Insertable, AsChangeset, Debug, PartialEq)]
/// #[table_name = "accounts"]
/// pub struct Account {
///     pub uuid: Option<Uuid>,
///     pub username: Option<String>,
///     pub password: Option<String>,
///     pub email: Option<String>,
/// }
/// impl diesel::Queryable<accounts::SqlType, diesel::pg::Pg> for AccountWithId {
///     type Row = (i32, Option<Uuid>, Option<String>, Option<String>, Option<String>, Option<i32>);
///     fn build(row: Self::Row) -> Self {
///         AccountWithId {
///             id: row.0,
///             account: Account { uuid: row.1, username: row.2, password: row.3, email: row.4 },
///             verification_id: row.5,
///         }
///     }
/// }
/// pub struct AccountController;
///
/// impl ResourceDB for AccountController {}
///
/// impl ResourceWithId for AccountController {
///     type ModelWithId = AccountWithId;
/// }
///
/// impl Resource for AccountController {
///     type Model = Account;
/// }
///
/// impl ResourceTable for AccountController {
///     type DBTable = crate::schema::accounts::table;
/// }
///
/// impl ResourceSql for AccountController {
///     type SQLType = crate::schema::accounts::SqlType;
/// }
///
/// impl ResourceController for AccountController {
///     fn create(&self, model: &Self::Model) -> Result<Self::ModelWithId, Error> {
///         Ok(insert_into(crate::schema::accounts::table)
///            .values(model)
///            .get_result(&self.connection())?)
///     }
///
///     fn get_one(&self, by: Expr<crate::schema::accounts::table>) -> Result<Self::ModelWithId, Error> {
///         Ok(crate::schema::accounts::table)
///            .filter(by)
///            .get_result::<Self::ModelWithId>(&self.connection())?)
///     }
///
///     fn get_all(&self, by: Expr<crate::schema::accounts::table>) -> Result<Vec<Self::ModelWithId>, Error> {
///         Ok(crate::schema::accounts::table)
///            .filter(by)
///            .get_results::<Self::ModelWithId>(&self.connection())?)
///     }
///
///     fn update(&self, model: &Self::Model, by: Expr<crate::schema::accounts::table>) -> Result<Self::ModelWithId, Error> {
///         Ok(update(crate::schema::accounts::table)
///            .filter(by)
///            .set(model)
///            .get_result::<Self::ModelWithId>(&self.connection())?)
///     }
/// }
/// ```

#[proc_macro_attribute]
pub fn resource(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_struct = parse_macro_input!(input as Struct);

    let parsed = Input { parsed_struct };
    println!("{}", parsed.gen_model().unwrap());

    let generated = quote_spanned! {Span::call_site()=>

    };

    generated.into()
}
