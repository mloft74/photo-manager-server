mod image_server;
mod server_tracing;

use axum::middleware;

pub async fn run() -> Result<(), hyper::Error> {
    server_tracing::init_tracing_subscriber();

    let app = image_server::create_image_server_router()
        .layer(middleware::from_fn(server_tracing::print_request_response));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
}
