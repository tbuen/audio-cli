use std::error;
use std::fmt;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{Builder, JoinHandle};
use std::time::Duration;

use backend::{Backend, Con, Event, Memory, Network, NotConnectedError, RemoteError, Version};
use log::debug;

#[derive(Debug)]
pub(crate) enum Error {
    NotConnected,
    Timeout,
    Remote(RemoteError),
}

pub(crate) struct Controller {
    backend: Backend,
    handle: Option<JoinHandle<()>>,
    sender: Sender<Command>,
    shared: Arc<(Mutex<SharedData>, Condvar)>,
}

#[derive(Default)]
struct SharedData {
    status: Status,
    error: Option<RemoteError>,
    info_memory: Option<Result<Memory, RemoteError>>,
    scan_result: Option<Result<Vec<Network>, RemoteError>>,
    network_list: Option<Result<Vec<String>, RemoteError>>,
}

#[derive(Default, Clone)]
pub(crate) enum Status {
    #[default]
    Disconnected,
    Connected((Con, Version)),
}

//#[derive(Default, Clone)]
//pub(crate) struct Info {
//    mode: String,
//    project: String,
//    version: String,
//    esp_idf: String,
//}

enum Command {
    Quit,
}

impl Controller {
    pub(crate) fn new() -> Self {
        let backend = Backend::new();
        let receiver = backend.receiver().unwrap();
        let shared = Arc::new((Mutex::new(SharedData::default()), Condvar::new()));
        let shared_thread = shared.clone();
        let (sender, rx) = mpsc::channel();
        Self {
            backend,
            handle: Some(
                Builder::new()
                    .name("control".into())
                    .spawn(move || Self::thread(rx, receiver, shared_thread))
                    .unwrap(),
            ),
            sender,
            shared,
        }
    }

    pub(crate) fn backend_name() -> &'static str {
        backend::NAME
    }

    pub(crate) fn backend_version() -> &'static str {
        backend::VERSION
    }

    pub(crate) fn get_access_point_mode(&self) -> bool {
        self.backend.get_access_point_mode()
    }

    pub(crate) fn set_access_point_mode(&self, auto: bool) {
        self.backend.set_access_point_mode(auto);
    }

    pub(crate) fn get_con_status(&self) -> Status {
        let (mutex, _) = &*self.shared;
        let data = mutex.lock().unwrap();
        data.status.clone()
    }

    pub(crate) fn get_info_memory(&self) -> Result<Memory, Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(NotConnectedError) = self.backend.get_info_memory() {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.info_memory.take().unwrap().map_err(Error::Remote)
            }
        }
    }

    pub(crate) fn get_wifi_scan_result(&self) -> Result<Vec<Network>, Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(NotConnectedError) = self.backend.get_wifi_scan_result() {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.scan_result.take().unwrap().map_err(Error::Remote)
            }
        }
    }

    pub(crate) fn get_wifi_network_list(&self) -> Result<Vec<String>, Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(NotConnectedError) = self.backend.get_wifi_network_list() {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.network_list.take().unwrap().map_err(Error::Remote)
            }
        }
    }

    pub(crate) fn set_wifi_network(&self, ssid: String, key: String) -> Result<(), Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(NotConnectedError) = self.backend.set_wifi_network(ssid, key) {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.error.take().map_or(Ok(()), |e| Err(Error::Remote(e)))
            }
        }
    }

    pub(crate) fn delete_wifi_network(&self, ssid: String) -> Result<(), Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(NotConnectedError) = self.backend.delete_wifi_network(ssid) {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.error.take().map_or(Ok(()), |e| Err(Error::Remote(e)))
            }
        }
    }

    fn thread(
        rx: Receiver<Command>,
        receiver: Receiver<Event>,
        shared: Arc<(Mutex<SharedData>, Condvar)>,
    ) {
        let (mutex, cvar) = &*shared;
        loop {
            if let Ok(Command::Quit) = rx.try_recv() {
                debug!("control thread received Quit");
                break;
            }

            if let Ok(event) = receiver.recv_timeout(Duration::from_millis(10)) {
                match event {
                    Event::Connected(con, info) => {
                        let mut data = mutex.lock().unwrap();
                        data.status = Status::Connected((con, info));
                    }
                    Event::Disconnected => {
                        let mut data = mutex.lock().unwrap();
                        data.status = Status::Disconnected;
                    }
                    Event::InfoMemory(res) => {
                        let mut data = mutex.lock().unwrap();
                        data.info_memory = Some(res);
                        cvar.notify_one();
                    }
                    Event::ScanResult(res) => {
                        let mut data = mutex.lock().unwrap();
                        data.scan_result = Some(res);
                        cvar.notify_one();
                    }
                    Event::NetworkList(res) => {
                        let mut data = mutex.lock().unwrap();
                        data.network_list = Some(res);
                        cvar.notify_one();
                    }
                    Event::SetNetwork(res) | Event::DeleteNetwork(res) => {
                        let mut data = mutex.lock().unwrap();
                        data.error = res.err();
                        cvar.notify_one();
                    }
                }
            }
        }

        debug!("exit control thread");
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        self.sender.send(Command::Quit).unwrap();
        self.handle.take().unwrap().join().unwrap();
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::NotConnected => write!(f, "Not connected"),
            Self::Timeout => write!(f, "Timeout"),
            Self::Remote(e) => write!(f, "Error: {} ({})", e.message, e.code),
        }
    }
}
