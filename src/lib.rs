use dotenvy::dotenv;

mod api;
mod domain;
mod persistence;
mod state;

mod server_tracing;

pub async fn run() {
    dotenv().expect(".env should be loadable from startup");

    let p_mngr = persistence::init_persistence().await;

    server_tracing::init_tracing_subscriber();

    let api_router = api::make_api_router(&p_mngr).await;

    axum::Server::bind(&"0.0.0.0:3000".parse().expect("Server URL should be valid"))
        .serve(api_router.into_make_service())
        .await
        .expect("Server should run without errors");
}
