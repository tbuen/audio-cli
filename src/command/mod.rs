use crate::control::Controller;
use interpreter::{CallbackParameter, Command, Id, Interpreter, Param, Sub};
use std::fmt::Display;

mod interpreter;

#[derive(Clone, Copy, Debug)]
enum Cmd {
    Help,
    Version,
    Ap,
    Auto,
    Network,
    List,
    Add,
    Delete,
}

impl Display for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{:?}", self);
        write!(f, "{}", s.to_lowercase())
    }
}

impl Id for Cmd {}

pub struct Interface {
    ctrl: Controller,
    intprt: Interpreter<Cmd>,
}

impl Interface {
    pub fn new(ctrl: Controller) -> Self {
        let commands = vec![
            Command::new(Cmd::Help, Sub::None(Box::new(Self::help))),
            Command::new(Cmd::Version, Sub::None(Box::new(Self::version))),
            Command::new(Cmd::Ap, Sub::Commands(vec![Command::new(Cmd::Auto, Sub::Params(vec![Param::Bool]))])),
            Command::new(
                Cmd::Network,
                Sub::Commands(vec![
                    Command::new(Cmd::List, Sub::None(Box::new(Self::network_list))),
                    Command::new(Cmd::Add, Sub::Params(vec![Param::string("ssid"), Param::string("key")])),
                    Command::new(Cmd::Delete, Sub::Params(vec![Param::string("ssid")])),
                ]),
            ),
        ];
        let intprt = Interpreter::new(commands);
        Self {
            ctrl,
            intprt,
        }
    }

    pub fn run(&mut self) {
        self.intprt.run();
    }

    fn help(params: Vec<CallbackParameter>) {
        println!("** help **");
    }
    fn version(params: Vec<CallbackParameter>) {
        println!("** version **");
    }
    fn network_list(params: Vec<CallbackParameter>) {
        println!("** network list **");
    }
}
