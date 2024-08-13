use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum ScreensaverEvent {
    A,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ScreensaverListenerId(u32);

pub struct ScreensaverListenerIdGenerator {
    next_id: u32,
}

impl ScreensaverListenerIdGenerator {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    pub fn gen(&mut self) -> ScreensaverListenerId {
        let id = self.next_id;
        self.next_id += 1;
        ScreensaverListenerId(id)
    }
}

pub trait ScreensaverEventSys {
    fn send(&self, event: ScreensaverEvent);

    fn register(
        &mut self,
        callback: Box<dyn Fn(ScreensaverEvent) + Send + Sync>,
    ) -> ScreensaverListenerId;

    fn unregister(&mut self, id: ScreensaverListenerId);
}
