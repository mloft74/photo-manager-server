use crate::domain::actions::images::{
    ImageCanonFetcher, ImageCanonUpdater, ImageFetcher, ImageSaver,
};

pub mod images;

pub trait ActionProvider {
    type ImageFetcherImpl: ImageFetcher + Clone + Send + Sync;
    fn get_image_fetcher(&self) -> Self::ImageFetcherImpl;

    type ImageSaverImpl: ImageSaver + Clone + Send + Sync;
    fn get_image_saver(&self) -> Self::ImageSaverImpl;

    type ImageCanonFetcherImpl: ImageCanonFetcher + Clone + Send + Sync;
    fn get_image_canon_fetcher(&self) -> Self::ImageCanonFetcherImpl;

    type ImageCanonUpdaterImpl: ImageCanonUpdater + Clone + Send + Sync;
    fn get_image_canon_updater(&self) -> Self::ImageCanonUpdaterImpl;
}
