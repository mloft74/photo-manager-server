use sea_orm::{Database, DbErr};

pub mod image_manager;
pub mod models;

const DB_URL: &str = "postgres://admin:root@db";
const DB_NAME: &str = "photo_manager_server";

pub async fn connect() -> Result<(), DbErr> {
    let db = Database::connect(DB_URL).await?;

    Ok(())
}
