use crate::control::Controller;
use helper::{Command, CommandHelper, Id, Param, Sub, Token};
use log::error;
use rustyline::config::Builder;
use rustyline::error::ReadlineError;
use rustyline::history::MemHistory;
use rustyline::{CompletionType, Editor};
use std::collections::VecDeque;
use std::fmt::Display;

mod helper;

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

pub struct CommandLine {
    ctrl: Controller,
    rl: Editor<CommandHelper<Cmd>, MemHistory>,
}

impl CommandLine {
    pub fn new(ctrl: Controller) -> Self {
        let commands = vec![
            Command::new(Cmd::Help, Sub::None),
            Command::new(Cmd::Version, Sub::None),
            Command::new(Cmd::Ap, Sub::Commands(vec![Command::new(Cmd::Auto, Sub::Params(vec![Param::Bool]))])),
            Command::new(
                Cmd::Network,
                Sub::Commands(vec![
                    Command::new(Cmd::List, Sub::None),
                    Command::new(Cmd::Add, Sub::Params(vec![Param::string("ssid"), Param::string("key")])),
                    Command::new(Cmd::Delete, Sub::Params(vec![Param::string("ssid")])),
                ]),
            ),
        ];
        let mut rl = Editor::with_config(Builder::new().completion_type(CompletionType::List).build()).unwrap();
        rl.set_helper(Some(CommandHelper::new(commands)));
        Self {
            ctrl,
            rl,
        }
    }

    pub fn run(&mut self) {
        loop {
            let readline = self.rl.readline(">> ");
            match readline {
                Ok(line) => match self.rl.helper().unwrap().parse(&line) {
                    Ok(res) => {
                        if self.interpret(res) {
                            match self.rl.add_history_entry(line) {
                                Ok(b) => println!("history: {b}"),
                                Err(e) => println!("{:?}", e),
                            }
                        }
                    }
                    Err(_) => println!("## invalid input"),
                },
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    error!("{:?}", err);
                    break;
                }
            }
        }
    }

    fn interpret(&self, mut tokens: VecDeque<Token<Cmd>>) -> bool {
        let mut ok = false;
        match tokens.pop_front() {
            Some(Token::Command(cmd)) => match cmd {
                Cmd::Help => println!("## unimplemented"),
                Cmd::Ap => match tokens.pop_front() {
                    Some(Token::Command(Cmd::Auto)) => match tokens.pop_front() {
                        Some(Token::ParamBool(b)) if tokens.is_empty() => {
                            self.ctrl.set_access_point_mode(b);
                            ok = true;
                        }
                        _ => println!("## usage: ap auto true|false"), // TODO automatic usage print
                    },
                    Some(Token::Invalid) | None => println!("## usage: ap auto true|false"),
                    Some(_) => panic!(),
                },
                Cmd::Network => match tokens.pop_front() {
                    Some(Token::Command(Cmd::List)) if tokens.is_empty() => {
                        println!("TODO: print network list");
                    }
                    Some(Token::Command(Cmd::Add)) => println!("## unimplemented"),
                    Some(Token::Command(Cmd::Delete)) => match tokens.pop_front() {
                        Some(Token::ParamString(ssid)) if tokens.is_empty() => {
                            println!("TODO: delete network {ssid}");
                        }
                        _ => println!("## usage: network delete <ssid>"),
                    },
                    Some(Token::Invalid) | None => {
                        println!("## usage: network list\n##        network add <ssid> <key>\n##        network delete <ssid>")
                    }
                    Some(_) => panic!(),
                },
                _ => panic!(),
            },
            Some(Token::Invalid) => println!("## unknown command"),
            Some(_) => panic!(),
            None => {}
        }
        ok
    }
}
