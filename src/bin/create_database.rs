use std::env;

use dotenvy::dotenv;
use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};

#[tokio::main]
async fn main() {
    dotenv().expect("Could not load .env");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");
    let db = Database::connect(db_url)
        .await
        .expect("Could not connect to DBMS");
    db.execute(Statement::from_string(
        DbBackend::Postgres,
        format!("CREATE DATABASE \"{}\";", db_name),
    ))
    .await
    .expect("Could not create database");
}
