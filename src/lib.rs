use dotenvy::dotenv;

use crate::domain::actions::ActionProvider;

mod api;
mod domain;
mod persistence;

mod server_tracing;

pub async fn run() {
    dotenv().expect("Could not load .env");

    let action_provider = persistence::init_persistence()
        .await
        .expect("Could not connect to DBMS");

    server_tracing::init_tracing_subscriber();

    let api_router = api::make_api_router(action_provider.get_image_saver());

    axum::Server::bind(&"0.0.0.0:3000".parse().expect("Couldn't parse server url"))
        .serve(api_router.into_make_service())
        .await
        .expect("Error encountered while running the server");
}
