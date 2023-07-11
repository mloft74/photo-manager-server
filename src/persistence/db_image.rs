use sea_orm::ActiveValue;

use crate::{domain::models::Image, persistence::entities::images};

pub mod db_image_canon_updater;
pub mod db_image_getter;
pub mod db_image_saver;

fn active_model_for_insert_from(image: &Image) -> images::ActiveModel {
    images::ActiveModel {
        file_name: ActiveValue::Set(image.file_name.clone()),
        width: ActiveValue::Set(image.width as i32),
        height: ActiveValue::Set(image.height as i32),
        ..Default::default()
    }
}
