use rand::{seq::SliceRandom, thread_rng, Rng, RngCore};

use crate::domain::{
    models::Image,
    screensaver::{ResolveState, Screensaver},
};

pub struct ScreensaverState {
    images: Vec<Image>,
    next_images: Vec<Image>,
    current_index: Option<usize>,
}

impl ScreensaverState {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            next_images: Vec::new(),
            current_index: None,
        }
    }

    fn insert_impl(&mut self, rng: &mut impl RngCore, value: Image) {
        if let Some(idx) = self.current_index {
            let len = self.images.len();
            // Prevents panic from generating against an empty range.
            if idx < len {
                self.images.push(value);
                // Generate `new_idx` after `push` as the last position is also valid.
                let new_idx = rng.gen_range(idx..self.images.len());
                // `len` is guaranteed to point at the last position after a single `push`.
                self.images.swap(len, new_idx);
            } else {
                self.images.push(value);
            }
        } else {
            self.images.push(value);
            self.current_index = Some(0);
        }
    }
}

impl Screensaver for ScreensaverState {
    fn current(&self) -> Option<Image> {
        self.current_index.map(|idx| self.images[idx].clone())
    }

    fn resolve(&mut self, file_name: &str) -> ResolveState {
        if let Some(idx) = self.current_index {
            let len = self.images.len();
            let img = &self.images[idx];
            if img.file_name == file_name {
                let new_idx = idx + 1;
                if new_idx >= len {
                    self.current_index = Some(0);
                    std::mem::swap(&mut self.images, &mut self.next_images);
                    let mut rng = thread_rng();
                    self.next_images.shuffle(&mut rng);
                    // TODO: check for equality on last of current and first of next
                } else {
                    self.current_index = Some(new_idx);
                }
                ResolveState::Resolved
            } else {
                ResolveState::NotCurrent
            }
        } else {
            ResolveState::NoImages
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

    fn replace<T: Iterator<Item = Image>>(&mut self, values: T) {
        let old_img = self.images.last();

        let mut rng = thread_rng();
        let mut values: Vec<_> = values.collect();
        values.shuffle(&mut rng);
        let new_img = values.first();

        // This section helps avoid a scenario where 2 clients could try to resolve
        // the same image across a replace operation. By moving the new start somewhere
        // else in the list, we guarantee that you can't have the same image twice in
        // a row, preventing a double resolve bug from a single image.
        if let Some((old_img, new_img)) = old_img.zip(new_img) {
            let len = values.len();
            if old_img.file_name == new_img.file_name && len >= 2 {
                let new_idx = rng.gen_range(1..len);
                values.swap(0, new_idx);
            }
        }

        self.images = values;
        self.current_index = Some(0);
    }
}
