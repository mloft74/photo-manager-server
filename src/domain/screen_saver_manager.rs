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
//    - Could also use a Vec of enum variants: Resolved | Ready. Vec access is constant, and probably simpler than a HashSet.
// 4. An iteration count. This is sent out and received when using current and resolve to ensure that what is being resolved is from the current iteration of the screensaver list.

// TODO: Another idea that is probably much simpler:
// 1. Vec of Images * index variable. Everything before the index is used, everything after isn't.
// 2. When reshuffling, just check and see what the current one is. If the current one is the same as what the new current one will be, swap it randomly.
// 3. When inserting, insert in the back, then just swap it in somewhere randomly. The order of what is next is not important.
//    - I think a screensaver iteration count could fix this as well. You get what iteration an image is part of, and then you pass it it. If you are resolving the wrong iteration, nothing happens.

pub enum ResolveState {
    /// The image being resolved is not the current image of the manager.
    NotCurrent,
    /// The image was resolved.
    Resolved,
    /// The manager contains no images, so resolving cannot occur.
    NoImages,
}

#[derive(Clone)]
pub struct ScreenSaverManager {
    state: Arc<Mutex<ScreensaverState>>,
}

struct ScreensaverState {
    images: Vec<Image>,
    next_images: Vec<Image>,
    current_index: Option<usize>,
}

impl ScreensaverState {
    fn swap_images(&mut self) {
        std::mem::swap(&mut self.images, &mut self.next_images);
    }
}

impl ScreenSaverManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ScreensaverState {
                images: Vec::new(),
                next_images: Vec::new(),
                current_index: None,
            })),
        }
    }

    /// In this case, we don't care if the mutex is poisoned, as we simply hold a list of values.
    fn acquire_lock(&self) -> MutexGuard<'_, ScreensaverState> {
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

    // TODO: change to result type because idx could be out of range
    // TODO: doc
    pub fn current(&self) -> Option<Image> {
        let state = self.acquire_lock();
        state
            .current_index
            .and_then(|idx| state.images.get(idx).cloned())
    }

    // TODO: doc
    pub fn resolve(&mut self, file_name: &str) -> ResolveState {
        let mut state = self.acquire_lock();
        if let Some(idx) = state.current_index {
            let len = state.images.len();
            let img = &state.images[idx];
            if img.file_name == file_name {
                let new_idx = idx + 1;
                if new_idx >= len {
                    state.current_index = Some(0);
                    state.swap_images();
                    let mut rng = thread_rng();
                    state.next_images.shuffle(&mut rng);
                    // TODO: check for equality on last of current and first of next
                } else {
                    state.current_index = Some(new_idx);
                }
                ResolveState::Resolved
            } else {
                ResolveState::NotCurrent
            }
        } else {
            ResolveState::NoImages
        }
    }

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
        let mut rng = thread_rng();
        let mut values: Vec<_> = values.collect();
        values.shuffle(&mut rng);
        let new_img = values.first();

        let mut state = self.acquire_lock();
        let old_img = state.images.last();

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

        state.images = values;
        state.current_index = Some(0);
    }
}

// TODO: insert on both lists and verify last of current does not equal first of next
fn insert_impl(state: &mut MutexGuard<'_, ScreensaverState>, rng: &mut impl RngCore, value: Image) {
    if let Some(idx) = state.current_index {
        let len = state.images.len();
        // Prevents panic from generating against an empty range.
        if idx < len {
            state.images.push(value);
            // Generate `new_idx` after `push` as the last position is also valid.
            let new_idx = rng.gen_range(idx..state.images.len());
            // `len` is guaranteed to point at the last position after a single `push`.
            state.images.swap(len, new_idx);
        } else {
            state.images.push(value);
        }
    } else {
        state.images.push(value);
        state.current_index = Some(0);
    }
}
