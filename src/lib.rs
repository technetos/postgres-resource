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

pub trait Resource {
    type Table: diesel::Table;
    type Model: Insertable<Self::Table>;
}

pub trait ResourceWithId {
    type SQLType;
    type ModelWithId: Queryable<Self::SQLType, Pg>;
}

pub type Expr<T> = Box<BoxableExpression<T, Pg, SqlType = Bool>>;

type Result<T> = std::result::Result<T, Error>;

pub trait ResourceController
where
    Self: Resource + ResourceWithId + ResourceDB,
{
    fn create(&self, model: &Self::Model) -> Result<Self::ModelWithId>;
    fn get_one(&self, by: Expr<Self::Table>) -> Result<Self::ModelWithId>;
    fn get_all(&self, by: Expr<Self::Table>) -> Result<Vec<Self::ModelWithId>>;
    fn update(&self, model: &Self::Model, by: Expr<Self::Table>) -> Result<Self::ModelWithId>;
    fn delete(&self, by: Expr<Self::Table>) -> Result<usize>;
}
