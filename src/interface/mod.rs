use crate::control::Controller;
use smart_repl::{Args, Command, Group, Parameter, Repl};

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
                Group::new("network")
                    .with_help("Configure the networks to which the device should connect.")
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

fn version(_: Option<&Controller>, _: &Args) {
    let mut versions = vec![
        (
            env!("CARGO_PKG_NAME").to_owned(),
            env!("VERSION").to_owned(),
        ),
        (smart_repl::NAME.to_owned(), smart_repl::VERSION.to_owned()),
    ];
    versions.push((
        Controller::backend_name().into(),
        Controller::backend_version().to_owned(),
    ));
    versions = versions.into_iter().map(|x| (x.0 + ":", x.1)).collect();
    let maxlen = versions
        .iter()
        .max_by(|a, b| a.0.len().cmp(&b.0.len()))
        .unwrap()
        .0
        .len();
    for v in &versions {
        println!("{:2$} {}", v.0, v.1, maxlen);
    }
}

fn ap_auto(ctrl: Option<&Controller>, args: &Args) {
    let ctrl = ctrl.unwrap();
    if let Some(val) = args.get_bool("value").unwrap() {
        ctrl.set_access_point_mode(val);
    } else if ctrl.get_access_point_mode() {
        println!("on");
    } else {
        println!("off");
    }
}

fn network_list(_: Option<&Controller>, _: &Args) {
    println!("** network list **");
}

fn network_add(_: Option<&Controller>, _: &Args) {
    println!("** network add **");
}

fn network_remove(_: Option<&Controller>, _: &Args) {
    println!("** network remove **");
}
