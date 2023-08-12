use async_trait::async_trait;

#[async_trait]
pub trait DeleteImage {
    async fn delete_image(&self, file_name: &str) -> Result<(), String>;
}
