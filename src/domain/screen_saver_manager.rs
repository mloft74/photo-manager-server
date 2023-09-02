use std::sync::{Arc, Mutex, MutexGuard};

use rand::{seq::SliceRandom, thread_rng, Rng, RngCore};

use crate::domain::models::Image;

// TODO: new impl for things:
// 1. Vec of images. You can push to the end of the list, but the list realistically shouldn't be altered, as the indices should be constant.
//    - Theoretically, you could manage the index somehow, so modifying the order of the list isn't necessarily forbidden. If you do this, make sure you properly manage indices.
//    - It's ideal if I don't update the indices at all, though, just for simplicity's sake. Avoid if possible.
// 2. Vec of indices. This is the upcoming . You can shuffle it, return the back for current, and update an index variable to move to the next image.
//    - By using an index indicator, I can see the entire list that is lined up and what has already been shown.
//    - Could keep the previous iteration as well, just for more debug information.
// 3. HashSet of indices. When resolving, this would be used to check if the index being resolved has already been resolved or not this iteration.
// 4. An iteration count. This is sent out and received when using current and resolve to ensure that what is being resolved is from the current iteration of the screensaver list.

// TODO: rename
pub enum ResolveState {}

#[derive(Clone)]
pub struct IndexedImage {
    pub index: usize,
    pub image: Image,
}

#[derive(Clone)]
pub struct ScreenSaverManager {
    state: Arc<Mutex<State>>,
}

struct State {
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

    // pub fn current(&self) -> Option<IndexedImage> {
    //     let state = self.acquire_lock();
    //     state.current_index.and_then(|idx| {
    //         state.images.get(idx).cloned().map(|img| IndexedImage {
    //             index: idx,
    //             image: img,
    //         })
    //     })
    // }

    // pub fn resolve(&mut self, file_name: &str) -> Option<ResolveState> {
    //     let state = self.acquire_lock();
    //     None
    // }

    /// Inserts an `Image` into a random location in the internal structure.
    pub fn insert(&mut self, value: Image) {
        let mut state = self.acquire_lock();
        let mut rng = thread_rng();
        insert_impl(&mut state, &mut rng, value);
    }

    /// Inserts the given `Image`s into random locations in the internal structure.
    pub fn _insert_many<T: Iterator<Item = Image>>(&mut self, values: T) {
        let mut state = self.acquire_lock();
        let mut rng = thread_rng();
        for value in values {
            insert_impl(&mut state, &mut rng, value);
        }
    }

    /// Removes all `Image`s from the internal structure.
    pub fn _clear(&mut self) {
        let mut state = self.acquire_lock();
        state.images.clear();
        state.current_index = None;
    }

    /// Shuffles the given `Image`s and replaces the images in the internal structure with the `Image`s.
    pub fn replace<T: Iterator<Item = Image>>(&mut self, values: T) {
        let mut state = self.acquire_lock();
        let mut rng = thread_rng();
        let mut values: Vec<_> = values.collect();
        values.shuffle(&mut rng);
        state.images = values;
        state.current_index = Some(0);
    }
}

fn insert_impl(state: &mut MutexGuard<'_, State>, rng: &mut impl RngCore, value: Image) {
    // Prevents panic from generating against an empty range.
    if state.images.is_empty() {
        state.images.push(value);
        state.current_index = Some(0);
    } else {
        let length = state.images.len();
        let i = state.current_index.expect("current_index should be Some");
        let index = rng.gen_range(i..length);
        state.images.insert(index, value);
    }
}
