use axum::{middleware, Router};
use tower_http::trace::TraceLayer;

use crate::domain::repos::RepoProvider;

mod image_server;
mod request_tracing;
mod routing;

const IMAGES_DIR: &str = "/var/lib/photo_manager_server/images";

pub fn make_api_router(action_provider: &(impl RepoProvider + 'static)) -> Router {
    let image_server_router = image_server::create_image_server_router();

    let demo_router = routing::make_api_router(action_provider);

    Router::new()
        .merge(image_server_router)
        .merge(demo_router)
        .layer(middleware::from_fn(request_tracing::print_request_response))
        .layer(TraceLayer::new_for_http())
}
