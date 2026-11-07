use dotenvy::dotenv;

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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("TcpListener should bind correctly");
    axum::serve(listener, api_router)
        .await
        .expect("Server should run without errors");
}
