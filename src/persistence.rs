use std::env;

use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::prelude::*;

use crate::{
    domain::actions::ActionProvider,
    persistence::{migrator::Migrator, persistence_manager::PersistenceManager},
};

mod entities;
pub mod image_manager;
mod migrator;
pub mod persistence_manager;

pub async fn init_persistence() -> Result<impl ActionProvider, Box<dyn std::error::Error>> {
    let db_conn = connect().await?;
    Ok(PersistenceManager::new(db_conn))
}

async fn connect() -> Result<DatabaseConnection, DbErr> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_name = env::var("DATABASE_NAME").expect("DATABASE_NAME must be set");
    let conn_url = format!("{}/{}", db_url, db_name);
    let db_conn = Database::connect(conn_url).await?;

    Migrator::up(&db_conn, None).await?;

    Ok(db_conn)
}
