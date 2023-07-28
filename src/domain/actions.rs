use crate::domain::actions::images::{
    ImageCanonFetcher, ImageCanonUpdater, ImageFetcher, ImageSaver, PaginatedImagesFetcher,
};

pub mod images;

pub trait ActionProvider {
    type ImageFetcherImpl: ImageFetcher + Clone + Send + Sync;
    fn get_image_fetcher(&self) -> Self::ImageFetcherImpl;

    type ImageSaverImpl: ImageSaver;
    fn get_image_saver(&self) -> Self::ImageSaverImpl;

    type ImageCanonFetcherImpl: ImageCanonFetcher;
    fn get_image_canon_fetcher(&self) -> Self::ImageCanonFetcherImpl;

    type ImageCanonUpdaterImpl: ImageCanonUpdater;
    fn get_image_canon_updater(&self) -> Self::ImageCanonUpdaterImpl;

    type PaginatedImagesFetcherImpl: PaginatedImagesFetcher;
    fn make_paginated_images_fetcher(&self) -> Self::PaginatedImagesFetcherImpl;
}
