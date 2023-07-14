use std::sync::{Arc, Mutex};

use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::domain::models::Image;

#[derive(Clone)]
pub struct ScreenSaverManager {
    images: Arc<Mutex<Vec<Image>>>,
}

impl ScreenSaverManager {
    pub fn new() -> Self {
        Self {
            images: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Removes the first element and returns it,
    /// or `None` if the internal structure is empty.
    pub fn take_next(&mut self) -> Option<Image> {
        self.images
            .lock()
            .expect("Problem acquiring lock in take_next")
            .pop()
    }

    /// Inserts an `Image` into a random location in the internal structure.
    pub fn insert(&mut self, value: Image) {
        let mut images = self
            .images
            .lock()
            .expect("Problem acquiring lock in insert");
        let length = images.len();
        let mut rng = thread_rng();
        let index = rng.gen_range(0..length);
        images.insert(index, value);
    }

    /// Inserts the given `Image`s into random locations in the internal structure.
    pub fn insert_many<T: Iterator<Item = Image>>(&mut self, values: T) {
        let mut images = self
            .images
            .lock()
            .expect("Problem acquiring lock in insert_many");
        let mut rng = thread_rng();
        for value in values {
            let length = images.len();
            let index = rng.gen_range(0..length);
            images.insert(index, value);
        }
    }

    /// Removes all `Image`s from the internal structure.
    pub fn clear(&mut self) {
        self.images
            .lock()
            .expect("Problem acquiring lock in clear")
            .clear()
    }

    /// Shuffles the given `Image`s and replaces the images in the internal structure with the `Image`s.
    pub fn replace<T: Iterator<Item = Image>>(&mut self, values: T) {
        let mut images = self
            .images
            .lock()
            .expect("Problem acquiring lock in replace");
        let mut rng = thread_rng();
        let mut values: Vec<_> = values.collect();
        values.shuffle(&mut rng);
        *images = values.into_iter().collect();
    }
}
