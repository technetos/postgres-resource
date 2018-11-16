use diesel::{
    deserialize::{self, FromSql},
    pg::Pg,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Uuid as UuidDiesel,
    not_none,
    expression::Expression,
};
use serde_derive::{Serialize, Deserialize};
use std::io::Write;
use uuid;

#[derive(Clone, Debug, AsExpression, PartialEq, FromSqlRow, Serialize, Deserialize, Hash, Eq)]
#[sql_type = "UuidDiesel"]
pub struct Uuid(pub uuid::Uuid);

impl ToSql<UuidDiesel, Pg> for Uuid {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        out.write_all(self.0.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<UuidDiesel, Pg> for Uuid {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let bytes = not_none!(bytes);
        Ok(Uuid(uuid::Uuid::from_slice(bytes)?))
    }
}

impl From<uuid::Uuid> for Uuid {
    fn from(uuid: uuid::Uuid) -> Self {
        Uuid(uuid)
    }
}
