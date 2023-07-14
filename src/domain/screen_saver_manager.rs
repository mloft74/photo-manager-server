use std::sync::{Arc, Mutex, MutexGuard};

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

    /// In this case, we don't care if the mutex is poisoned, as we simply hold a list of values.
    fn acquire_lock(&self) -> MutexGuard<'_, Vec<Image>> {
        match self.images.lock() {
            Ok(guard) => guard,
            Err(poison) => {
                tracing::debug!("Accessing poisoned mutex");
                poison.into_inner()
            }
        }
    }

    /// Removes the first element and returns it,
    /// or `None` if the internal structure is empty.
    pub fn take_next(&mut self) -> Option<Image> {
        self.acquire_lock().pop()
    }

    /// Inserts an `Image` into a random location in the internal structure.
    pub fn insert(&mut self, value: Image) {
        let mut images = self.acquire_lock();
        // Prevents panic from generating against an empty range.
        if images.is_empty() {
            images.push(value)
        } else {
            let length = images.len();
            let mut rng = thread_rng();
            let index = rng.gen_range(0..length);
            images.insert(index, value);
        }
    }

    /// Inserts the given `Image`s into random locations in the internal structure.
    pub fn _insert_many<T: Iterator<Item = Image>>(&mut self, values: T) {
        let mut images = self.acquire_lock();
        let mut rng = thread_rng();
        for value in values {
            // Prevents panic from generating against an empty range.
            if images.is_empty() {
                images.push(value);
            } else {
                let length = images.len();
                let index = rng.gen_range(0..length);
                images.insert(index, value);
            }
        }
    }

    /// Removes all `Image`s from the internal structure.
    pub fn _clear(&mut self) {
        self.acquire_lock().clear()
    }

    /// Shuffles the given `Image`s and replaces the images in the internal structure with the `Image`s.
    pub fn replace<T: Iterator<Item = Image>>(&mut self, values: T) {
        let mut images = self.acquire_lock();
        let mut rng = thread_rng();
        let mut values: Vec<_> = values.collect();
        values.shuffle(&mut rng);
        *images = values;
    }
}
