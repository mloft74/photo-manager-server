use async_trait::async_trait;

use crate::domain::models::Image;

#[async_trait]
pub trait DeleteImage {
    async fn delete_image(&self, file_name: &str) -> Result<(), String>;
}

#[async_trait]
pub trait FetchCanon {
    async fn fetch_canon(&self) -> Result<Vec<Image>, String>;
}
