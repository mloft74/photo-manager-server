use sea_orm::{CursorTrait, DbConn, EntityTrait};

use crate::{
    domain::models::Image,
    persistence::entities::{images, prelude::Images},
};

#[derive(Clone)]
pub struct PaginatedImagesFetcher {
    db_conn: DbConn,
}

pub struct ImagesPage {
    pub images: Vec<Image>,
    pub cursor: Option<i32>,
}

impl PaginatedImagesFetcher {
    pub fn new(db_conn: DbConn) -> Self {
        Self { db_conn }
    }

    pub async fn fetch_images(&self, count: u64, after: Option<i32>) -> Result<ImagesPage, String> {
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
