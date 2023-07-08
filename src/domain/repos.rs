use crate::domain::repos::images::ImageRepo;

pub mod images;

pub trait RepoProvider {
    type ImageRepoImpl: ImageRepo;
    fn get_image_repo(&self) -> Self::ImageRepoImpl;
}
