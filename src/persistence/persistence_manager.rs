use sea_orm::DbConn;

use crate::persistence::image::{
    image_renamer::ImageRenamer, image_saver::ImageSaver,
    paginated_images_fetcher::PaginatedImagesFetcher,
};

#[derive(Clone)]
pub struct PersistenceManager {
    pub(in crate::persistence) db_conn: DbConn,
}

impl PersistenceManager {
    pub fn new(db_conn: DbConn) -> Self {
        Self { db_conn }
    }

    pub fn make_image_saver(&self) -> ImageSaver {
        ImageSaver::new(self.db_conn.clone())
    }

    pub fn make_paginated_images_fetcher(&self) -> PaginatedImagesFetcher {
        PaginatedImagesFetcher::new(self.db_conn.clone())
    }

    pub fn make_image_renamer(&self) -> ImageRenamer {
        ImageRenamer::new(self.db_conn.clone())
    }
}
