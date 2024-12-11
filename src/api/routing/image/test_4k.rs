use axum::Router;
use fetcher::Fetcher;
use test_screensaver_state::TestScreensaverManager;

use crate::domain::models::Image;

mod current;
mod fetcher;
mod get;
mod resolve;
mod test_screensaver_state;

const DELTA_4K_FILE_NAME: &str = "delta_4k.jpeg";
fn delta_4k() -> Image {
    Image {
        file_name: DELTA_4K_FILE_NAME.to_string(),
        width: 3840,
        height: 2160,
    }
}

const DELTA_RESIZED_FILE_NAME: &str = "delta_resized.jpeg";
fn delta_resized() -> Image {
    Image {
        file_name: DELTA_RESIZED_FILE_NAME.to_string(),
        width: 1920,
        height: 1080,
    }
}

pub fn make_test_4k_router() -> Router {
    let mngr = TestScreensaverManager::new();

    Router::new().nest(
        "/test_4k",
        Router::new()
            .merge(get::make_get_router(Fetcher))
            .merge(current::make_current_router(mngr.clone()))
            .merge(resolve::make_resolve_router(mngr)),
    )
}
