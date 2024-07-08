use std::{
    any::Any,
    io::{self, BufRead, BufReader, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, SendError, Sender, TryRecvError},
    thread::{self, JoinHandle},
    time::Duration,
};

use tracing::{debug, error, warn};

pub struct RtcHandle {
    send_app_stop: Sender<()>,
    app_rtc: JoinHandle<()>,
    // send_consumer_stop: Sender<()>,
    // consumer_rtc: JoinHandle<()>,
}

#[derive(Debug)]
pub struct RtcCloseError {
    send_app_stop: Option<SendError<()>>,
    app_join: Option<Box<dyn Any + Send>>,
    // send_consumer_stop: Option<SendError<()>>,
    // consumer_join: Option<Box<dyn Any + Send>>,
}

impl RtcCloseError {
    fn any(&self) -> bool {
        self.send_app_stop.is_some() || self.app_join.is_some()
        // || self.send_consumer_stop.is_some()
        // || self.consumer_join.is_some()
    }
}

impl RtcHandle {
    pub fn close(self) -> Result<(), RtcCloseError> {
        let send_app_stop = self.send_app_stop.send(()).err();
        // let send_consumer_stop = self.send_consumer_stop.send(()).err();
        let app_join = self.app_rtc.join().err();
        // let consumer_join = self.consumer_rtc.join().err();

        let err = RtcCloseError {
            send_app_stop,
            // send_consumer_stop,
            app_join,
            // consumer_join,
        };

        if err.any() {
            Err(err)
        } else {
            Ok(())
        }
    }
}

pub fn init_rtc() -> RtcHandle {
    let (send_app_stop, recv_app_stop) = mpsc::channel();
    let app_rtc = app_rtc_thread(recv_app_stop);

    RtcHandle {
        send_app_stop,
        app_rtc,
    }
}

fn app_rtc_thread(recv_stop_rtc: Receiver<()>) -> JoinHandle<()> {
    thread::spawn(move || {
        let listener = TcpListener::bind("0.0.0.0:4000").expect("TcpListener should be valid");
        listener
            .set_nonblocking(true)
            .expect("Should be able to set listener as non-blocking");
        debug!("now listening");

        let mut stream_threads = Vec::new();
        loop {
            let should_stop = recv_stop_rtc.try_recv();
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
                    let thread = app_stream_thread(recv_stop, stream, addr);
                    stream_threads.push((send_stop, addr, thread));
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
                Err(e) => {
                    error!("encountered error accepting tcp connection: {e}");
                }
            }

            // This section is written with mutating loops instead of list transforms
            // because we need to remove threads from the thread list as we go.
            let mut finished_idxs = Vec::new();
            for (idx, (_, _, thread)) in stream_threads.iter().enumerate() {
                if thread.is_finished() {
                    finished_idxs.push(idx);
                }
            }
            for idx in finished_idxs {
                let (_, addr, thread) = stream_threads.remove(idx);
                debug!("detected that {addr} thread finished, joining");
                let res = thread.join();
                if let Err(err) = res {
                    error!("peer thread {addr} failed to join: {err:?}");
                } else {
                    debug!("finished joining {addr} thread");
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

fn app_stream_thread(
    recv_stop: Receiver<()>,
    stream: TcpStream,
    addr: SocketAddr,
) -> JoinHandle<()> {
    thread::spawn(move || {
        stream
            .set_nonblocking(false)
            .expect("Should be able to set stream as blocking");
        let mut reader = BufReader::new(stream);
        debug!("[{addr}] now handling stream");

        loop {
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
            {
                let res = reader.read_line(&mut string);
                match res {
                    Ok(0) => {
                        debug!("[{addr}] got no bytes | string: {string}");
                    }
                    Ok(x) => {
                        debug!("[{addr}] got {x} bytes | string: {string}");
                    }
                    Err(e) => {
                        error!("[{addr}] encountered error reading from tcp connection: {e}");
                        break;
                    }
                }
            }

            if !string.is_empty() {
                debug!("[{addr}] got msg: {string}");
                let msg = format!("[{addr}] sending prompted message to peer");
                let res = reader.get_ref().write_all(msg.as_bytes());
                if let Err(err) = res {
                    error!("[{addr}] error sending message to peer: {err}");
                    break;
                }
            } else {
                debug!("[{addr}] got no msg, sending to peer anyway");
                let msg = format!("[{addr}] sending unprompted message to peer");
                let res = reader.get_ref().write_all(msg.as_bytes());
                if let Err(err) = res {
                    error!("[{addr}] error sending message to peer: {err}");
                    break;
                }
            }

            thread::sleep(Duration::from_millis(100));
        }

        debug!("[{addr}] shutting down connection");
        let res = reader.get_ref().shutdown(Shutdown::Both);
        if let Err(err) = res {
            error!("[{addr}] could not close peer connection because error: {err}");
        }
    })
}
