use axum::Router;

use crate::domain::repos::RepoProvider;

mod get;
mod upload;

pub fn make_image_router(action_provider: &(impl RepoProvider + 'static)) -> Router {
    Router::new().nest(
        "/image",
        Router::new()
            .merge(upload::make_upload_router(action_provider.get_image_repo()))
            .merge(get::make_get_router(action_provider.get_image_repo())),
    )
}
