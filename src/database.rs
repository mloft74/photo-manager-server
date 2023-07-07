use std::env;

use sea_orm::{Database, DbErr};

pub mod image_manager;
pub mod models;

pub async fn connect() -> Result<(), DbErr> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");
    let conn_url = format!("{}/{}", db_url, db_name);
    let db = Database::connect(conn_url).await?;

    Ok(())
}
