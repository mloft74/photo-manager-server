use std::env;

use deadpool_diesel::postgres::{Manager, Pool, Runtime};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub mod image_manager;
pub mod models;

// This embeddes the migrations into the application binary.
// The migration path is relative to the `CARGO_MANIFEST_DIR`.
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub fn make_connection_pool() -> Pool {
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = Manager::new(url, Runtime::Tokio1);
    Pool::builder(manager)
        .max_size(8)
        .build()
        .expect("Error creating database pool")
}

pub async fn run_migrations(pool: &Pool) {
    // Lot's of unwraps here since there are nested results and whatnot.
    // rust-analyzer is having trouble with types here, so I'm just going to keep it as unwrap.
    let conn = pool.get().await.unwrap();
    conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
        .await
        .unwrap()
        .unwrap();
}
