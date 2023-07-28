use sea_orm::DatabaseConnection;

use crate::persistence::image::{
    image_canon_fetcher::ImageCanonFetcher, image_canon_updater::ImageCanonUpdater,
    image_fetcher::ImageFetcher, image_saver::ImageSaver,
    paginated_images_fetcher::PaginatedImagesFetcher,
};

pub struct PersistenceManager {
    db_conn: DatabaseConnection,
}

impl PersistenceManager {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }

    pub fn make_image_fetcher(&self) -> ImageFetcher {
        ImageFetcher::new(self.db_conn.clone())
    }

    pub fn make_image_saver(&self) -> ImageSaver {
        ImageSaver::new(self.db_conn.clone())
    }

    pub fn make_image_canon_fetcher(&self) -> ImageCanonFetcher {
        ImageCanonFetcher::new(self.db_conn.clone())
    }

    pub fn make_image_canon_updater(&self) -> ImageCanonUpdater {
        ImageCanonUpdater::new(self.db_conn.clone())
    }

    fn make_paginated_images_fetcher(&self) -> PaginatedImagesFetcher {
        PaginatedImagesFetcher::new(self.db_conn.clone())
    }
}
