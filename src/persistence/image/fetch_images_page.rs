use async_trait::async_trait;
use sea_orm::{CursorTrait, EntityTrait};

use crate::{
    domain::{actions::image::FetchImagesPage, models::ImagesPage},
    persistence::{
        entities::{images, prelude::Images},
        persistence_manager::PersistenceManager,
    },
};

#[async_trait]
impl FetchImagesPage for PersistenceManager {
    async fn fetch_images_page(
        &self,
        count: u64,
        after: Option<i32>,
    ) -> Result<ImagesPage, String> {
        let mut pagination = Images::find().cursor_by(images::Column::Id);
        let pagination = match after {
            Some(cursor) => pagination.after(cursor),
            None => &mut pagination,
        };
        let images: Vec<_> = pagination
            .first(count)
            .all(&self.db_conn)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .collect();

        Ok(ImagesPage {
            cursor: images.last().map(|v| v.id),
            images: images.into_iter().map(|m| m.into()).collect(),
        })
    }
}
