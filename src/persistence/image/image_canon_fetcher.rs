use sea_orm::{DbConn, EntityTrait};

use crate::{domain::models::Image, persistence::entities::prelude::Images};

#[derive(Clone)]
pub struct ImageCanonFetcher {
    db_conn: DbConn,
}

impl ImageCanonFetcher {
    pub(in crate::persistence) fn new(db_conn: DbConn) -> Self {
        Self { db_conn }
    }

    pub async fn fetch_canon(&self) -> Result<Vec<Image>, String> {
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
