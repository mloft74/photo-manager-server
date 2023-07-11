use crate::domain::actions::images::{ImageCanonUpdater, ImageGetter, ImageSaver};

pub mod images;

pub trait ActionProvider {
    type ImageGetterImpl: ImageGetter + Clone + Send + Sync;
    fn get_image_getter(&self) -> Self::ImageGetterImpl;

    type ImageSaverImpl: ImageSaver + Clone + Send + Sync;
    fn get_image_saver(&self) -> Self::ImageSaverImpl;

    type ImageCanonUpdaterImpl: ImageCanonUpdater + Clone + Send + Sync;
    fn get_image_canon_updater(&self) -> Self::ImageCanonUpdaterImpl;
}
