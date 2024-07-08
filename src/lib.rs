use std::{fmt::Debug, sync::mpsc};

use dotenvy::dotenv;

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

    let api_router = api::make_api_router(&persistence_mngr).await;

    let (send_stop_listen, recv_stop_listen) = mpsc::channel();

    let listen_handler = rtc::listener_thread(recv_stop_listen);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("TcpListener should be valid");
    axum::serve(listener, api_router)
        .await
        .expect("Server should run without errors");
    send_stop_listen
        .send(())
        .expect("Should be able to send stop signal to listen");
    listen_handler
        .join()
        .expect("Listen thread should not panic");
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
