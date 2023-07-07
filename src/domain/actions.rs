use crate::domain::actions::images::ImageSaver;

pub mod images;

pub trait ActionProvider {
    type ImageSaverImpl: ImageSaver;
    fn get_image_saver(&self) -> Self::ImageSaverImpl;
}
