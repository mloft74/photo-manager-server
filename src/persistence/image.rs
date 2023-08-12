use sea_orm::ActiveValue;

use crate::{domain::models::Image, persistence::entities::images};

pub mod delete_image;
pub mod fetch_canon;
pub mod fetch_image;
pub mod fetch_images_page;
pub mod image_saver;
pub mod rename_image;
pub mod update_canon;

fn active_model_for_insert_from(image: &Image) -> images::ActiveModel {
    images::ActiveModel {
        file_name: ActiveValue::Set(image.file_name.clone()),
        width: ActiveValue::Set(image.width as i32),
        height: ActiveValue::Set(image.height as i32),
        ..Default::default()
    }
}

impl From<images::Model> for Image {
    fn from(value: images::Model) -> Self {
        Self {
            file_name: value.file_name,
            width: value.width as u32,
            height: value.height as u32,
        }
    }
}
