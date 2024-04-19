use std::collections::HashMap;

use crate::{
    api::{
        image_ops::{compute_resize_dimensions, Dimensions},
        SCALED_IMAGE_PREFIX,
    },
    domain::models::Image,
};

pub struct CanonPartition {
    pub canon: Vec<Image>,
    pub scaled: Vec<Image>,
}

pub struct CanonScale {
    pub canon: Image,
    pub scale: Image,
}

pub struct CanonScaledPairData {
    pub pairs: Vec<CanonScale>,
    pub canon_without_scale: Vec<Image>,
    pub scale_without_canon: Vec<Image>,
}

pub fn separate_canon(images: Vec<Image>) -> CanonPartition {
    let (scaled, canon) = images
        .into_iter()
        .partition(|i| i.file_name.starts_with(SCALED_IMAGE_PREFIX));
    CanonPartition { canon, scaled }
}

pub fn pair_canon_with_scaled(partition: CanonPartition) -> CanonScaledPairData {
    let mut pairs = Vec::new();
    let mut canon_without_scale = Vec::new();
    let mut scaled: HashMap<String, Image> = partition
        .scaled
        .into_iter()
        .map(|i| (i.file_name.clone(), i))
        .collect();
    for canon in partition.canon {
        let scaled_name = format!("{}{}", SCALED_IMAGE_PREFIX, &canon.file_name);
        let existing = scaled.remove(&scaled_name);
        if let Some(scale) = existing {
            pairs.push(CanonScale { canon, scale });
        } else {
            canon_without_scale.push(canon);
        }
    }

    let scale_without_canon: Vec<_> = scaled.into_values().collect();

    CanonScaledPairData {
        pairs,
        canon_without_scale,
        scale_without_canon,
    }
}

pub fn find_invalid_pairs(images: Vec<CanonScale>) -> Vec<CanonScale> {
    images
        .into_iter()
        .filter(|i| {
            let expected = compute_resize_dimensions(Dimensions {
                width: i.canon.width,
                height: i.canon.height,
            });
            let actual = Dimensions {
                width: i.scale.width,
                height: i.scale.height,
            };
            expected != actual
        })
        .collect()
}
