#![recursion_limit = "128"]

#[macro_use]
extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;

use proc_macro2::Span;
use syn::Ident;

fn concat(string: &str, suffix: &str) -> syn::Ident {
    Ident::new(&format!("{}{}", string, suffix), Span::call_site())
}

type TokenStream = proc_macro::TokenStream;

#[proc_macro]
pub fn resource_controller(input: TokenStream) -> TokenStream {
    let string = &input.to_string()[..];
    let model = Ident::new(string, Span::call_site());

    let controller = concat(string, "Controller");
    let model_with_id = concat(string, "WithId");

    let gen = quote_spanned! {Span::call_site()=>
        pub struct #controller;

        impl ResourceDB for #controller {}

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

        impl ResourceController for #controller {
            fn create(&self, model: &Self::Model) -> Result<Self::ModelWithId, Error> {
                Ok(insert_into(table)
                   .values(model)
                   .get_result(&self.connection())?)
            }

            fn get_one(&self, by: Expr<table>) -> Result<Self::ModelWithId, Error> {
                Ok(table
                   .filter(by)
                   .get_result::<Self::ModelWithId>(&self.connection())?)
            }

            fn get_all(&self, by: Expr<table>) -> Result<Vec<Self::ModelWithId>, Error> {
                Ok(table
                   .filter(by)
                   .get_results::<Self::ModelWithId>(&self.connection())?)
            }

            fn update(&self, model: &Self::Model, by: Expr<table>) -> Result<Self::ModelWithId, Error> {
                Ok(update(table)
                   .filter(by)
                   .set(model)
                   .get_result::<Self::ModelWithId>(&self.connection())?)
            }
        }
    };

    gen.into()
}

use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token, Attribute,
};

#[derive(Debug)]
struct Struct {
    pub attrs: Vec<Attribute>,
    pub struct_token: Token![struct],
    pub ident: syn::Ident,
    pub brace_token: token::Brace,
    pub fields: Punctuated<Field, Token![,]>,
}

#[derive(Debug)]
struct Field {
    pub attr: Option<Vec<Attribute>>,
    pub name: Ident,
    pub colon_token: Token![:],
    pub ty: Ident,
}

impl Field {
    fn walk_attrs(&self, callback: &mut FnMut(&Ident)) {
        if let Some(ref field_attrs) = self.attr {
            field_attrs.iter().for_each(|a| {
                let Attribute { path, .. } = a;
                let syn::Path { segments, .. } = path;
                let syn::PathSegment { ident, .. } = &segments[0];

                callback(ident);
            })
        }
    }

    fn optional(&self) -> bool {
        let mut ret = false;
        self.walk_attrs(&mut |ref ident| {
            if *ident == "optional" {
                ret = true;
            }
        });
        ret
    }

    fn fk(&self) -> bool {
        let mut ret = false;
        self.walk_attrs(&mut |ref ident| {
            if *ident == "fk" {
                ret = true;
            }
        });
        ret
    }
}

impl Parse for Struct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Struct {
            attrs: input.call(Attribute::parse_outer)?,
            struct_token: input.parse()?,
            ident: input.parse()?,
            brace_token: braced!(content in input),
            fields: content.parse_terminated(Field::parse)?,
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Field {
            attr: input.call(Attribute::parse_outer).ok(),
            name: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

struct Attrs {
    pub schema: Schema,
    pub comma_token: Token![,],
    pub table: Table,
}

struct Schema {
    pub schema_token: Ident,
    pub assignment_token: Token![=],
    pub schema: syn::Path,
}

struct Table {
    pub table_token: Ident,
    pub assignment_token: Token![=],
    pub table: proc_macro2::Literal,
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Attrs {
            schema: input.call(Schema::parse)?,
            comma_token: input.parse()?,
            table: input.call(Table::parse)?,
        })
    }
}

impl Parse for Schema {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Schema {
            schema_token: input.parse()?,
            assignment_token: input.parse()?,
            schema: input.call(syn::Path::parse_mod_style)?,
        })
    }
}

impl Parse for Table {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Table {
            table_token: input.parse()?,
            assignment_token: input.parse()?,
            table: input.parse()?,
        })
    }
}

struct Parsed {
    pub attr: Attrs,
    pub input: Struct,
}

impl Parsed {
    fn gen_model_with_id_fields(&self) -> Vec<proc_macro2::TokenStream> {
        let mut fields = Vec::new();
        fields.push(quote_spanned!(Span::call_site()=> pub id: i32));

        let model_name = &self.input.ident;

        let model_name_lower =
            Ident::new(&self.input.ident.to_string().to_lowercase(), Span::call_site());

        let model_with_id_model_field =
            quote_spanned!(Span::call_site()=> pub #model_name_lower: #model_name);

        fields.push(model_with_id_model_field);

        self.input.fields.iter().for_each(|field| {
            if field.fk() {
                let ty = &field.ty;
                let name = &field.name;

                if field.optional() {
                    fields.push(quote_spanned!(Span::call_site()=> pub #name: Option<#ty>));
                } else {
                    fields.push(quote_spanned!(Span::call_site()=> pub #name: #ty));
                }
            }
        });
        fields
    }

    fn gen_model_with_id_ident(&self) -> Ident {
        Ident::new(&format!("{}WithId", self.input.ident), Span::call_site())
    }

    fn gen_model_with_id(&self) -> proc_macro2::TokenStream {
        let model_with_id = self.gen_model_with_id_ident();
        let table_name = &self.attr.table.table;
        let model_with_id_fields = self.gen_model_with_id_fields();

        quote_spanned! {Span::call_site()=>
            #[derive(Serialize, Deserialize, FromSqlRow, Associations, Identifiable, Debug, PartialEq)]
            #[table_name = #table_name]
            pub struct #model_with_id {
                #(#model_with_id_fields,)*
            }
        }
    }

    fn gen_model_fields(&self) -> Vec<proc_macro2::TokenStream> {
        let mut fields = Vec::new();
        self.input.fields.iter().for_each(|field| {
            if !field.fk() {
                let ty = &field.ty;
                let name = &field.name;

                if field.optional() {
                    fields.push(quote_spanned!(Span::call_site()=> pub #name: Option<#ty>));
                } else {
                    fields.push(quote_spanned!(Span::call_site()=> pub #name: #ty));
                }
            }
        });
        fields
    }

    fn gen_model(&self) -> proc_macro2::TokenStream {
        let model_name = &self.input.ident;
        let model_fields = self.gen_model_fields();
        let table_name = &self.attr.table.table;

        quote_spanned! {Span::call_site()=>
            #[derive(Serialize, Deserialize, FromSqlRow, Insertable, AsChangeset, Debug, PartialEq)]
            #[table_name = #table_name]
            pub struct #model_name {
                #(#model_fields,)*
            }
        }
    }

    fn gen_table(&self) -> proc_macro2::TokenStream {
        let schema = &self.attr.schema.schema;
        quote_spanned!(Span::call_site()=> #schema::table)
    }

    fn gen_sql_type(&self) -> proc_macro2::TokenStream {
        let schema = &self.attr.schema.schema;
        quote_spanned!(Span::call_site()=> #schema::SqlType)
    }

    fn gen_resource_controller(&self) -> proc_macro2::TokenStream {
        let model = &self.input.ident;
        let model_with_id = self.gen_model_with_id_ident();
        let controller = Ident::new(&format!("{}Controller", &self.input.ident), Span::call_site());
        let table = self.gen_table();
        let sql_type = self.gen_sql_type();

        quote_spanned! {Span::call_site()=>
            pub struct #controller;

            impl ResourceDB for #controller {}

            impl ResourceWithId for #controller {
                type ModelWithId = #model_with_id;
            }

            impl Resource for #controller {
                type Model = #model;
            }

            impl ResourceTable for #controller {
                type DBTable = #table;
            }

            impl ResourceSql for #controller {
                type SQLType = #sql_type;
            }

            impl ResourceController for #controller {
                fn create(&self, model: &Self::Model) -> Result<Self::ModelWithId, Error> {
                    Ok(insert_into(#table)
                       .values(model)
                       .get_result(&self.connection())?)
                }

                fn get_one(&self, by: Expr<#table>) -> Result<Self::ModelWithId, Error> {
                    Ok(#table
                       .filter(by)
                       .get_result::<Self::ModelWithId>(&self.connection())?)
                }

                fn get_all(&self, by: Expr<#table>) -> Result<Vec<Self::ModelWithId>, Error> {
                    Ok(#table
                       .filter(by)
                       .get_results::<Self::ModelWithId>(&self.connection())?)
                }

                fn update(&self, model: &Self::Model, by: Expr<#table>) -> Result<Self::ModelWithId, Error> {
                    Ok(update(#table)
                       .filter(by)
                       .set(model)
                       .get_result::<Self::ModelWithId>(&self.connection())?)
                }
            }
        }
    }
}

///
/// # Resources
///
/// ### Model Definition
/// /// use postgres_resource::*;
/// /// use diesel::{insert_into, prelude::*, result::Error, update};
/// /// use crate::schema::accounts;
///
/// ```
/// #[resource(schema = accounts, table = "accounts")]
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
///
/// ```
/// #[derive(Serialize, Deserialize, FromSqlRow, Associations, Identifiable, Debug, PartialEq)]
/// #[belongs_to(Verification)]
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
pub fn resource(attr: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_attr = parse_macro_input!(attr as Attrs);
    let parsed_struct = parse_macro_input!(input as Struct);

    let parsed = Parsed { attr: parsed_attr, input: parsed_struct };

    let model_with_id = parsed.gen_model_with_id();
    let model = parsed.gen_model();
    let resource_controller = parsed.gen_resource_controller();

    let generated = quote_spanned! {Span::call_site()=>
        #model_with_id
        #model
        #resource_controller
    };

    generated.into()

    //    let mut field_ty = Vec::<Ident>::with_capacity(parsed_struct.fields.len());
    //    parsed_struct.fields.into_iter().for_each(|field| {
    //        field_ty.push(field.ty);
    //    });
    //
    //    let query = quote_spanned!(Span::call_site()=> type Row = (#(#field_ty,)*););
}

