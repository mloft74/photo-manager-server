use std::collections::{HashMap, HashSet};

use rand::{seq::SliceRandom, thread_rng, Rng, RngCore};

use crate::domain::{
    models::Image,
    screensaver::{ResolveState, Screensaver},
};

// Invariants:
// - If `images` is empty, `current_index` is `None`, otherwise `Some`.
// - If `current_index` is `Some`, the value is always within range.
// - The last image of the current iteration of `images` is not the first image of the next iteration of `images`.
//   - Only applies when more than 1 image is held.
pub struct ScreensaverState {
    /// The images for the current iteration of the screensaver.
    images: Vec<Image>,
    /// The current index of `images`.
    current_index: Option<usize>,
}

impl ScreensaverState {
    /// Create a [ScreensaverState] with no images.
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            current_index: None,
        }
    }

    /// This inserts a single image. This could be called multiple times to insert multiple images.
    fn insert_impl(&mut self, rng: &mut impl RngCore, value: Image) {
        self.images.push(value);
        match self.current_index {
            None => {
                self.current_index = Some(0);
            }
            Some(idx) => {
                let len = self.images.len();
                let last_idx = len - 1;
                // The images at idx and before need to be stable.
                let x = idx + 1;
                // Generate on non-empty range.
                if x < last_idx {
                    let new_idx = rng.gen_range(x..len);
                    self.images.swap(last_idx, new_idx);
                }
            }
        }
    }
}

impl Screensaver for ScreensaverState {
    fn current(&self) -> Option<Image> {
        self.current_index.map(|idx| self.images[idx].clone())
    }

    fn resolve(&mut self, file_name: &str) -> ResolveState {
        match self.current_index {
            None => ResolveState::NoImages,
            Some(idx) => {
                let len = self.images.len();
                let curr_name = self.images[idx].file_name.clone();
                if curr_name != file_name {
                    ResolveState::NotCurrent
                } else {
                    let new_idx = idx + 1;
                    if new_idx < len {
                        self.current_index = Some(new_idx);
                    } else {
                        self.current_index = Some(0);

                        let mut rng = thread_rng();
                        self.images.shuffle(&mut rng);

                        ensure_different_next_image(&curr_name, &mut self.images, &mut rng);
                    }

                    ResolveState::Resolved
                }
            }
        }
    }

    fn insert(&mut self, value: Image) -> Result<(), ()> {
        if self.images.iter().any(|x| x.file_name == value.file_name) {
            Err(())
        } else {
            let mut rng = thread_rng();
            self.insert_impl(&mut rng, value);
            Ok(())
        }
    }

    fn insert_many(&mut self, values: HashMap<String, Image>) -> Result<(), Vec<String>> {
        let names: HashSet<_> = values.keys().collect();
        let conflicts = self.images.iter().fold(vec![], |mut acc, img| {
            if names.iter().any(|n| **n == img.file_name) {
                acc.push(img.file_name.clone());
            }
            acc
        });
        if conflicts.is_empty() {
            let mut rng = thread_rng();
            for value in values.into_values() {
                self.insert_impl(&mut rng, value);
            }

            Ok(())
        } else {
            Err(conflicts)
        }
    }

    fn clear(&mut self) {
        self.images.clear();
        self.current_index = None;
    }

    fn replace<T: Iterator<Item = Image>>(&mut self, values: T) {
        let mut values: Vec<_> = values.collect();
        if values.is_empty() {
            self.current_index = None;
        } else {
            let curr_name = self.current().map(|e| e.file_name);
            self.current_index = Some(0);

            let mut rng = thread_rng();
            values.shuffle(&mut rng);

            if let Some(curr_name) = curr_name {
                ensure_different_next_image(&curr_name, &mut values, &mut rng);
            }
        }
        self.images = values;
    }
}

// This helps avoid a bug when making successive calls to resolve
// the same image at the list end. By moving the next start somewhere
// else in the list, we guarantee that you can't have the same image twice in
// a row, preventing a double resolve bug from a single image.
// Also, generate on non-empty range.
fn ensure_different_next_image(curr_name: &str, images: &mut [Image], rng: &mut impl RngCore) {
    let new_name = &images[0].file_name;
    let len = images.len();
    let first_swap_idx = 1;
    if curr_name == new_name && len > first_swap_idx {
        tracing::debug!("swapping");
        let new_idx = rng.gen_range(first_swap_idx..len);
        images.swap(0, new_idx);
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::*;

    /// Exists to easily hide the impl behind the trait,
    /// forcing tests to only test the public api.
    fn mk_sut() -> impl Screensaver {
        ScreensaverState::new()
    }

    fn mk_img(x: u32) -> Image {
        Image {
            file_name: format!("test {}", x),
            width: x,
            height: x,
        }
    }

    fn mk_imgs(r: Range<u32>) -> HashMap<String, Image> {
        r.map(|i| {
            let img = mk_img(i);
            (img.file_name.clone(), img)
        })
        .collect()
    }

    #[test]
    fn current_is_none_when_created() {
        // Arrange
        let sut = mk_sut();

        // Assert
        assert!(sut.current().is_none());
    }

    #[test]
    fn current_is_some_after_insert() {
        // Arrange
        let mut sut = mk_sut();
        let img = mk_img(1);

        // Act
        sut.insert(img.clone())
            .expect("sut should not already have img");

        // Assert
        let curr = sut.current().expect("curr should have been inserted");
        assert_eq!(curr, img);
    }

    #[test]
    fn current_is_some_after_non_empty_replace() {
        // Arrange
        let mut sut = mk_sut();
        let img = mk_img(1);

        // Act
        sut.replace(vec![img.clone()].into_iter());

        // Assert
        let curr = sut.current().expect("curr should have been inserted");
        assert_eq!(curr, img);
    }

    #[test]
    fn current_is_none_after_empty_replace() {
        // Arrange
        let mut sut = mk_sut();

        // Act
        sut.insert(mk_img(1))
            .expect("sut should not already have the inserted image");
        sut.replace(vec![].into_iter());

        // Assert
        assert!(sut.current().is_none());
    }

    #[test]
    fn current_is_none_after_clear() {
        // Arrange
        let mut sut = mk_sut();

        // Act
        sut.insert(mk_img(1))
            .expect("sut should not already have the inserted image");
        sut.clear();

        // Assert
        assert!(sut.current().is_none());
    }

    #[test]
    fn current_is_same_from_multiple_current_calls() {
        // Arrange
        let mut sut = mk_sut();

        // Act
        // Inserting mutliple images logically allows for multiple currents if sut works incorrectly.
        sut.insert(mk_img(1))
            .expect("sut should not already have the inserted image");
        sut.insert(mk_img(2))
            .expect("sut should not already have the inserted image");

        // Assert
        let a = sut.current().expect("image should have been inserted");
        let b = sut.current().expect("image should have been inserted");
        assert_eq!(a, b);
    }

    #[test]
    fn current_is_same_from_multiple_current_and_insert_calls() {
        // Arrange
        let mut sut = mk_sut();

        // Act
        let max = 11;
        for x in 1..max {
            sut.insert(mk_img(x))
                .expect("sut should not already have the inserted image");
        }
        let a = sut.current().expect("image should have been inserted");
        sut.insert(mk_img(max))
            .expect("sut should not already have the inserted image");

        // Assert
        let b = sut.current().expect("image should have been inserted");
        assert_eq!(a, b);
    }

    #[test]
    fn current_is_same_from_multiple_current_and_insert_many_calls() {
        // Arrange
        let mut sut = mk_sut();

        // Act
        sut.insert_many(mk_imgs(1..11))
            .expect("sut should not already have the inserted images");
        let a = sut.current().expect("image should have been inserted");
        sut.insert_many(mk_imgs(11..15))
            .expect("sut should not already have the inserted images");

        // Assert
        let b = sut.current().expect("image should have been inserted");
        assert_eq!(a, b);
    }

    #[test]
    fn current_is_different_after_replace() {
        // Arrange
        let mut sut = mk_sut();
        let min = 1;
        let max = 3;
        let imgs = (min..max).map(mk_img);

        // Act
        sut.replace(imgs.clone());
        for _ in min..(max - 1) {
            sut.resolve(
                &sut.current()
                    .expect("there should still be a current available")
                    .file_name,
            );
        }
        let a = sut
            .current()
            .expect("there should still be a current available");
        sut.replace(imgs);

        // Assert
        let b = sut.current().expect("replace should have added images");
        assert_ne!(a, b);
    }

    #[test]
    fn current_is_different_after_fully_resolved() {
        // Arrange
        let mut sut = mk_sut();
        let min = 1;
        let max = 3;
        let imgs = (min..max).map(mk_img);

        // Act
        sut.replace(imgs.clone());
        for _ in min..(max - 1) {
            sut.resolve(
                &sut.current()
                    .expect("there should still be a current available")
                    .file_name,
            );
        }
        let a = sut
            .current()
            .expect("there should still be a current available");
        sut.resolve(&a.file_name);

        // Assert
        let b = sut.current().expect("replace should have added images");
        assert_ne!(a, b);
    }

    #[test]
    fn can_insert_more_than_2_images() {
        // Arrange
        let mut sut = mk_sut();

        // Act
        for x in 1..11 {
            sut.insert(mk_img(x))
                .expect("sut should not already have the inserted image");
        }

        // No Assert, testing for panic above.
    }

    #[test]
    fn can_insert_many_more_than_2_images() {
        // Arrange
        let mut sut = mk_sut();

        // Act
        sut.insert_many(mk_imgs(1..11))
            .expect("sut should not already have the inserted images");

        // No Assert, testing for panic above.
    }

    #[test]
    fn resolve_no_images() {
        // Arrange
        let mut sut = mk_sut();

        // Act
        let res = sut.resolve("does not exist");

        // Assert
        assert_eq!(res, ResolveState::NoImages);
    }

    #[test]
    fn resolve_not_current() {
        // Arrange
        let mut sut = mk_sut();

        // Act
        sut.insert(mk_img(1))
            .expect("sut should not already have the inserted image");
        let res = sut.resolve("does not exist");

        // Assert
        assert_eq!(res, ResolveState::NotCurrent);
    }

    #[test]
    fn resolve_resolved() {
        // Arrange
        let mut sut = mk_sut();
        let img = mk_img(1);

        // Act
        sut.insert(img.clone())
            .expect("sut should not already have img");
        let res = sut.resolve(&img.file_name);

        // Assert
        assert_eq!(res, ResolveState::Resolved);
    }
}
