use std::env;

use sea_orm::{Database, DbErr};
use sea_orm_migration::prelude::*;

use crate::persistence::migrator::Migrator;

mod entities;
pub mod image_manager;
mod migrator;

pub async fn connect() -> Result<(), DbErr> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");
    let conn_url = format!("{}/{}", db_url, db_name);
    let db = Database::connect(conn_url).await?;

    Migrator::up(&db, None).await?;

    Ok(())
}
