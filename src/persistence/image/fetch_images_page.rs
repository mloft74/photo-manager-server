use std::cmp::Reverse;

use async_trait::async_trait;
use sea_orm::{sea_query::PostgresQueryBuilder, CursorTrait, EntityTrait, QueryOrder};
use tracing::debug;

use crate::{
    domain::{
        actions::image::{FetchImagesPage, PaginationOrder},
        models::ImagesPage,
    },
    persistence::{
        entities::{images, prelude::Images},
        PersistenceManager,
    },
};

#[async_trait]
impl FetchImagesPage for PersistenceManager {
    async fn fetch_images_page(
        &self,
        count: u64,
        cursor_value: Option<i32>,
        order: PaginationOrder,
    ) -> Result<ImagesPage, String> {
        let mut cursor = Images::find().cursor_by(images::Column::Id);
        if let Some(cursor_value) = cursor_value {
            match order {
                PaginationOrder::NewToOld => cursor.before(cursor_value),
                PaginationOrder::OldToNew => cursor.after(cursor_value),
            };
        }
        match order {
            PaginationOrder::NewToOld => cursor.last(count),
            PaginationOrder::OldToNew => cursor.first(count),
        };

        let query = cursor.query().to_string(PostgresQueryBuilder);
        debug!("query: {}", query);

        let mut images = cursor.all(&self.db_conn).await.map_err(|e| e.to_string())?;
        match order {
            PaginationOrder::NewToOld => images.sort_by_key(|i| Reverse(i.id)),
            PaginationOrder::OldToNew => images.sort_by_key(|i| i.id),
        };
        for image in images.iter() {
            debug!("id: {}, name: {}", image.id, &image.file_name);
        }

        Ok(ImagesPage {
            cursor: images.last().map(|v| v.id),
            images: images.into_iter().map(|m| m.into()).collect(),
        })
    }
}
