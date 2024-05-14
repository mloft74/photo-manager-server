use std::{
    fmt::Debug,
    io::{self, Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread::{self, JoinHandle},
    time::Duration,
};

use dotenvy::dotenv;
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

    let (send_stop_listen, recv_stop_listen) = mpsc::channel();

    let listen_handler = listener_thread(recv_stop_listen);

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

fn listener_thread(recv_stop_listen: Receiver<()>) -> JoinHandle<()> {
    thread::spawn(move || {
        let listener = TcpListener::bind("0.0.0.0:4000").expect("TcpListener should be valid");
        listener
            .set_nonblocking(true)
            .expect("Should be able to set listener as non-blocking");
        let mut stream_threads = Vec::new();
        debug!("now listening");
        loop {
            let should_stop = recv_stop_listen.try_recv();
            match should_stop {
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    warn!("recv_stop_listen disconnected, stopping thread");
                    break;
                }
                Ok(_) => {
                    debug!("recv_stop_listen received stop signal, stopping thread");
                    break;
                }
            }
            let res = listener.accept();
            match res {
                Ok((stream, addr)) => {
                    debug!("established connection with {addr}");
                    let (send_stop, recv_stop) = mpsc::channel();
                    let thread = stream_thread(recv_stop, stream, addr);
                    stream_threads.push((send_stop, addr, thread));
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
                Err(e) => {
                    error!("encountered error accepting tcp connection: {e}");
                }
            }

            let mut finished_idxs = Vec::new();
            for (idx, (_, _, thread)) in stream_threads.iter().enumerate() {
                if thread.is_finished() {
                    finished_idxs.push(idx);
                }
            }
            for idx in finished_idxs {
                let (_, addr, thread) = stream_threads.remove(idx);
                let res = thread.join();
                if let Err(err) = res {
                    error!("peer thread {addr} failed to join: {err:?}");
                }
            }

            thread::sleep(Duration::from_millis(100));
        }

        let (send_stops, threads) =
            stream_threads
                .into_iter()
                .fold((Vec::new(), Vec::new()), |mut acc, element| {
                    let (send_stop, addr, thread) = element;
                    acc.0.push((send_stop, addr));
                    acc.1.push((thread, addr));
                    acc
                });
        for (send_stop, addr) in send_stops {
            let res = send_stop.send(());
            if let Err(err) = res {
                error!("failed to send stop signal to thread for {addr}: {err}");
            }
        }
        for (thread, addr) in threads {
            let res = thread.join();
            if let Err(err) = res {
                error!("failed to join thread for {addr}: {err:?}");
            }
        }
    })
}

fn stream_thread(
    recv_stop: Receiver<()>,
    mut stream: TcpStream,
    addr: SocketAddr,
) -> JoinHandle<()> {
    thread::spawn(move || {
        stream
            .set_nonblocking(true)
            .expect("Should be able to set stream as non-blocking");
        debug!("[{addr}] now handling stream");
        'thread: loop {
            let should_stop = recv_stop.try_recv();
            match should_stop {
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    warn!("[{addr}] recv_stop disconnected, stopping thread");
                    break;
                }
                Ok(_) => {
                    debug!("[{addr}] recv_stop received stop signal, stopping thread");
                    break;
                }
            }

            let mut string = String::new();
            loop {
                let mut buf = Vec::with_capacity(512);
                let res = stream.read(&mut buf);
                match res {
                    Ok(0) => {
                        debug!("[{addr}] got no bytes | buf: {buf:?}");
                        break;
                    }
                    Ok(x) => {
                        let bytes = &buf[0..x];
                        debug!("[{addr}] got {x} bytes: {bytes:?}");
                        let x = std::str::from_utf8(bytes);
                        match x {
                            Ok(val) => string.push_str(val),
                            Err(err) => error!(
                                "[{addr}] could not convert bytes to string | err: {err}, bytes: {bytes:?}"
                            ),
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        debug!("[{addr}] would block, breaking read loop");
                        break;
                    }
                    Err(e) => {
                        error!("[{addr}] encountered error reading from tcp connection: {e}");
                        break 'thread;
                    }
                }
            }
            if !string.is_empty() {
                debug!("[{addr}] got msg: {string}");
                let msg = format!("[{addr}] sending message to peer");
                let res = stream.write_all(msg.as_bytes());
                if let Err(err) = res {
                    error!("[{addr}] error sending message to peer: {err}");
                    break;
                }
            }

            thread::sleep(Duration::from_millis(100));
        }

        let res = stream.shutdown(Shutdown::Both);
        if let Err(err) = res {
            error!("[{addr}] could not close peer connection because error: {err}");
        }
    })
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
