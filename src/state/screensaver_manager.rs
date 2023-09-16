use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use crate::{
    domain::{
        models::Image,
        screensaver::{ResolveState, Screensaver},
    },
    state::screensaver_state::ScreensaverState,
};

#[derive(Clone)]
pub struct ScreensaverManager {
    state: Arc<Mutex<ScreensaverState>>,
}

impl ScreensaverManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ScreensaverState::new())),
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
}

impl Screensaver for ScreensaverManager {
    fn current(&self) -> Option<Image> {
        self.acquire_lock().current()
    }

    fn resolve(&mut self, file_name: &str) -> ResolveState {
        self.acquire_lock().resolve(file_name)
    }

    fn insert(&mut self, value: Image) -> Result<(), ()> {
        self.acquire_lock().insert(value)
    }

    fn insert_many(&mut self, values: HashMap<String, Image>) -> Result<(), Vec<String>> {
        self.acquire_lock().insert_many(values)
    }

    fn clear(&mut self) {
        self.acquire_lock().clear()
    }

    fn replace<T: Iterator<Item = Image>>(&mut self, values: T) {
        self.acquire_lock().replace(values)
    }
}
