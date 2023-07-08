use async_trait::async_trait;

use crate::domain::models::Image;

#[async_trait]
pub trait ImageRepo: Clone + Send + Sync {
    async fn get_image(&self, file_name: &str)
        -> Result<Option<Image>, Box<dyn std::error::Error>>;
    async fn save_image(&self, image: Image) -> Result<(), Box<dyn std::error::Error>>;
}
