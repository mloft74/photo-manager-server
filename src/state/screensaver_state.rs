use rand::{seq::SliceRandom, thread_rng, Rng, RngCore};

use crate::domain::{
    models::Image,
    screensaver::{ResolveState, Screensaver},
};

// Invariants:
// - if images is empty, current_index is None, otherwise Some.
// - if current_index is Some, the value is always within range.
// - the last image of the current iteration of images is not the first image of the next iteration of images.
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
                // The images at idx and before need to be stable.
                let x = idx + 1;
                // Generate on non-empty range.
                if x < len - 1 {
                    let new_idx = rng.gen_range(x..len);
                    self.images.swap(len, new_idx);
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

                        // This helps avoid a bug when making successive calls to resolve
                        // the same image at the list end. By moving the next start somewhere
                        // else in the list, we guarantee that you can't have the same image twice in
                        // a row, preventing a double resolve bug from a single image.
                        // Also, generate on non-empty range.
                        let new_name = &self.images[0].file_name;
                        if curr_name == *new_name && len > 1 {
                            tracing::debug!("swapping");
                            let new_idx = rng.gen_range(1..len);
                            self.images.swap(0, new_idx);
                        }
                    }

                    ResolveState::Resolved
                }
            }
        }
    }

    fn insert(&mut self, value: Image) {
        let mut rng = thread_rng();
        self.insert_impl(&mut rng, value)
    }

    fn insert_many<T: Iterator<Item = Image>>(&mut self, values: T) {
        let mut rng = thread_rng();
        for value in values {
            self.insert_impl(&mut rng, value);
        }
    }

    fn clear(&mut self) {
        self.images.clear();
        self.current_index = None;
    }

    // TODO: Check length of images we were given
    // TODO: Verify that the current image is not the next image
    fn replace<T: Iterator<Item = Image>>(&mut self, values: T) {
        let mut rng = thread_rng();
        let mut values: Vec<_> = values.collect();
        values.shuffle(&mut rng);

        self.images = values;
        self.current_index = Some(0);
    }
}
