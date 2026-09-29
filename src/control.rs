use std::error;
use std::fmt;
use std::result;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{Builder, JoinHandle};
use std::time::Duration;

use backend::{
    About, Backend, ChangeDirection, Connection, Event, FileSync, FileViewContent, Memory, Network,
    SPIFlash, TagSync, TagViewContent,
};
use log::{debug, info};

pub(crate) type Result<T> = result::Result<T, Error>;

#[derive(Debug, Clone)]
pub(crate) enum Error {
    Timeout,
    Backend(backend::Error),
}

pub(crate) struct Controller {
    backend: Backend,
    handle: Option<JoinHandle<()>>,
    sender: Sender<Command>,
    shared: Arc<(Mutex<SharedData>, Condvar)>,
}

#[derive(Default)]
struct SharedData {
    error: Option<Error>,
    info_connection: Option<Connection>,
    info_about: Option<About>,
    info_memory: Option<Memory>,
    info_spiflash: Option<SPIFlash>,
    scan_result: Option<Vec<Network>>,
    network_list: Option<Vec<String>>,
    volume: Option<i32>,
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

    pub(crate) fn get_info_connection(&self) -> Result<Connection> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        self.backend.get_info_connection();
        let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
        if result.timed_out() {
            Err(Error::Timeout)
        } else {
            data.info_connection
                .take()
                .ok_or_else(|| data.error.take().unwrap())
        }
    }

    pub(crate) fn get_info_about(&self) -> Result<About> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        self.backend.get_info_about();
        let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
        if result.timed_out() {
            Err(Error::Timeout)
        } else {
            data.info_about
                .take()
                .ok_or_else(|| data.error.take().unwrap())
        }
    }

    pub(crate) fn get_info_memory(&self) -> Result<Memory> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        self.backend.get_info_memory();
        let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
        if result.timed_out() {
            Err(Error::Timeout)
        } else {
            data.info_memory
                .take()
                .ok_or_else(|| data.error.take().unwrap())
        }
    }

    pub(crate) fn get_info_spiflash(&self) -> Result<SPIFlash> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        self.backend.get_info_spiflash();
        let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
        if result.timed_out() {
            Err(Error::Timeout)
        } else {
            data.info_spiflash
                .take()
                .ok_or_else(|| data.error.take().unwrap())
        }
    }

    pub(crate) fn get_wifi_scan_result(&self) -> Result<Vec<Network>> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        self.backend.get_wifi_scan_result();
        let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
        if result.timed_out() {
            Err(Error::Timeout)
        } else {
            data.scan_result
                .take()
                .ok_or_else(|| data.error.take().unwrap())
        }
    }

    pub(crate) fn get_wifi_network_list(&self) -> Result<Vec<String>> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        self.backend.get_wifi_network_list();
        let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
        if result.timed_out() {
            Err(Error::Timeout)
        } else {
            data.network_list
                .take()
                .ok_or_else(|| data.error.take().unwrap())
        }
    }

    pub(crate) fn set_wifi_network(&self, ssid: String, key: String) -> Result<()> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        self.backend.set_wifi_network(ssid, key);
        let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
        if result.timed_out() {
            Err(Error::Timeout)
        } else {
            data.error.take().map_or(Ok(()), Err)
        }
    }

    pub(crate) fn delete_wifi_network(&self, ssid: String) -> Result<()> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        self.backend.delete_wifi_network(ssid);
        let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
        if result.timed_out() {
            Err(Error::Timeout)
        } else {
            data.error.take().map_or(Ok(()), Err)
        }
    }

    pub(crate) fn sync_files(&self) -> Result<()> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        self.backend.sync_files();
        let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(10)).unwrap();
        if result.timed_out() {
            Err(Error::Timeout)
        } else {
            data.error.take().map_or(Ok(()), Err)
        }
    }

    pub(crate) fn sync_tags(&self) -> Result<()> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        self.backend.sync_tags();
        let (mut data, result) = cvar.wait_timeout(data, Duration::from_mins(10)).unwrap();
        if result.timed_out() {
            Err(Error::Timeout)
        } else {
            data.error.take().map_or(Ok(()), Err)
        }
    }

    pub(crate) fn fileview_current(&self) -> String {
        self.backend.fileview_current().pop().unwrap_or_default()
    }

    pub(crate) fn fileview_change(&self, dir: &str) -> Result<()> {
        let d = {
            if dir == "/" {
                ChangeDirection::ToRoot
            } else if dir == ".." {
                ChangeDirection::ToParent
            } else {
                ChangeDirection::ToChild(dir)
            }
        };
        self.backend.fileview_change(d).map_err(Into::into)
    }

    pub(crate) fn fileview_content(&self) -> FileViewContent {
        self.backend.fileview_content()
    }

    pub(crate) fn tagview_current(&self) -> String {
        self.backend.tagview_current().pop().unwrap_or_default()
    }

    pub(crate) fn tagview_change(&self, dir: &str) -> Result<()> {
        let d = {
            if dir == "/" {
                ChangeDirection::ToRoot
            } else if dir == ".." {
                ChangeDirection::ToParent
            } else {
                ChangeDirection::ToChild(dir)
            }
        };
        self.backend.tagview_change(d).map_err(Into::into)
    }

    pub(crate) fn tagview_content(&self) -> TagViewContent {
        self.backend.tagview_content()
    }

    pub(crate) fn volume(&self) -> Result<i32> {
        let (mutex, _) = &*self.shared;
        let data = mutex.lock().unwrap();
        data.volume
            .ok_or(Error::Backend(backend::Error::NotConnected))
    }

    pub(crate) fn set_volume(&self, value: i32) -> Result<()> {
        let (mutex, cvar) = &*self.shared;
        let data = mutex.lock().unwrap();
        self.backend.set_volume(value);
        let (mut data, result) = cvar.wait_timeout(data, Duration::from_secs(3)).unwrap();
        if result.timed_out() {
            Err(Error::Timeout)
        } else {
            data.error.take().map_or(Ok(()), Err)
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
                    Event::WiFiScanResult(res) => {
                        let mut data = mutex.lock().unwrap();
                        data.scan_result = Some(res);
                        cvar.notify_one();
                    }
                    Event::WiFiNetworkList(res) => {
                        let mut data = mutex.lock().unwrap();
                        data.network_list = Some(res);
                        cvar.notify_one();
                    }
                    Event::WiFiSetNetwork | Event::WiFiDeleteNetwork | Event::SetVolume => {
                        cvar.notify_one();
                    }
                    Event::FileSync(res) => {
                        let mut data = mutex.lock().unwrap();
                        match res {
                            FileSync::Started => info!("STARTED"),
                            FileSync::Aborted => info!("ABORTED"),
                            FileSync::Completed => {
                                info!("COMPLETED");
                                data.error = None;
                                cvar.notify_one();
                            }
                        }
                    }
                    Event::TagSync(res) => {
                        let mut data = mutex.lock().unwrap();
                        match res {
                            TagSync::Started => info!("STARTED"),
                            TagSync::Aborted => info!("ABORTED"),
                            TagSync::Step(n, t) => debug!("progress: {n}/{t}"),
                            TagSync::Completed => {
                                info!("COMPLETED");
                                data.error = None;
                                cvar.notify_one();
                            }
                        }
                    }
                    Event::Error(e) => {
                        let mut data = mutex.lock().unwrap();
                        data.error = Some(e.into());
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => write!(f, "timeout"),
            Self::Backend(e) => write!(f, "{e}"),
        }
    }
}

impl From<backend::Error> for Error {
    fn from(value: backend::Error) -> Self {
        Self::Backend(value)
    }
}
