use async_trait::async_trait;

use crate::domain::models::Image;

#[async_trait]
pub trait ImageSaver: Clone + Send + Sync {
    async fn save_image(&self, image: &Image) -> Result<(), Box<dyn std::error::Error>>;
}
