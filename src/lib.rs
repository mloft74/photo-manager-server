use dotenvy::dotenv;

use crate::database::image_manager::ImageManager;

mod api;
mod database;
mod domain;
mod server_tracing;

pub use database::connect;

pub async fn run() {
    dotenv().expect("Could not load .env");

    server_tracing::init_tracing_subscriber();

    let image_manager = ImageManager {};

    let api_router = api::make_api_router(image_manager);

    axum::Server::bind(&"0.0.0.0:3000".parse().expect("Couldn't parse server url"))
        .serve(api_router.into_make_service())
        .await
        .expect("Error encountered while running the server");
}
