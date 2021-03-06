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
mod model;
mod queryable;

use crate::{model::*, queryable::*, builder::*, r#struct::*};

use proc_macro2::Span;
use syn::{parse::Result, Ident};

use heck::{CamelCase, SnakeCase};

trait IdentExt {
    fn append(&self, string: &str) -> Ident;
    fn camel_case(&self) -> Ident;
    fn snake_case(&self) -> Ident;
}

impl IdentExt for syn::Ident {
    fn append(&self, string: &str) -> Ident {
        Ident::new(&format!("{}{}", self, string), self.span())
    }

    fn camel_case(&self) -> Ident {
        Ident::new(&self.to_string().to_camel_case(), self.span())
    }

    fn snake_case(&self) -> Ident {
        Ident::new(&self.to_string().to_snake_case(), self.span())
    }
}

impl Input {
    fn gen_queryable(&self) -> Result<proc_macro2::TokenStream> {
        Ok(Queryable.build(&self)?)
    }

    fn gen_model(&self) -> Result<proc_macro2::TokenStream> {
        let table_macro = TableMacro.build(&self)?;

        let model_with_id = ModelWithId.build(&self)?;
        let model = Model.build(&self)?;

        Ok(quote! {
            #[derive(Serialize, Deserialize, FromSqlRow, Associations, Identifiable, Debug, PartialEq)]
            #table_macro
            #model_with_id

            #[derive(Serialize, Deserialize, FromSqlRow, Insertable, AsChangeset, Debug, PartialEq)]
            #table_macro
            #model
        })
    }

    fn gen_controller(&self) -> Result<proc_macro2::TokenStream> {
        let schema = Schema.build(&self)?;
        let connection = DatabaseConnection.build(&self)?;

        let model = self.parsed_struct.inner_model_name();
        let model_with_id = self.parsed_struct.model_name_with_id();
        let controller = self.parsed_struct.controller_name();

        Ok(quote! {
            pub struct #controller;

            impl ResourceDB for #controller {}
            impl Resource for #controller {
                type Table = #schema::table;
                type Model = #model;
            }
            impl ResourceWithId for #controller {
                type SQLType = #schema::SqlType;
                type ModelWithId = #model_with_id;
            }
            impl ResourceController for #controller {
                fn create(&self, model: &Self::Model) -> Result<Self::ModelWithId, Error> {
                    Ok(insert_into(#schema::table)
                       .values(model)
                       .get_result(#connection)?)
                }

                fn get_one(&self, by: Expr<#schema::table>) -> Result<Self::ModelWithId, Error> {
                    Ok(#schema::table
                       .filter(by)
                       .get_result::<Self::ModelWithId>(#connection)?)
                }

                fn get_all(&self, by: Expr<#schema::table>) -> Result<Vec<Self::ModelWithId>, Error> {
                    Ok(#schema::table
                       .filter(by)
                       .get_results::<Self::ModelWithId>(#connection)?)
                }

                fn update(&self, model: &Self::Model, by: Expr<#schema::table>) -> Result<Self::ModelWithId, Error> {
                    Ok(update(#schema::table)
                       .filter(by)
                       .set(model)
                       .get_result::<Self::ModelWithId>(#connection)?)
                }

                fn delete(&self, by: Expr<#schema::table>) -> Result<usize, Error> {
                    Ok(delete(#schema::table).filter(by).execute(#connection)?)
                }
            }
        })
    }
}

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
/// #[belongs_to(Verification)]
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
///     type SQLType = crate::schema::accounts::SqlType;
///     type ModelWithId = AccountWithId;
/// }
///
/// impl Resource for AccountController {
///     type DBTable = crate::schema::accounts::table;
///     type Model = Account;
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
pub fn resource(_: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_struct = parse_macro_input!(input as Struct);
    let parsed = Input { parsed_struct };

    let model = parsed.gen_model().unwrap();
    let controller = parsed.gen_controller().unwrap();
    let queryable = parsed.gen_queryable().unwrap();

    let generated = quote_spanned! {Span::call_site()=>
        #model
        #queryable
        #controller
    };

    generated.into()
}
