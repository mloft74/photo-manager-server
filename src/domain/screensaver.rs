use crate::domain::models::Image;

#[derive(PartialEq, Eq, Debug)]
pub enum ResolveState {
    /// The image being resolved is not the current image of the manager.
    NotCurrent,
    /// The image was resolved.
    Resolved,
    /// The manager contains no images, so resolving cannot occur.
    NoImages,
}

pub trait Screensaver {
    /// Returns the current image if it exists.
    fn current(&self) -> Option<Image>;

    /// Resolves an image of the given name.
    /// If the name being resolved isn't the current image,
    /// or if there are no images,nothing happens.
    fn resolve(&mut self, file_name: &str) -> ResolveState;

    /// Inserts an `Image` into a random location in the internal structure.
    fn insert(&mut self, value: Image);

    /// Inserts the given `Image`s into random locations in the internal structure.
    fn insert_many<T: Iterator<Item = Image>>(&mut self, values: T);

    /// Removes all `Image`s from the internal structure.
    fn clear(&mut self);

    /// Shuffles the given `Image`s and replaces the images in the internal structure with the `Image`s.
    fn replace<T: Iterator<Item = Image>>(&mut self, values: T);
}
