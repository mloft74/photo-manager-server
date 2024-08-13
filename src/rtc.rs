use std::{
    any::Any,
    io::{self, BufRead, BufReader, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam::channel::{self, Receiver, SendError, Sender, TryRecvError};
use tracing::{debug, error, warn};

use crate::{
    domain::event_system::{ScreensaverEvent, ScreensaverEventSys, ScreensaverListenerId},
    state::screensaver_event_manager::ScreensaverEventManager,
};

pub struct RtcHandle {
    send_app_stop: Sender<()>,
    app_rtc: JoinHandle<()>,
    listener_id: ScreensaverListenerId,
}

#[derive(Debug)]
pub struct RtcCloseError {
    send_app_stop: Option<SendError<()>>,
    app_join: Option<Box<dyn Any + Send>>,
}

impl RtcCloseError {
    fn any(&self) -> bool {
        self.send_app_stop.is_some() || self.app_join.is_some()
    }
}

impl RtcHandle {
    pub fn close(self, event_mngr: &mut ScreensaverEventManager) -> Result<(), RtcCloseError> {
        event_mngr.unregister(self.listener_id);

        let send_app_stop = self.send_app_stop.send(()).err();
        let app_join = self.app_rtc.join().err();

        let err = RtcCloseError {
            send_app_stop,
            app_join,
        };

        if err.any() {
            Err(err)
        } else {
            Ok(())
        }
    }
}

pub fn init_rtc(event_mngr: &mut ScreensaverEventManager) -> RtcHandle {
    let (send_app_stop, recv_app_stop) = channel::unbounded();
    let (send_event, recv_event) = channel::unbounded();
    let app_rtc = app_rtc_thread(recv_app_stop, recv_event);
    let id = event_mngr.register(Box::new(move |e| {
        let err = send_event.send(e);
        if let Err(e) = err {
            error!("[app_rtc_event_handler] could not send due to error {e}");
        }
    }));

    RtcHandle {
        send_app_stop,
        app_rtc,
        listener_id: id,
    }
}

fn app_rtc_thread(
    recv_stop_rtc: Receiver<()>,
    recv_event: Receiver<ScreensaverEvent>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let listener =
            TcpListener::bind("0.0.0.0:4000").expect("[app_rtc] TcpListener should be valid");
        listener
            .set_nonblocking(true)
            .expect("[app_rtc] Should be able to set listener as non-blocking");
        debug!("[app_rtc] now listening for connections");

        let mut stream_threads = Vec::new();
        loop {
            let should_stop = recv_stop_rtc.try_recv();
            match should_stop {
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    let name = stringify!(recv_stop_rtc);
                    warn!("[app_rtc] {name} disconnected, stopping thread");
                    break;
                }
                Ok(_) => {
                    let name = stringify!(recv_stop_rtc);
                    debug!("[app_rtc] {name} received stop signal, stopping thread");
                    break;
                }
            }

            let res = listener.accept();
            match res {
                Ok((stream, addr)) => {
                    debug!("[app_rtc] established connection with {addr}");
                    let (send_stop, recv_stop) = channel::unbounded();
                    let thread = app_stream_thread(recv_stop, recv_event.clone(), stream, addr);
                    stream_threads.push((send_stop, addr, thread));
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => (),
                Err(e) => {
                    error!("[app_rtc] encountered error accepting tcp connection: {e}");
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
                debug!("[app_rtc] detected that {addr} thread finished, joining");
                let res = thread.join();
                if let Err(err) = res {
                    error!("[app_rtc] peer thread {addr} failed to join: {err:?}");
                } else {
                    debug!("[app_rtc] finished joining {addr} thread");
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
                error!("[app_rtc] failed to send stop signal to thread for {addr}: {err}");
            }
        }
        for (thread, addr) in threads {
            let res = thread.join();
            if let Err(err) = res {
                error!("[app_rtc] failed to join thread for {addr}: {err:?}");
            }
        }
    })
}

fn app_stream_thread(
    recv_stop: Receiver<()>,
    recv_event: Receiver<ScreensaverEvent>,
    stream: TcpStream,
    addr: SocketAddr,
) -> JoinHandle<()> {
    thread::spawn(move || {
        stream
            .set_nonblocking(false)
            .expect("[app_stream] Should be able to set stream as blocking");
        let mut reader = BufReader::new(stream);
        debug!("[app_stream] [{addr}] now handling stream");

        loop {
            let should_stop = recv_stop.try_recv();
            match should_stop {
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    warn!("[app_stream] [{addr}] recv_stop disconnected, stopping thread");
                    break;
                }
                Ok(_) => {
                    debug!("[app_stream] [{addr}] recv_stop received stop signal, stopping thread");
                    break;
                }
            }

            let mut string = String::new();
            {
                let res = reader.read_line(&mut string);
                match res {
                    Ok(0) => {
                        debug!("[app_stream] [{addr}] got no bytes | string: {string}");
                    }
                    Ok(x) => {
                        debug!("[app_stream] [{addr}] got {x} bytes | string: {string}");
                    }
                    Err(e) => {
                        error!("[app_stream] [{addr}] encountered error reading from tcp connection: {e}");
                        break;
                    }
                }
            }

            if !string.is_empty() {
                debug!("[app_stream] [{addr}] got msg: {string}");
            }

            let event = recv_event.try_recv();
            match event {
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    warn!("[app_stream] [{addr}] recv_stop disconnected, stopping thread");
                    break;
                }
                Ok(event) => {
                    let json = serde_json::to_string(&event);
                    match json {
                        Err(err) => {
                            error!(
                                "[app_stream] [{addr}] error converting {event:?} to json: {err}"
                            );
                            break;
                        }
                        Ok(json) => {
                            let res = reader.get_ref().write_all(json.as_bytes());
                            if let Err(err) = res {
                                error!(
                                    "[app_stream] [{addr}] error sending message to peer: {err}"
                                );
                                break;
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }

        debug!("[app_stream] [{addr}] shutting down connection");
        let res = reader.get_ref().shutdown(Shutdown::Both);
        if let Err(err) = res {
            error!("[app_stream] [{addr}] could not close peer connection because error: {err}");
        }
    })
}
