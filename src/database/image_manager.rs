use crate::domain::models::Image;

#[derive(Clone)]
pub struct ImageManager {}

// Hide this behind Traits and use generics to pass it around.
impl ImageManager {
    pub async fn get_image() -> Result<Image, Box<dyn std::error::Error>> {
        todo!()
    }

    pub async fn save_image(&self, image: &Image) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}
