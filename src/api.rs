use axum::{middleware, Router};
use deadpool_diesel::postgres::Pool;

mod demo_routing;
mod error_handling;
mod image_server;
mod request_tracing;

pub fn make_api_router(pool: &Pool) -> Router {
    let image_router = image_server::create_image_server_router();

    let demo_router = demo_routing::make_demo_router(pool);

    Router::new()
        .merge(image_router)
        .merge(demo_router)
        .layer(middleware::from_fn(request_tracing::print_request_response))
}
