use dotenvy::dotenv;
use tokio::net::TcpListener;

mod api;
mod domain;
mod persistence;
mod server_tracing;
mod state;

pub async fn run() {
    dotenv().expect(".env should be loadable from startup");

    let persistence_mngr = persistence::init_persistence().await;

    server_tracing::init_tracing_subscriber();

    let api_router = api::make_api_router(&persistence_mngr).await;

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("TcpListener should be valid");
    axum::serve(listener, api_router)
        .await
        .expect("Server should run without errors");
}
