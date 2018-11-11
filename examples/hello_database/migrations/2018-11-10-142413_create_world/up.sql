-- Your SQL goes here
create table worlds (
  id SERIAL PRIMARY KEY NOT NULL,
  uuid UUID UNIQUE NOT NULL,
  name TEXT UNIQUE NOT NULL
)
