use dotenvy::dotenv;

use crate::database::image_manager::ImageManager;

mod api;
mod database;
mod domain;
mod schema;
mod server_tracing;

pub async fn run() {
    dotenv().expect("Could not load .env");

    server_tracing::init_tracing_subscriber();

    let pool = database::make_connection_pool();
    database::run_migrations(&pool).await;

    let image_manager = ImageManager::new(pool.clone());

    let api_router = api::make_api_router(image_manager);

    axum::Server::bind(&"0.0.0.0:3000".parse().expect("Couldn't parse server url"))
        .serve(api_router.into_make_service())
        .await
        .expect("Error encountered while running the server");
}
