#![feature(custom_attribute)]
pub use postgres_resource_derive::resource;
pub mod uuid;

#[macro_use]
extern crate diesel;

use diesel::{
    expression::BoxableExpression,
    pg::{Pg, PgConnection},
    prelude::*,
    result::Error,
    sql_types::Bool,
    Connection,
};

pub trait ResourceDB {
    fn connection(&self) -> PgConnection {
        let env_var = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
        PgConnection::establish(&env_var[..]).expect("Unable to establish connection to database")
    }
}

pub trait Resource
where
    Self: ResourceTable,
{
    type Model: Insertable<Self::DBTable>;
}

pub trait ResourceWithId
where
    Self: ResourceSql,
{
    type ModelWithId: Queryable<Self::SQLType, Pg>;
}

pub trait ResourceTable {
    type DBTable: diesel::Table;
}

pub trait ResourceSql {
    type SQLType;
}

pub type Expr<T> = Box<BoxableExpression<T, Pg, SqlType = Bool>>;

type Result<T> = std::result::Result<T, Error>;

pub trait ResourceController
where
    Self: Resource + ResourceWithId + ResourceDB,
{
    fn create(&self, model: &Self::Model) -> Result<Self::ModelWithId>;
    fn get_one(&self, by: Expr<Self::DBTable>) -> Result<Self::ModelWithId>;
    fn get_all(&self, by: Expr<Self::DBTable>) -> Result<Vec<Self::ModelWithId>>;
    fn update(&self, model: &Self::Model, by: Expr<Self::DBTable>) -> Result<Self::ModelWithId>;
}
