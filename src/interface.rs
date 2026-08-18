#![allow(clippy::print_stdout)]
use std::cmp;

use colored::Colorize as _;
use smart_repl::{Args, Command, Group, Parameter, Repl};

use crate::control::Controller;

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
            .with_command(
                Command::new("connection", connection).with_help("Show connection status."),
            )
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
                Group::new("info")
                    .with_help("Show device information.")
                    .with_command(
                        Command::new("about", info_about)
                            .with_help("Show device version information."),
                    )
                    .with_command(
                        Command::new("memory", info_memory)
                            .with_help("Show device memory information."),
                    )
                    .with_command(
                        Command::new("flash", info_flash)
                            .with_help("Show device flash information."),
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
            .with_group(
                Group::new("sync")
                    .with_help("Sync data from device.")
                    .with_command(Command::new("files", sync_files).with_help("Sync file list."))
                    .with_command(
                        Command::new("tags", sync_tags)
                            .with_help("Sync tags of current and child directory."),
                    ),
            )
            .with_group(
                Group::new("fs")
                    .with_help("Access file system.")
                    .with_command(Command::new("pwd", fs_pwd).with_help("Print current directory."))
                    .with_command(
                        Command::new("cd", fs_cd)
                            .with_help("Change directory.")
                            .with_parameter(Parameter::string("dir")),
                    )
                    .with_command(Command::new("ls", fs_ls).with_help("Print directory content.")),
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

fn connection(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let result = ctrl.get_info_connection();
    match result {
        Ok(info) => {
            println!("Mode: {}", info.mode);
        }
        Err(e) => println!("{}", e.to_string().bold()),
    }
}

fn info_about(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let result = ctrl.get_info_about();
    match result {
        Ok(info) => {
            println!("Project:   {}", info.project);
            println!("Version:   {}", info.version);
            println!("ESP-IDF:   {}", info.esp_idf);
        }
        Err(e) => println!("{}", e.to_string().bold()),
    }
}

fn info_memory(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let result = ctrl.get_info_memory();
    match result {
        Ok(info) => {
            println!("heap");
            println!(
                "   total:        {:3} KiB",
                info.heap.allocated / 1024 + info.heap.free / 1024
            );
            println!("   allocated:    {:3} KiB", info.heap.allocated / 1024);
            println!("   free:         {:3} KiB", info.heap.free / 1024);
            println!("   minimum free: {:3} KiB", info.heap.minimum_free / 1024);
        }
        Err(e) => println!("{}", e.to_string().bold()),
    }
}

fn info_flash(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let result = ctrl.get_info_spiflash();
    match result {
        Ok(info) => {
            println!("files");
            for f in info.files {
                println!("   {} {:6} {}", f.md5, f.size, f.name);
            }
            println!("total: {:3} KiB", info.total / 1024);
            println!("free:  {:3} KiB", info.free / 1024);
        }
        Err(e) => println!("{}", e.to_string().bold()),
    }
}

fn network_scan(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let result = ctrl.get_wifi_scan_result();
    match result {
        Ok(list) => {
            for network in list {
                //println!("{} {}", '\u{1f6dc}', network.ssid);
                match network.rssi {
                    3 => println!("\u{1f7e2} {}", network.ssid),
                    2 => println!("\u{1f7e1} {}", network.ssid),
                    1 => println!("\u{1f534} {}", network.ssid),
                    _ => println!("? {}", network.ssid),
                }
            }
        }
        Err(e) => println!("{}", e.to_string().bold()),
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
        Err(e) => println!("{}", e.to_string().bold()),
    }
}

fn network_add(ctrl: Option<&Controller>, mut args: Args) {
    let ctrl = ctrl.unwrap();
    let ssid = args.get_string("ssid").unwrap().unwrap();
    let key = args.get_string("key").unwrap().unwrap();
    let result = ctrl.set_wifi_network(ssid, key);
    match result {
        Ok(()) => {}
        Err(e) => println!("{}", e.to_string().bold()),
    }
}

fn network_remove(ctrl: Option<&Controller>, mut args: Args) {
    let ctrl = ctrl.unwrap();
    let ssid = args.get_string("ssid").unwrap().unwrap();
    let result = ctrl.delete_wifi_network(ssid);
    match result {
        Ok(()) => {}
        Err(e) => println!("{}", e.to_string().bold()),
    }
}

fn sync_files(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let result = ctrl.sync_files();
    match result {
        Ok(()) => {}
        Err(e) => println!("{}", e.to_string().bold()),
    }
}

fn sync_tags(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let result = ctrl.sync_tags();
    match result {
        Ok(()) => {}
        Err(e) => println!("{}", e.to_string().bold()),
    }
}

fn fs_pwd(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let result = ctrl.current_directory();
    match result {
        Ok(v) => println!("{v}"),
        Err(e) => println!("{}", e.to_string().bold()),
    }
}

fn fs_cd(ctrl: Option<&Controller>, mut args: Args) {
    let ctrl = ctrl.unwrap();
    let dir = args.get_string("dir").unwrap().unwrap();
    let result = ctrl.change_directory(&dir);
    match result {
        Ok(()) => {}
        Err(e) => println!("{}", e.to_string().bold()),
    }
}

fn fs_ls(ctrl: Option<&Controller>, _: Args) {
    let ctrl = ctrl.unwrap();
    let result = ctrl.directory_content();
    match result {
        Ok(content) => {
            for d in content.dirs {
                println!("<{d}>");
            }
            if let Some(c) = content.cover {
                println!("#{c}#");
            }
            for t in content.tracks {
                println!("{t}");
            }
        }
        Err(e) => println!("{}", e.to_string().bold()),
    }
}
