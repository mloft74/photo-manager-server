use crate::domain::actions::images::{ImageGetter, ImageSaver};

pub mod images;

pub trait ActionProvider {
    type ImageGetterImpl: ImageGetter;
    fn get_image_getter(&self) -> Self::ImageGetterImpl;

    type ImageSaverImpl: ImageSaver;
    fn get_image_saver(&self) -> Self::ImageSaverImpl;
}
