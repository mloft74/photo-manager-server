use sea_orm::{DatabaseConnection, EntityTrait};

use crate::{domain::models::Image, persistence::entities::prelude::Images};

#[derive(Clone)]
pub struct ImageCanonFetcher {
    db_conn: DatabaseConnection,
}

impl ImageCanonFetcher {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }

    pub async fn fetch_canon(&self) -> Result<Vec<Image>, String> {
        let images: Vec<Image> = Images::find()
            .all(&self.db_conn)
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|m| m.into())
            .collect();

        Ok(images)
    }
}
