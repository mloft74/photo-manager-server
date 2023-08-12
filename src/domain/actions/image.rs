use async_trait::async_trait;
use auto_impl::auto_impl;

use crate::domain::models::Image;

#[async_trait]
#[auto_impl(&)]
pub trait DeleteImage {
    async fn delete_image(&self, file_name: &str) -> Result<(), String>;
}

#[async_trait]
#[auto_impl(&)]
pub trait FetchCanon {
    async fn fetch_canon(&self) -> Result<Vec<Image>, String>;
}

#[async_trait]
#[auto_impl(&)]
pub trait UpdateCanon {
    async fn update_canon<'a, T: Iterator<Item = &'a Image> + Send>(
        &self,
        canon: T,
    ) -> Result<(), String>;
}

#[async_trait]
#[auto_impl(&)]
pub trait FetchImage {
    async fn fetch_image(&self, file_name: &str) -> Result<Option<Image>, String>;
}
