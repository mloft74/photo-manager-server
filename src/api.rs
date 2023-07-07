use axum::{middleware, Router};

use crate::domain::actions::ActionProvider;

mod error_handling;
mod image_server;
mod request_tracing;
mod routing;

pub fn make_api_router(action_provider: &(impl ActionProvider + 'static)) -> Router {
    let image_server_router = image_server::create_image_server_router();

    let demo_router = routing::make_api_router(action_provider);

    Router::new()
        .merge(image_server_router)
        .merge(demo_router)
        .layer(middleware::from_fn(request_tracing::print_request_response))
}
