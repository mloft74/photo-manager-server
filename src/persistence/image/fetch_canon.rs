use async_trait::async_trait;
use sea_orm::EntityTrait;

use crate::{
    domain::{actions::image::FetchCanon, models::Image},
    persistence::{entities::prelude::Images, persistence_manager::PersistenceManager},
};

#[async_trait]
impl FetchCanon for PersistenceManager {
    async fn fetch_canon(&self) -> Result<Vec<Image>, String> {
        let images: Vec<_> = Images::find()
            .all(&self.db_conn)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|m| m.into())
            .collect();

        Ok(images)
    }
}
