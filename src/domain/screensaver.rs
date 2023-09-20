use std::collections::HashMap;

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

// Invariants:
// - Implementors contain no duplicate images according to the image file name.
// - Implementors never return the same image from `current` across calls to `resolve`.
//   - E.G. `current` returns an image named "first", then "first" is resolved,
//     implementors can not then return "first" from `current` until another
//     image is successfully resolved.
//   - An exception is made when implementors can only return a single image.
pub trait Screensaver {
    /// Returns the current image if it exists.
    fn current(&self) -> Option<Image>;

    /// Resolves an image of the given name.
    /// If the name being resolved isn't the current image,
    /// or if there are no images, nothing happens.
    fn resolve(&mut self, file_name: &str) -> ResolveState;

    /// Inserts an `Image` into a random location in the internal structure.
    /// Returns `Err` if the screensaver already contains an image with the same name.
    /// If `Err`, no modifications were made to the internals.
    fn insert(&mut self, value: Image) -> Result<(), ()>;

    /// Inserts the given `Image`s into random locations in the internal structure.
    /// Returns `Err` with any image names that are already contained.
    /// If `Err`, no modifications were made to the internals.
    /// The key should be the file name of the image the key refers to.
    fn insert_many(&mut self, values: HashMap<String, Image>) -> Result<(), Vec<String>>;

    /// Renames an image.
    /// Returns an `Err` if the `old_name` is not found.
    fn rename_image(&mut self, old_name: &str, new_name: &str) -> Result<(), ()>;

    /// Removes an image.
    /// Returns an `Err` if the `file_name` is not found.
    fn delete_image(&mut self, file_name: &str) -> Result<(), ()>;

    /// Removes all images from the internal structure.
    fn clear(&mut self);

    /// Shuffles the given `Image`s and replaces the images in the internal structure.
    /// The key should be the file name of the image the key refers to.
    fn replace(&mut self, values: HashMap<String, Image>);
}
