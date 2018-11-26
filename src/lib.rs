pub use postgres_resource_derive::resource;

use diesel::{
    self,
    expression::BoxableExpression,
    pg::{Pg, PgConnection},
    prelude::*,
    result::Error,
    sql_types::Bool,
    Connection,
};

pub trait ResourceDB {
    fn connection(&self) -> PgConnection {
        self.connection_string("DATABASE_URL")
    }

    fn connection_string(&self, string: &str) -> PgConnection {
        let env_var = std::env::var(&string).expect(&format!("{} not set", &string));
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
    fn delete(&self, by: Expr<Self::DBTable>) -> Result<usize>;
}
