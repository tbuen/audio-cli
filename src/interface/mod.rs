use std::cmp;

use colored::Colorize as _;
use smart_repl::{Args, Command, Group, Parameter, Repl};

use crate::control::{Controller, Status};

pub(crate) struct Cli<'a> {
    repl: Repl<'a, Controller>,
}

impl<'a> Cli<'a> {
    pub(crate) fn new(ctrl: &'a Controller) -> Self {
        let repl = Repl::builder()
            .with_context(ctrl)
            .with_prompt(">> ")
            .with_help()
            .with_command(Command::new("version", version).with_help("Show version information."))
            .with_group(
                Group::new("ap")
                    .with_help("Handle connection to the device's access point.")
                    .with_command(
                        Command::new("auto", ap_auto)
                            .with_help("Switch automatic access point connection on/off.")
                            .with_optional_parameter(Parameter::bool("value", "on", "off")),
                    ),
            )
            .with_group(
                Group::new("con")
                    .with_help("Handle connection to the device.")
                    .with_command(
                        Command::new("status", con_status).with_help("Show connection status."),
                    )
                    .with_command(
                        Command::new("version", con_version)
                            .with_help("Show device version information."),
                    ),
            )
            .with_group(
                Group::new("network")
                    .with_help("Configure the networks to which the device should connect.")
                    .with_command(
                        Command::new("scan", network_scan)
                            .with_help("Display the list of networks scanned by device."),
                    )
                    .with_command(
                        Command::new("list", network_list)
                            .with_help("Display the list of configured networks."),
                    )
                    .with_command(
                        Command::new("add", network_add)
                            .with_help("Add a network to the list or edit an existing one.")
                            .with_parameter(Parameter::string("ssid"))
                            .with_parameter(Parameter::string("key")),
                    )
                    .with_command(
                        Command::new("remove", network_remove)
                            .with_help("Remove a network from the list.")
                            .with_parameter(Parameter::string("ssid")),
                    ),
            )
            .build();

        Self { repl }
    }

    pub(crate) fn run(&mut self) {
        self.repl.run();
    }
}

fn version(_: Option<&Controller>, _: Args) {
    let versions = vec![
        (
            env!("CARGO_PKG_NAME").to_owned() + ":",
            env!("VERSION").to_owned(),
        ),
        (
            smart_repl::NAME.to_owned() + ":",
            smart_repl::VERSION.to_owned(),
        ),
        (
            Controller::backend_name().to_owned() + ":",
            Controller::backend_version().to_owned(),
        ),
    ];
    let max = versions.iter().fold(0, |m, (n, _)| cmp::max(m, n.len()));
    for (n, v) in &versions {
        println!("{n:max$} {v}");
    }
}

fn ap_auto(ctrl: Option<&Controller>, args: Args) {
    let ctrl = ctrl.unwrap();
    if let Some(val) = args.get_bool("value").unwrap() {
        ctrl.set_access_point_mode(val);
    } else if ctrl.get_access_point_mode() {
        println!("on");
    } else {
        println!("off");
    }
}

fn con_status(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let status = ctrl.get_con_status();
    if let Status::Connected((con, _)) = status {
        println!("Connected: true");
        println!("Mode:      {}", con.mode);
    } else {
        println!("Connected: false");
    }
}

fn con_version(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let status = ctrl.get_con_status();
    if let Status::Connected((_, version)) = status {
        println!("Project:   {}", version.project);
        println!("Version:   {}", version.version);
        println!("ESP-IDF:   {}", version.esp_idf);
    }
}

fn network_scan(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let result = ctrl.get_wifi_scan_result();
    match result {
        Ok(list) => {
            for network in list {
                println!("{network}");
            }
        }
        Err(e) => println!("{}", e.to_string().red()),
    }
}

fn network_list(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let result = ctrl.get_wifi_network_list();
    match result {
        Ok(list) => {
            for network in list {
                println!("{network}");
            }
        }
        Err(e) => println!("{}", e.to_string().red()),
    }
}

fn network_add(ctrl: Option<&Controller>, mut args: Args) {
    let ctrl = ctrl.unwrap();
    let ssid = args.get_string("ssid").unwrap().unwrap();
    let key = args.get_string("key").unwrap().unwrap();
    let result = ctrl.set_wifi_network(ssid, key);
    match result {
        Ok(()) => {}
        Err(e) => println!("{}", e.to_string().red()),
    }
}

fn network_remove(ctrl: Option<&Controller>, mut args: Args) {
    let ctrl = ctrl.unwrap();
    let ssid = args.get_string("ssid").unwrap().unwrap();
    let result = ctrl.delete_wifi_network(ssid);
    match result {
        Ok(()) => {}
        Err(e) => println!("{}", e.to_string().red()),
    }
}
