#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Image {
    pub file_name: String,
    pub width: u32,
    pub height: u32,
}

pub struct ImagesPage {
    pub images: Vec<Image>,
    pub cursor: Option<i32>,
}
