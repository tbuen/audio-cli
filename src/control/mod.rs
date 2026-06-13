use std::fmt;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{Builder, JoinHandle};
use std::time::Duration;

use backend::{Backend, Con, Event, Version};
use log::debug;

pub(crate) enum Error {
    NotConnected,
}

pub(crate) struct Controller {
    backend: Backend,
    handle: Option<JoinHandle<()>>,
    sender: Sender<Command>,
    data: Arc<Mutex<Data>>,
}

#[derive(Default)]
struct Data {
    status: Status,
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
        let data = Arc::new(Mutex::new(Data::default()));
        let d = Arc::clone(&data);
        let (sender, rx) = mpsc::channel();
        Self {
            backend,
            handle: Some(
                Builder::new()
                    .name("control".into())
                    .spawn(move || Self::thread(rx, receiver, d))
                    .unwrap(),
            ),
            sender,
            data,
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
        let data = self.data.lock().unwrap();
        data.status.clone()
    }

    pub(crate) fn get_wifi_scan_result(&self) -> Result<Vec<String>, Error> {
        let data = self.data.lock().unwrap();
        match data.status {
            Status::Connected(_) => {
                self.backend.get_wifi_scan_result();
                Ok(Vec::new())
            }
            Status::Disconnected => Err(Error::NotConnected),
        }
    }

    pub(crate) fn get_wifi_network_list(&self) -> Result<Vec<String>, Error> {
        let data = self.data.lock().unwrap();
        match data.status {
            Status::Connected(_) => {
                self.backend.get_wifi_network_list();
                Ok(Vec::new())
            }
            Status::Disconnected => Err(Error::NotConnected),
        }
    }

    pub(crate) fn set_wifi_network(&self, ssid: String, key: String) -> Result<(), Error> {
        let data = self.data.lock().unwrap();
        match data.status {
            Status::Connected(_) => {
                self.backend.set_wifi_network(ssid, key);
                Ok(())
            }
            Status::Disconnected => Err(Error::NotConnected),
        }
    }

    pub(crate) fn delete_wifi_network(&self, ssid: String) -> Result<(), Error> {
        let data = self.data.lock().unwrap();
        match data.status {
            Status::Connected(_) => {
                self.backend.delete_wifi_network(ssid);
                Ok(())
            }
            Status::Disconnected => Err(Error::NotConnected),
        }
    }

    fn thread(rx: Receiver<Command>, receiver: Receiver<Event>, data: Arc<Mutex<Data>>) {
        loop {
            if let Ok(Command::Quit) = rx.try_recv() {
                debug!("control thread received Quit");
                break;
            }

            if let Ok(event) = receiver.recv_timeout(Duration::from_millis(10)) {
                match event {
                    Event::Connected(con, info) => {
                        let mut data = data.lock().unwrap();
                        data.status = Status::Connected((con, info));
                    }
                    Event::Disconnected => {
                        let mut data = data.lock().unwrap();
                        data.status = Status::Disconnected;
                    }
                    Event::ScanResult(res) => match res {
                        Ok(list) => println!("{list:?}"),
                        Err(e) => println!("{e:?}"),
                    },
                    Event::NetworkList(res) => match res {
                        Ok(list) => println!("{list:?}"),
                        Err(e) => println!("{e:?}"),
                    },
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::NotConnected => write!(f, "Not connected."),
        }
    }
}
