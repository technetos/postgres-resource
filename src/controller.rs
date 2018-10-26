use diesel::{
    self, expression::BoxableExpression, pg::Pg, prelude::*, result::Error, sql_types::Bool,
};

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
    Self: Resource + ResourceWithId,
{
    fn create(&self, model: &Self::Model) -> Result<Self::ModelWithId>;
    fn get_one(&self, by: Expr<Self::DBTable>) -> Result<Self::ModelWithId>;
    fn get_all(&self, by: Expr<Self::DBTable>) -> Result<Vec<Self::ModelWithId>>;
    fn update(&self, model: &Self::Model, by: Expr<Self::DBTable>) -> Result<Self::ModelWithId>;
}