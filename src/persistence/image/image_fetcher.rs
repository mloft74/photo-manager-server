use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};

use crate::{
    domain::models::Image,
    persistence::entities::{images, prelude::Images},
};

#[derive(Clone)]
pub struct ImageFetcher {
    db_conn: DbConn,
}

impl ImageFetcher {
    pub fn new(db_conn: DbConn) -> Self {
        Self { db_conn }
    }

    pub async fn fetch_image(&self, file_name: &str) -> Result<Option<Image>, String> {
        Ok(Images::find()
            .filter(images::Column::FileName.eq(file_name))
            .one(&self.db_conn)
            .await
            .map_err(|e| e.to_string())?
            .map(|m| m.into()))
    }
}
