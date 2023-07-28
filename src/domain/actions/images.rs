use async_trait::async_trait;

use crate::domain::models::Image;

#[async_trait]
pub trait ImageFetcher: Clone + Send + Sync {
    async fn fetch_image(&self, file_name: &str) -> Result<Option<Image>, String>;
}

#[async_trait]
pub trait ImageSaver: Clone + Send + Sync {
    async fn save_image(&self, image: &Image) -> Result<(), String>;
}

#[async_trait]
pub trait PaginatedImagesFetcher: Clone + Send + Sync {
    async fn fetch_images(&self, count: u64, after: Option<i32>) -> Result<Vec<Image>, String>;
}

#[async_trait]
pub trait ImageCanonFetcher: Clone + Send + Sync {
    async fn fetch_canon(&self) -> Result<Vec<Image>, String>;
}

#[async_trait]
pub trait ImageCanonUpdater: Clone + Send + Sync {
    async fn update_canon<'a, T: Iterator<Item = &'a Image> + Send>(
        &self,
        canon: T,
    ) -> Result<(), String>;
}
