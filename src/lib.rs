mod database;
mod demo_routing;
mod image_server;
mod schema;
mod server_tracing;

use axum::middleware;
use dotenvy::dotenv;

pub async fn run() {
    dotenv().expect("Could not load .env");

    server_tracing::init_tracing_subscriber();

    let pool = database::make_connection_pool();
    database::run_migrations(&pool).await;

    let image_router = image_server::create_image_server_router()
        .layer(middleware::from_fn(server_tracing::print_request_response));

    let demo_router = demo_routing::make_demo_router(&pool);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(image_router.merge(demo_router).into_make_service())
        .await
        .expect("Error encountered while running the server");
}
