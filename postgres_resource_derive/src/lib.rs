#![recursion_limit = "128"]

#[macro_use]
extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;

type TokenStream = proc_macro::TokenStream;

use proc_macro2::Span;
use syn::Ident;
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

impl Struct {
    fn camel_to_snake(&self) -> String {
        let name = &self.ident.to_string();
        let mut result = String::with_capacity(name.len());
        result.push_str(&name[..1].to_lowercase());
        for character in name[1..].chars() {
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
        fields.push(quote!(pub id: i32));

        let model_name = &self.input.ident;

        let model_name_lower = Ident::new(&self.input.camel_to_snake(), Span::call_site());

        let model_with_id_model_field = quote!(pub #model_name_lower: #model_name);

        fields.push(model_with_id_model_field);

        self.input.fields.iter().for_each(|field| {
            if field.fk() {
                let ty = &field.ty;
                let name = &field.name;

                if field.optional() {
                    fields.push(quote!(pub #name: Option<#ty>));
                } else {
                    fields.push(quote!(pub #name: #ty));
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
                    fields.push(quote!(pub #name: Option<#ty>));
                } else {
                    fields.push(quote!(pub #name: #ty));
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

    fn gen_queryable_row(&self) -> proc_macro2::TokenStream {
        let mut fields = Vec::new();
        fields.push(quote!(i32));

        self.input.fields.iter().for_each(|field| {
            let ty = &field.ty;
            if field.optional() {
                fields.push(quote!(Option<#ty>));
            } else {
                fields.push(quote!(#ty));
            }
        });

        quote_spanned!(Span::call_site()=> type Row = (#(#fields,)*);)
    }

    fn gen_queryable_inner_fields(&self) -> Vec<proc_macro2::TokenStream> {
        let mut fields = Vec::new();
        let mut inner_fields = Vec::new();

        let model_name = &self.input.ident;
        let model_name_lower = Ident::new(&self.input.camel_to_snake(), Span::call_site());

        let mut index = 0;

        // Push id
        let idx = syn::Index::from(index);
        fields.push(quote!(id: row.#idx));

        self.input.fields.iter().enumerate().for_each(|(i, field)| {
            let field_name = &field.name;
            if !field.fk() {
                index = i + 1;
                let idx = syn::Index::from(index);
                inner_fields.push(quote!(#field_name: row.#idx));
            }
        });

        let generated_inner_fields = quote!({ #(#inner_fields,)* });

        // Push inner fields
        fields.push(quote!(#model_name_lower: #model_name #generated_inner_fields));

        // Push remaining fields
        self.input.fields.iter().for_each(|field| {
            if field.fk() {
                index = index + 1;
                let name = &field.name;
                let idx = syn::Index::from(index);
                fields.push(quote!(#name: row.#idx));
            }
        });

        fields
    }

    fn gen_queryable_fields(&self) -> proc_macro2::TokenStream {
        let fields = self.gen_queryable_inner_fields();
        let model_with_id = self.gen_model_with_id_ident();

        quote! {
            #model_with_id {
                #(#fields,)*
            }
        }
    }

    fn gen_queryable_impl(&self) -> proc_macro2::TokenStream {
        let fields = self.gen_queryable_fields();
        let model_with_id = self.gen_model_with_id_ident();
        let sql_type = self.gen_sql_type();
        let row = self.gen_queryable_row();

        quote_spanned! {Span::call_site()=>
            impl diesel::Queryable<#sql_type, diesel::pg::Pg> for #model_with_id {
                #row
                fn build(row: Self::Row) -> Self {
                    #fields
                }
            }
        }
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
/// ### Model Definition
/// ```
/// #[resource(schema = crate::schema::accounts, table = "accounts")]
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
pub fn resource(attr: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_attr = parse_macro_input!(attr as Attrs);
    let parsed_struct = parse_macro_input!(input as Struct);

    let parsed = Parsed { attr: parsed_attr, input: parsed_struct };

    let model_with_id = parsed.gen_model_with_id();
    let model = parsed.gen_model();
    let resource_controller = parsed.gen_resource_controller();
    let queryable = parsed.gen_queryable_impl();

    let generated = quote_spanned! {Span::call_site()=>
        #model_with_id
        #model
        #queryable
        #resource_controller
    };

    generated.into()
}
