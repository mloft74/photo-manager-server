use std::sync::{Arc, Mutex, MutexGuard};

use crate::{
    api::routing::image::test_4k::{delta_4k, delta_resized},
    domain::{
        models::Image,
        screensaver::{ResolveTestState, TestScreensaver},
    },
};

pub struct TestScreensaverState {
    current: Image,
    next: Image,
}

impl TestScreensaverState {
    fn new() -> Self {
        Self {
            current: delta_4k(),
            next: delta_resized(),
        }
    }
}

impl TestScreensaver for TestScreensaverState {
    fn current(&self) -> Image {
        self.current.clone()
    }

    fn resolve(&mut self, file_name: &str) -> ResolveTestState {
        if self.current.file_name == file_name {
            std::mem::swap(&mut self.current, &mut self.next);
            ResolveTestState::Resolved
        } else {
            ResolveTestState::NotCurrent
        }
    }
}

#[derive(Clone)]
pub struct TestScreensaverManager {
    state: Arc<Mutex<TestScreensaverState>>,
}

impl TestScreensaverManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(TestScreensaverState::new())),
        }
    }

    /// In this case, we don't care if the mutex is poisoned, as we simply hold a list of values.
    fn acquire_lock(&self) -> MutexGuard<'_, TestScreensaverState> {
        match self.state.lock() {
            Ok(guard) => guard,
            Err(poison) => {
                tracing::debug!("Accessing poisoned mutex");
                poison.into_inner()
            }
        }
    }
}

impl TestScreensaver for TestScreensaverManager {
    fn current(&self) -> Image {
        self.acquire_lock().current()
    }

    fn resolve(&mut self, file_name: &str) -> ResolveTestState {
        self.acquire_lock().resolve(file_name)
    }
}
