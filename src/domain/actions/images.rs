use async_trait::async_trait;

use crate::domain::models::Image;

#[async_trait]
pub trait ImageGetter: Clone + Send + Sync {
    async fn get_image(&self, file_name: &str)
        -> Result<Option<Image>, Box<dyn std::error::Error>>;
}

#[async_trait]
pub trait ImageSaver: Clone + Send + Sync {
    async fn save_image(&self, image: &Image) -> Result<(), Box<dyn std::error::Error>>;
}

#[async_trait]
pub trait ImageCanonSaver: Clone + Send + Sync {
    async fn save_canon<'a, T: Iterator<Item = &'a Image> + Send>(
        &self,
        canon: T,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
