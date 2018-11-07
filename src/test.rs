use diesel::types::Uuid;
use postgres_resource_derive::resource;

#[resource(schema = crate::schema::accounts, table = "accounts")]
struct Account {
    #[optional]
    uuid: Uuid,

    #[optional]
    username: String,

    #[optional]
    password: String,

    #[optional]
    email: String,

    #[optional]
    #[fk]
    verification_id: i32,
}

#[test]
fn test() {}
