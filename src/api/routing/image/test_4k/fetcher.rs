use crate::domain::{actions::image::FetchTestImage, models::Image};

use crate::api::routing::image::test_4k::{
    delta_4k, delta_resized, DELTA_4K_FILE_NAME, DELTA_RESIZED_FILE_NAME,
};

#[derive(Clone)]
pub struct Fetcher;

impl FetchTestImage for Fetcher {
    fn fetch_image(&self, file_name: &str) -> Option<Image> {
        match file_name {
            DELTA_4K_FILE_NAME => Some(delta_4k()),
            DELTA_RESIZED_FILE_NAME => Some(delta_resized()),
            _ => None,
        }
    }
}
