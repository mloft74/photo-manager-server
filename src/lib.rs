use dotenvy::dotenv;

mod api;
mod database;
mod schema;
mod server_tracing;

pub async fn run() {
    dotenv().expect("Could not load .env");

    server_tracing::init_tracing_subscriber();

    let pool = database::make_connection_pool();
    database::run_migrations(&pool).await;

    let api_router = api::make_api_router(&pool);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(api_router.into_make_service())
        .await
        .expect("Error encountered while running the server");
}
