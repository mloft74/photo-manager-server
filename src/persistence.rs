use std::env;

use sea_orm::{Database, DbConn};
use sea_orm_migration::MigratorTrait;

use crate::persistence::{migrator::Migrator, persistence_manager::PersistenceManager};

mod entities;
pub mod image;
mod migrator;
pub mod persistence_manager;

pub async fn init_persistence() -> PersistenceManager {
    let db_conn = connect().await;
    PersistenceManager::new(db_conn)
}

async fn connect() -> DbConn {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_conn = Database::connect(db_url)
        .await
        .expect("Database should be connectable from startup");

    Migrator::up(&db_conn, None)
        .await
        .expect("Database should be migratable from startup");

    db_conn
}
