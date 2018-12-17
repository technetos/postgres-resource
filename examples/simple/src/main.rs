use postgres_resource::*;

#[resource]
#[table = "worlds"]
struct World {
    uuid: Uuid,
    name: String,
}

fn main() {
}
