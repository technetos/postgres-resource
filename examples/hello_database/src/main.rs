#[macro_use]
extern crate diesel as other_diesel;

#[macro_use]
extern crate serde_derive;

use diesel::{
    self, insert_into, prelude::*, result::Error, update, Associations, FromSqlRow, Identifiable,
    Insertable, Queryable,
};
use postgres_resource::*;
use uuid::Uuid;
use crate::schema::worlds;
mod schema;


#[resource(schema = worlds, table = "worlds")]
struct World {
    uuid: Uuid,
    name: String,
}

fn main() {
    println!("{:#?}", create_world("Mercury").expect("already exists"));
    println!("{:#?}", create_world("Venus").expect("already exists"));
    println!("{:#?}", create_world("Earth").expect("already exists"));
    println!("{:#?}", create_world("Mars").expect("already exists"));
}

fn create_world(name: &str) -> Result<WorldWithId, ()> {
    let modelWithId = WorldController.create(&World {
        uuid: Uuid::new_v4(),
        name: name.to_string(),
    }).map_err(|_| ())?;

    return Ok(modelWithId);
}
