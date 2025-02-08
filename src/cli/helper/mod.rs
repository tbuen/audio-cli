use rustyline::completion::{Completer, Pair};
use rustyline::{Context, Helper, Highlighter, Hinter, Validator};
use std::collections::VecDeque;
use tokenizer::tokenize;

mod tokenizer;

const TRUE: &str = "true";
const FALSE: &str = "false";

pub trait Id: ToString + Clone {}

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
    None,
}

#[derive(Debug)]
pub enum Token<T: Id> {
    Command(T),
    ParamString(String),
    ParamBool(bool),
    Invalid,
}

#[derive(Debug)]
pub enum Error {
    Parse,
}

impl<T: Id> Command<T> {
    pub fn new(id: T, sub: Sub<T>) -> Self {
        Self {
            id,
            sub,
        }
    }
}

impl Param {
    pub fn string(description: &str) -> Self {
        Self::String(description.into())
    }
}

#[derive(Helper, Highlighter, Hinter, Validator)]
pub struct CommandHelper<T: Id> {
    subs: Sub<T>,
}

impl<T: Id> CommandHelper<T> {
    pub fn new(cmds: Vec<Command<T>>) -> Self {
        Self {
            subs: Sub::Commands(cmds),
        }
    }

    pub fn parse(&self, line: &str) -> Result<VecDeque<Token<T>>, Error> {
        if let Some(mut tokens) = tokenize(line) {
            let mut parse_result = VecDeque::new();
            let mut subs = &self.subs;
            loop {
                match &subs {
                    Sub::Commands(commands) => match tokens.pop_front() {
                        Some(token) => match commands.iter().find(|cmd| cmd.id.to_string() == token.text && !token.quoted) {
                            Some(c) => {
                                parse_result.push_back(Token::Command(c.id.clone()));
                                subs = &c.sub;
                            }
                            None => {
                                parse_result.push_back(Token::Invalid);
                                for _ in tokens {
                                    parse_result.push_back(Token::Invalid);
                                }
                                break;
                            }
                        },
                        None => break,
                    },
                    Sub::Params(params) => {
                        for param in params {
                            match param {
                                Param::String(_) => match tokens.pop_front() {
                                    Some(token) => parse_result.push_back(Token::ParamString(token.text)),
                                    None => break,
                                },
                                Param::Bool => match tokens.pop_front() {
                                    Some(token) => parse_result.push_back(if token.text == TRUE {
                                        Token::ParamBool(true)
                                    } else if token.text == FALSE {
                                        Token::ParamBool(false)
                                    } else {
                                        Token::Invalid
                                    }),
                                    None => break,
                                },
                            }
                        }
                        for _ in tokens {
                            parse_result.push_back(Token::Invalid);
                        }
                        break;
                    }
                    Sub::None => {
                        for _ in tokens {
                            parse_result.push_back(Token::Invalid);
                        }
                        break;
                    }
                }
            }
            Ok(parse_result)
        } else {
            Err(Error::Parse)
        }
    }
}

impl<T: Id> Completer for CommandHelper<T> {
    type Candidate = Pair;
    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let line = &line[..pos];
        let mut pairs = vec![];
        let mut rpos = 0;
        if let Some(mut tokens) = tokenize(line) {
            let mut subs = &self.subs;
            loop {
                match &subs {
                    Sub::Commands(cmds) => match tokens.pop_front() {
                        Some(token) if !token.quoted => {
                            if let Some(cmd) = cmds.iter().find(|c| c.id.to_string() == token.text && line.len() > token.end) {
                                subs = &cmd.sub;
                            } else if tokens.is_empty() {
                                for cmd in cmds.iter().filter(|c| c.id.to_string().starts_with(&token.text)) {
                                    pairs.push(Pair {
                                        display: cmd.id.to_string(),
                                        replacement: cmd.id.to_string().clone() + " ",
                                    });
                                }
                                rpos = token.begin;
                                break;
                            } else {
                                break;
                            }
                        }
                        Some(_) => break,
                        None => {
                            for cmd in cmds {
                                pairs.push(Pair {
                                    display: cmd.id.to_string(),
                                    replacement: cmd.id.to_string() + " ",
                                });
                            }
                            rpos = pos;
                            break;
                        }
                    },
                    Sub::Params(params) => {
                        for param in params {
                            match param {
                                Param::String(descr) => match tokens.pop_front() {
                                    Some(_) => {}
                                    None => {
                                        if line.ends_with(" ") {
                                            pairs.push(Pair {
                                                display: format!("<{descr}> "),
                                                replacement: format!("<{descr}> "),
                                            });
                                            rpos = pos;
                                        }
                                        break;
                                    }
                                },
                                Param::Bool => match tokens.pop_front() {
                                    Some(token) if !token.quoted => {
                                        if (token.text == TRUE || token.text == FALSE) && line.len() > token.end {
                                        } else if TRUE.starts_with(&token.text) {
                                            pairs.push(Pair {
                                                display: String::from(TRUE),
                                                replacement: String::from(TRUE) + " ",
                                            });
                                            rpos = token.begin;
                                            break;
                                        } else if FALSE.starts_with(&token.text) {
                                            pairs.push(Pair {
                                                display: String::from(FALSE),
                                                replacement: String::from(FALSE) + " ",
                                            });
                                            rpos = token.begin;
                                            break;
                                        } else {
                                            break;
                                        }
                                    }
                                    Some(_) => break,
                                    None => {
                                        pairs.push(Pair {
                                            display: String::from(TRUE),
                                            replacement: String::from(TRUE) + " ",
                                        });
                                        pairs.push(Pair {
                                            display: String::from(FALSE),
                                            replacement: String::from(FALSE) + " ",
                                        });
                                        rpos = pos;
                                        break;
                                    }
                                },
                            }
                        }
                        break;
                    }
                    Sub::None => break,
                }
            }
        }
        Ok((rpos, pairs))
    }
}
