use std::{
    any::type_name,
    sync::{Arc, Mutex, MutexGuard},
};

use crate::{
    domain::event_system::{ScreensaverEvent, ScreensaverEventSys, ScreensaverListenerId},
    state::screensaver_event_system::ScreensaverEventSystem,
};

#[derive(Clone)]
pub struct ScreensaverEventManager {
    s: Arc<Mutex<ScreensaverEventSystem>>,
}

impl ScreensaverEventManager {
    pub fn new() -> Self {
        Self {
            s: Arc::new(Mutex::new(ScreensaverEventSystem::new())),
        }
    }

    /// In this case, we don't care if the mutex is poisoned, as we simply hold a list of values.
    fn acquire_lock(&self) -> MutexGuard<'_, ScreensaverEventSystem> {
        match self.s.lock() {
            Ok(guard) => guard,
            Err(poison) => {
                let name = type_name::<ScreensaverEventManager>();
                tracing::debug!("Accessing poisoned {name} mutex");
                poison.into_inner()
            }
        }
    }
}

impl ScreensaverEventSys for ScreensaverEventManager {
    fn send(&self, event: ScreensaverEvent) {
        self.acquire_lock().send(event)
    }

    fn register(
        &mut self,
        callback: Box<dyn Fn(ScreensaverEvent) + Send + Sync>,
    ) -> ScreensaverListenerId {
        self.acquire_lock().register(callback)
    }

    fn unregister(&mut self, id: ScreensaverListenerId) {
        self.acquire_lock().unregister(id)
    }
}
