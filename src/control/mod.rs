use std::error;
use std::fmt;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{Builder, JoinHandle};
use std::time::Duration;

use backend::{
    About, Backend, Connection, Dir, Event, FsError, Memory, Network, RemoteError, SPIFlash,
    SyncStatus,
};
use log::{debug, info};

#[derive(Debug)]
pub(crate) enum Error {
    NotConnected,
    AlreadyRunning,
    Timeout,
    Remote,
    NotSynced,
    NotFound,
}

pub(crate) struct Controller {
    backend: Backend,
    handle: Option<JoinHandle<()>>,
    sender: Sender<Command>,
    shared: Arc<(Mutex<SharedData>, Condvar)>,
}

#[derive(Default)]
struct SharedData {
    error: Option<RemoteError>,
    info_connection: Option<Result<Connection, RemoteError>>,
    info_about: Option<Result<About, RemoteError>>,
    info_memory: Option<Result<Memory, RemoteError>>,
    info_spiflash: Option<Result<SPIFlash, RemoteError>>,
    scan_result: Option<Result<Vec<Network>, RemoteError>>,
    network_list: Option<Result<Vec<String>, RemoteError>>,
    file_sync: Option<bool>,
}

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

    pub(crate) fn get_info_connection(&self) -> Result<Connection, Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(backend::Error::NotConnected) = self.backend.get_info_connection() {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.info_connection
                    .take()
                    .unwrap()
                    .map_err(|_| Error::Remote)
            }
        }
    }

    pub(crate) fn get_info_about(&self) -> Result<About, Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(backend::Error::NotConnected) = self.backend.get_info_about() {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.info_about.take().unwrap().map_err(|_| Error::Remote)
            }
        }
    }

    pub(crate) fn get_info_memory(&self) -> Result<Memory, Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(backend::Error::NotConnected) = self.backend.get_info_memory() {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.info_memory.take().unwrap().map_err(|_| Error::Remote)
            }
        }
    }

    pub(crate) fn get_info_spiflash(&self) -> Result<SPIFlash, Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(backend::Error::NotConnected) = self.backend.get_info_spiflash() {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.info_spiflash
                    .take()
                    .unwrap()
                    .map_err(|_| Error::Remote)
            }
        }
    }

    pub(crate) fn get_wifi_scan_result(&self) -> Result<Vec<Network>, Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(backend::Error::NotConnected) = self.backend.get_wifi_scan_result() {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.scan_result.take().unwrap().map_err(|_| Error::Remote)
            }
        }
    }

    pub(crate) fn get_wifi_network_list(&self) -> Result<Vec<String>, Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(backend::Error::NotConnected) = self.backend.get_wifi_network_list() {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.network_list.take().unwrap().map_err(|_| Error::Remote)
            }
        }
    }

    pub(crate) fn set_wifi_network(&self, ssid: String, key: String) -> Result<(), Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(backend::Error::NotConnected) = self.backend.set_wifi_network(ssid, key) {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.error.take().map_or(Ok(()), |_| Err(Error::Remote))
            }
        }
    }

    pub(crate) fn delete_wifi_network(&self, ssid: String) -> Result<(), Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        if let Err(backend::Error::NotConnected) = self.backend.delete_wifi_network(ssid) {
            Err(Error::NotConnected)
        } else {
            let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
            if result.timed_out() {
                Err(Error::Timeout)
            } else {
                data.error.take().map_or(Ok(()), |_| Err(Error::Remote))
            }
        }
    }

    pub(crate) fn sync_files(&self) -> Result<bool, Error> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        match self.backend.sync_files() {
            Err(backend::Error::NotConnected) => Err(Error::NotConnected),
            Err(backend::Error::AlreadyRunning) => Err(Error::AlreadyRunning),
            Ok(()) => {
                let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(10)).unwrap();
                if result.timed_out() {
                    Err(Error::Timeout)
                } else {
                    Ok(data.file_sync.take().unwrap())
                }
            }
        }
    }

    pub(crate) fn fs_pwd(&self) -> Result<String, Error> {
        let fs = self.backend.filesystem();
        fs.lock().unwrap().pwd().map_err(|_| Error::NotSynced)
    }

    pub(crate) fn fs_cd(&self, dir: &str) -> Result<(), Error> {
        let fs = self.backend.filesystem();
        let d = {
            if dir == "/" {
                Dir::Root
            } else if dir == ".." {
                Dir::Up
            } else {
                Dir::Down(dir)
            }
        };
        fs.lock().unwrap().cd(d).map_err(|e| match e {
            FsError::NotSynced => Error::NotSynced,
            FsError::NotFound => Error::NotFound,
        })
    }

    pub(crate) fn fs_ls(&self) -> Result<(Vec<String>, Vec<String>), Error> {
        let fs = self.backend.filesystem();
        fs.lock().unwrap().ls().map_err(|_| Error::NotSynced)
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
                    Event::Connected => info!("Connected"),
                    Event::Disconnected => info!("Disconnected"),
                    Event::InfoConnection(res) => {
                        let mut data = mutex.lock().unwrap();
                        data.info_connection = Some(res);
                        cvar.notify_one();
                    }
                    Event::InfoAbout(res) => {
                        let mut data = mutex.lock().unwrap();
                        data.info_about = Some(res);
                        cvar.notify_one();
                    }
                    Event::InfoMemory(res) => {
                        let mut data = mutex.lock().unwrap();
                        data.info_memory = Some(res);
                        cvar.notify_one();
                    }
                    Event::InfoSPIFlash(res) => {
                        let mut data = mutex.lock().unwrap();
                        data.info_spiflash = Some(res);
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
                    Event::FileSync(status) => {
                        let mut data = mutex.lock().unwrap();
                        match status {
                            SyncStatus::Idle => {
                                info!("IDLE");
                                data.file_sync = Some(false);
                                cvar.notify_one();
                            }
                            SyncStatus::Running => info!("RUNNING"),
                            SyncStatus::Completed => {
                                info!("COMPLETED");
                                data.file_sync = Some(true);
                                cvar.notify_one();
                            }
                        }
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
            Self::AlreadyRunning => write!(f, "Already running"),
            Self::Timeout => write!(f, "Timeout"),
            Self::Remote => write!(f, "Error"),
            Self::NotSynced => write!(f, "Not synchronized"),
            Self::NotFound => write!(f, "Not found"),
        }
    }
}
