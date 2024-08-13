use std::collections::HashMap;

use crate::domain::event_system::{
    ScreensaverEvent, ScreensaverEventSys, ScreensaverListenerId, ScreensaverListenerIdGenerator,
};

pub struct ScreensaverEventSystem {
    id_gen: ScreensaverListenerIdGenerator,
    callbacks: HashMap<ScreensaverListenerId, Box<dyn Fn(ScreensaverEvent) + Send + Sync>>,
}

impl ScreensaverEventSystem {
    pub fn new() -> Self {
        Self {
            id_gen: ScreensaverListenerIdGenerator::new(),
            callbacks: HashMap::new(),
        }
    }
}

impl ScreensaverEventSys for ScreensaverEventSystem {
    fn send(&self, event: ScreensaverEvent) {
        for cb in self.callbacks.values() {
            cb(event.clone());
        }
    }

    fn register(
        &mut self,
        callback: Box<dyn Fn(ScreensaverEvent) + Send + Sync>,
    ) -> ScreensaverListenerId {
        let id = self.id_gen.gen();
        self.callbacks.insert(id, callback);

        id
    }

    fn unregister(&mut self, id: ScreensaverListenerId) {
        _ = self.callbacks.remove(&id);
    }
}
