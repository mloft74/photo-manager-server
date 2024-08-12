use std::fmt::Debug;

use dotenvy::dotenv;
use tokio::net::TcpListener;

use crate::state::screensaver_event_manager::ScreensaverEventManager;

mod api;
mod domain;
mod persistence;
mod rtc;
mod server_tracing;
mod state;

pub async fn run() {
    dotenv().expect(".env should be loadable from startup");

    let persistence_mngr = persistence::init_persistence().await;

    server_tracing::init_tracing_subscriber();

    let mut event_mngr = ScreensaverEventManager::new();

    let api_router = api::make_api_router(&persistence_mngr).await;

    let rtc_handle = rtc::init_rtc();

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("TcpListener should be valid");
    axum::serve(listener, api_router)
        .await
        .expect("Server should run without errors");
    rtc_handle.close().expect("Rtc should close without issue");
}

trait LazyExpect {
    type Value;

    fn expect_lazy(self, msg_fn: impl FnOnce() -> String) -> Self::Value;
}

impl<T, E: Debug> LazyExpect for Result<T, E> {
    type Value = T;

    fn expect_lazy(self, msg_fn: impl FnOnce() -> String) -> Self::Value {
        match self {
            Ok(v) => v,
            Err(e) => {
                let msg = msg_fn();
                panic!("{msg}: {e:?}");
            }
        }
    }
}
