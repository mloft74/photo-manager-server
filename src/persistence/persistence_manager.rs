use sea_orm::DatabaseConnection;

use crate::{
    domain::actions::ActionProvider,
    persistence::db_image::{
        db_image_canon_fetcher::DbImageCanonFetcher, db_image_canon_updater::DbImageCanonUpdater,
        db_image_fetcher::DbImageFetcher, db_image_saver::DbImageSaver,
        db_paginated_images_fetcher::DbPaginatedImagesFetcher,
    },
};

pub struct PersistenceManager {
    db_conn: DatabaseConnection,
}

impl PersistenceManager {
    pub fn new(db_conn: DatabaseConnection) -> Self {
        Self { db_conn }
    }
}

impl ActionProvider for PersistenceManager {
    type ImageFetcherImpl = DbImageFetcher;
    fn get_image_fetcher(&self) -> Self::ImageFetcherImpl {
        Self::ImageFetcherImpl::new(self.db_conn.clone())
    }

    type ImageSaverImpl = DbImageSaver;
    fn get_image_saver(&self) -> Self::ImageSaverImpl {
        Self::ImageSaverImpl::new(self.db_conn.clone())
    }

    type ImageCanonFetcherImpl = DbImageCanonFetcher;
    fn get_image_canon_fetcher(&self) -> Self::ImageCanonFetcherImpl {
        Self::ImageCanonFetcherImpl::new(self.db_conn.clone())
    }

    type ImageCanonUpdaterImpl = DbImageCanonUpdater;
    fn get_image_canon_updater(&self) -> Self::ImageCanonUpdaterImpl {
        Self::ImageCanonUpdaterImpl::new(self.db_conn.clone())
    }

    type PaginatedImagesFetcherImpl = DbPaginatedImagesFetcher;
    fn make_paginated_images_fetcher(&self) -> Self::PaginatedImagesFetcherImpl {
        Self::PaginatedImagesFetcherImpl::new(self.db_conn.clone())
    }
}
