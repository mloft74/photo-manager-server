use tracing::debug;

use crate::{api::canon::scaling::CanonScaledPairData, domain::models::Image};

pub fn log_images(images: &[Image]) {
    for image in images.iter() {
        debug!("found image: {}", &image.file_name);
    }
}

pub fn log_canon_scaled_pair_data(data: &CanonScaledPairData) {
    for image in data.canon_without_scale.iter() {
        debug!("found canon without scale: {}", &image.file_name);
    }
    for image in data.scale_without_canon.iter() {
        debug!("found scale without canon: {}", &image.file_name);
    }
    for pair in data.pairs.iter() {
        debug!(
            "found pair | canon: {}, scale: {}",
            &pair.canon.file_name, &pair.scale.file_name,
        );
    }
}

pub fn log_needing_new_scales(canons_needing_new_scales: &[Image]) {
    for image in canons_needing_new_scales.iter() {
        debug!("found canon needing new scale: {}", &image.file_name);
    }
}

pub fn log_invalid_scales(invalid_scales: &[Image]) {
    for image in invalid_scales.iter() {
        debug!("found invalid scale: {}", &image.file_name);
    }
}
