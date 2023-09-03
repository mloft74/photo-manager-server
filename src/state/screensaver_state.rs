use rand::{seq::SliceRandom, thread_rng, Rng, RngCore};

use crate::domain::{
    models::Image,
    screensaver::{ResolveState, Screensaver},
};

// Invariants:
// - images and next_images always contain the same values.
// - if images is empty, current_index is None, otherwise Some.
// - if current_index is Some, the value is always within range.
// - the last image of images is not the same as the first image of next_images.
pub struct ScreensaverState {
    /// The images for the current iteration of the screensaver.
    images: Vec<Image>,
    /// The images for the next iteration of the screensaver.
    next_images: Vec<Image>,
    /// The current index of `images`.
    current_index: Option<usize>,
}

impl ScreensaverState {
    /// Create a [ScreensaverState] with no images.
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            next_images: Vec::new(),
            current_index: None,
        }
    }

    /// This inserts a single image. This could be called multiple times to insert multiple images.
    fn insert_impl(&mut self, rng: &mut impl RngCore, value: Image) {
        self.images.push(value.clone());
        self.next_images.push(value);
        match self.current_index {
            None => {
                self.current_index = Some(0);
            }
            Some(idx) => {
                // `images` swap. Uses `last_idx` because we generate a range against `idx`.
                let last_idx = self.images.len() - 1;
                // Generates when the range would be at least 2.
                if idx < last_idx {
                    let new_idx = rng.gen_range(idx..last_idx + 1);
                    self.images.swap(last_idx, new_idx);
                }

                // `next_images` swap. Uses `len` because we generate a range on the entire Vec.
                let len = self.next_images.len();
                // Generates when the range would be at least 2.
                if len > 1 {
                    let new_idx = rng.gen_range(0..len);
                    self.next_images.swap(len - 1, new_idx);
                }

                // NOTE: potential performance gain when inserting multiple images
                // by making calling code call this method.
                self.ensure_curr_end_and_next_start_are_different(rng);
            }
        }
    }

    /// This helps avoid a bug when making successive calls to resolve
    /// the same image near list ends. By moving the next start somewhere
    /// else in the list, we guarantee that you can't have the same image twice in
    /// a row, preventing a double resolve bug from a single image.
    fn ensure_curr_end_and_next_start_are_different(&mut self, rng: &mut impl RngCore) {
        let curr_end = self.images.last();
        let next_start = self.next_images.first();
        if let Some((curr_end, next_start)) = curr_end.zip(next_start) {
            let len = self.next_images.len();
            if curr_end.file_name == next_start.file_name && len > 1 {
                let new_idx = rng.gen_range(1..len);
                self.next_images.swap(0, new_idx);
            }
        }
    }
}

impl Screensaver for ScreensaverState {
    fn current(&self) -> Option<Image> {
        self.current_index.map(|idx| self.images[idx].clone())
    }

    fn next(&self) -> Option<Image> {
        self.current_index.map(|idx| {
            let next_idx = idx + 1;
            let len = self.images.len();
            if next_idx < len {
                self.images[next_idx].clone()
            } else {
                let next_idx = len - next_idx;
                self.next_images[next_idx].clone()
            }
        })
    }

    fn resolve(&mut self, file_name: &str) -> ResolveState {
        match self.current_index {
            None => ResolveState::NoImages,
            Some(idx) => {
                let len = self.images.len();
                let img = &self.images[idx];
                if img.file_name != file_name {
                    ResolveState::NotCurrent
                } else {
                    let new_idx = idx + 1;
                    if new_idx < len {
                        self.current_index = Some(new_idx);
                    } else {
                        std::mem::swap(&mut self.images, &mut self.next_images);
                        self.current_index = Some(0);

                        let mut rng = thread_rng();
                        self.next_images.shuffle(&mut rng);
                        self.ensure_curr_end_and_next_start_are_different(&mut rng);
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
        self.next_images.clear();
        self.current_index = None;
    }

    fn replace<T: Iterator<Item = Image>>(&mut self, values: T) {
        let mut rng = thread_rng();
        let mut values: Vec<_> = values.collect();
        let mut next_values = values.clone();
        values.shuffle(&mut rng);
        next_values.shuffle(&mut rng);

        self.images = values;
        self.next_images = next_values;
        self.current_index = Some(0);
        self.ensure_curr_end_and_next_start_are_different(&mut rng);
    }
}
