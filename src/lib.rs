use std::{
    fmt::Debug,
    io::{self, Read, Write},
    sync::mpsc::{self, TryRecvError},
    thread,
    time::Duration,
};

use dotenvy::dotenv;
use tokio::net::TcpListener;
use tracing::{debug, error, warn};

mod api;
mod domain;
mod persistence;
mod server_tracing;
mod state;

pub async fn run() {
    dotenv().expect(".env should be loadable from startup");

    let persistence_mngr = persistence::init_persistence().await;

    server_tracing::init_tracing_subscriber();

    let api_router = api::make_api_router(&persistence_mngr).await;

    // TODO: move thread code into a new location

    // TODO: manually test this code

    let (listen_stop_send, listen_stop_recv) = mpsc::channel();
    let (stream_stop_send, stream_stop_recv) = mpsc::channel();
    let (stream_send, stream_recv) = mpsc::channel();

    let listen_handler = thread::spawn(move || {
        let listener =
            std::net::TcpListener::bind("0.0.0.0:4000").expect("TcpListener should be valid");
        listener
            .set_nonblocking(true)
            .expect("Should be able to set listener as non-blocking");
        debug!("now listening");
        loop {
            let should_stop = listen_stop_recv.try_recv();
            match should_stop {
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    warn!("listen_stop_recv disconnected, stopping thread");
                    break;
                }
                Ok(_) => {
                    debug!("listen received stop signal, stopping thread");
                    break;
                }
            }
            let res = listener.accept();
            match res {
                Ok((stream, addr)) => {
                    debug!("established connection with {addr}");
                    let send_res = stream_send.send((stream, addr));
                    if let Err(err) = send_res {
                        error!("could not send stream due to problem: {err}");
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
                Err(e) => {
                    error!("encountered error accepting tcp connection: {e}");
                }
            }

            thread::sleep(Duration::from_millis(100));
        }
    });

    let stream_handler = thread::spawn(move || {
        let mut streams: Vec<(std::net::TcpStream, std::net::SocketAddr)> = Vec::new();
        loop {
            debug!("starting stream loop");
            let should_stop = stream_stop_recv.try_recv();
            match should_stop {
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    warn!("listen_stop_recv disconnected, stopping thread");
                    for (stream, addr) in streams.iter() {
                        let res = stream.shutdown(std::net::Shutdown::Both);
                        if let Err(err) = res {
                            error!("could not shutdown {addr} because of error: {err}");
                        }
                    }
                    break;
                }
                Ok(_) => {
                    debug!("listen received stop signal, stopping thread");
                    for (stream, addr) in streams.iter() {
                        let res = stream.shutdown(std::net::Shutdown::Both);
                        if let Err(err) = res {
                            error!("could not shutdown {addr} because of error: {err}");
                        }
                    }
                    break;
                }
            }
            let new_stream = stream_recv.try_recv();
            match new_stream {
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    warn!("stream_recv disconnected, stopping thread");
                    for (stream, addr) in streams.iter() {
                        let res = stream.shutdown(std::net::Shutdown::Both);
                        if let Err(err) = res {
                            error!("could not shutdown {addr} because of error: {err}");
                        }
                    }
                    break;
                }
                Ok((stream, addr)) => {
                    stream.set_nonblocking(false).expect_lazy(|| {
                        format!("should be able to set stream as blocking | addr: {addr}")
                    });
                    streams.push((stream, addr));
                }
            }
            let mut to_remove = Vec::new();
            for (idx, (stream, addr)) in streams.iter_mut().enumerate() {
                let mut string = String::new();
                loop {
                    let mut buf = Vec::with_capacity(512);
                    let res = stream.read(&mut buf);
                    match res {
                        Ok(0) => {
                            debug!("got no bytes");
                            break;
                        }
                        Ok(x) => {
                            debug!("got {x} bytes");
                            let bytes = &buf[0..x];
                            let x = std::str::from_utf8(bytes);
                            match x {
                                Ok(val) => string.push_str(val),
                                Err(err) => error!("could not convert bytes to string | err: {err}, bytes: {bytes:?}"),
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            debug!("would block, breaking loop");
                            break;
                        }
                        Err(e) => {
                            error!("encountered error reading from tcp connection {addr}: {e}");
                            to_remove.push(idx);
                            break;
                        }
                    }
                }
                if !string.is_empty() {
                    debug!("got msg from {addr}: {string}");
                    let msg = format!("sending message to {addr}");
                    let res = stream.write_all(msg.as_bytes());
                    if let Err(err) = res {
                        error!("error sending message to {addr}: {err}");
                    }
                }
            }
            for idx in to_remove.into_iter().rev() {
                let (stream, addr) = streams.remove(idx);
                debug!("closing {addr}");
                let res = stream.shutdown(std::net::Shutdown::Both);
                if let Err(err) = res {
                    error!("could not shutdown {addr} because of error: {err}");
                }
            }

            thread::sleep(Duration::from_millis(100));
        }
    });

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("TcpListener should be valid");
    axum::serve(listener, api_router)
        .await
        .expect("Server should run without errors");
    listen_stop_send
        .send(())
        .expect("Should be able to send stop signal to listen");
    stream_stop_send
        .send(())
        .expect("Should be able to send stop signal to stream");
    listen_handler
        .join()
        .expect("Listen thread should not panic");
    stream_handler
        .join()
        .expect("Steam thread should not panic");
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
