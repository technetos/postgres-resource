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

#[macro_export]
macro_rules! resource_controller {
    ($model:ident) => {

        mashup! {
            controller["controller"] = $model Controller;
            modelWithId["modelWithId"] = $model WithId;
        }

        controller! {
            pub struct "controller";

            modelWithId! {
                impl ResourceWithId for "controller" {
                    type ModelWithId = "modelWithId";
                }
            }

            impl Resource for "controller" {
                type Model = $model;
            }

            impl ResourceTable for "controller" {
                type DBTable = table;
            }

            impl ResourceSql for "controller" {
                type SQLType = SqlType;
            }

            use crate::db::establish_connection as connection;

            impl ResourceController for "controller" {
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
        }
    }
}
