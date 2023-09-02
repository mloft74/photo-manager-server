use std::sync::{Arc, Mutex, MutexGuard};

use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::domain::models::Image;

#[derive(Clone)]
pub struct ScreenSaverManager {
    state: Arc<Mutex<State>>,
}

pub struct State {
    images: Vec<Image>,
    current_index: Option<usize>,
}

impl ScreenSaverManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                images: Vec::new(),
                current_index: None,
            })),
        }
    }

    /// In this case, we don't care if the mutex is poisoned, as we simply hold a list of values.
    fn acquire_lock(&self) -> MutexGuard<'_, State> {
        match self.state.lock() {
            Ok(guard) => guard,
            Err(poison) => {
                tracing::debug!("Accessing poisoned mutex");
                poison.into_inner()
            }
        }
    }

    #[deprecated = "Use current and resolve"]
    /// Removes the first element and returns it,
    /// or `None` if the internal structure is empty.
    pub fn take_next(&mut self) -> Option<Image> {
        self.acquire_lock().images.pop()
    }

    /// Inserts an `Image` into a random location in the internal structure.
    pub fn insert(&mut self, value: Image) {
        let mut images = self.acquire_lock();
        // Prevents panic from generating against an empty range.
        if images.images.is_empty() {
            images.images.push(value)
        } else {
            let length = images.images.len();
            let mut rng = thread_rng();
            let index = rng.gen_range(0..length);
            images.images.insert(index, value);
        }
    }

    /// Inserts the given `Image`s into random locations in the internal structure.
    pub fn _insert_many<T: Iterator<Item = Image>>(&mut self, values: T) {
        let mut images = self.acquire_lock();
        let mut rng = thread_rng();
        for value in values {
            // Prevents panic from generating against an empty range.
            if images.images.is_empty() {
                images.images.push(value);
            } else {
                let length = images.images.len();
                let index = rng.gen_range(0..length);
                images.images.insert(index, value);
            }
        }
    }

    /// Removes all `Image`s from the internal structure.
    pub fn _clear(&mut self) {
        self.acquire_lock().images.clear()
    }

    /// Shuffles the given `Image`s and replaces the images in the internal structure with the `Image`s.
    pub fn replace<T: Iterator<Item = Image>>(&mut self, values: T) {
        let mut state = self.acquire_lock();
        let mut rng = thread_rng();
        let mut values: Vec<_> = values.collect();
        values.shuffle(&mut rng);
        state.images = values;
    }
}
