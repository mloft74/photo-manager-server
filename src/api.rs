use axum::{middleware, Router};

use crate::database::image_manager::ImageManager;

mod demo_routing;
mod error_handling;
mod image_server;
mod request_tracing;

pub fn make_api_router(image_manager: ImageManager) -> Router {
    let image_router = image_server::create_image_server_router();

    let demo_router = demo_routing::make_demo_router(image_manager);

    Router::new()
        .merge(image_router)
        .merge(demo_router)
        .layer(middleware::from_fn(request_tracing::print_request_response))
}
