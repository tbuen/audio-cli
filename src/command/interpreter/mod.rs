use helper::{CommandHelper, Token};
use log::error;
use rustyline::config::Builder;
use rustyline::error::ReadlineError;
use rustyline::history::MemHistory;
use rustyline::{CompletionType, Editor};
use std::collections::VecDeque;

mod helper;

pub trait Id: ToString + Clone {}

pub enum CallbackParameter {
    String(String),
    Bool(bool),
}

type Callback = Box<dyn Fn(Vec<CallbackParameter>)>;

pub struct Command<T: Id> {
    id: T,
    sub: Sub<T>,
}

pub enum Param {
    String(String),
    Bool,
}

pub enum Sub<T: Id> {
    Commands(Vec<Command<T>>),
    Params(Vec<Param>),
    None(Callback),
}

pub struct Interpreter<T: Id> {
    rl: Editor<CommandHelper<T>, MemHistory>,
}

impl<T: Id> Interpreter<T> {
    pub fn new(commands: Vec<Command<T>>) -> Self {
        let mut rl = Editor::with_config(Builder::new().completion_type(CompletionType::List).build()).unwrap();
        rl.set_helper(Some(CommandHelper::new(commands)));
        Self {
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

    fn interpret(&self, tokens: VecDeque<Token<T>>) -> bool {
        let mut ok = true;
        let mut commands = vec![];
        let mut params = vec![];
        let mut callback = None;
        for token in tokens {
            match token {
                Token::Command(cmd) => commands.push(cmd),
                Token::ParamString(val) => params.push(CallbackParameter::String(val)),
                Token::ParamBool(val) => params.push(CallbackParameter::Bool(val)),
                Token::Callback(cb) => callback = Some(cb),
                _ => ok = false,
            }
        }
        if ok {
            if let Some(cb) = callback {
                cb(params);
            } else {
                println!("unimplemented");
                ok = false;
            }
        } else {
            println!("TODO: print usage");
        }
        ok
    }

    /*fn interpret(&self, mut tokens: VecDeque<Token<Cmd>>) -> bool {
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
    }*/
}
