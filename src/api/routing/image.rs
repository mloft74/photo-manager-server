use axum::Router;

use crate::domain::actions::ActionProvider;

mod get;
mod upload;

pub fn make_image_router(action_provider: &(impl ActionProvider + 'static)) -> Router {
    Router::new().nest(
        "/image",
        Router::new()
            .merge(upload::make_upload_router(
                action_provider.get_image_getter(),
                action_provider.get_image_saver(),
            ))
            .merge(get::make_get_router(action_provider.get_image_getter())),
    )
}
