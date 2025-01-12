use rustyline::completion::{Completer, Pair};
use rustyline::{Context, Helper, Highlighter, Hinter, Validator};
use tokenizer::tokenize;

mod tokenizer;

// TODO: pub trait GroupId {} ??

pub struct Command<T: Clone> {
    id: T,
    name: String,
    sub: Sub<T>,
}

pub enum Param {
    String(String),
}

pub enum Sub<T: Clone> {
    Commands(Vec<Command<T>>),
    Params(Vec<Param>),
    None,
}

#[derive(Debug)]
pub enum Token<T: Clone> {
    Command(T),
    Param(String),
    Invalid(String),
}

#[derive(Debug)]
pub enum Error {
    Parse,
}

impl<T: Clone> Command<T> {
    pub fn new(id: T, name: &str, sub: Sub<T>) -> Self {
        Self {
            id,
            name: name.into(),
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
pub struct CommandHelper<T: Clone> {
    subs: Sub<T>,
}

impl<T: Clone> CommandHelper<T> {
    pub fn new(cmds: Vec<Command<T>>) -> Self {
        Self {
            subs: Sub::Commands(cmds),
        }
    }

    pub fn parse(&self, line: &str) -> Result<Vec<Token<T>>, Error> {
        if let Some(mut tokens) = tokenize(line) {
            let mut parse_result = Vec::new();
            let mut subs = &self.subs;
            loop {
                match &subs {
                    Sub::Commands(commands) => match tokens.pop_front() {
                        Some(token) => match commands.iter().find(|cmd| cmd.name == token.text && !token.quoted) {
                            Some(c) => {
                                parse_result.push(Token::Command(c.id.clone()));
                                subs = &c.sub;
                            }
                            None => {
                                parse_result.push(Token::Invalid(token.text));
                                for token in tokens {
                                    parse_result.push(Token::Invalid(token.text));
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
                                    Some(token) => parse_result.push(Token::Param(token.text)),
                                    None => break,
                                },
                            }
                        }
                        for token in tokens {
                            parse_result.push(Token::Invalid(token.text));
                        }
                        break;
                    }
                    Sub::None => {
                        for token in tokens {
                            parse_result.push(Token::Invalid(token.text));
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

impl<T: Clone> Completer for CommandHelper<T> {
    type Candidate = Pair;
    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let line = &line[..pos];
        let mut pairs = vec![];
        let mut rpos = 0;
        if let Some(mut tokens) = tokenize(line) {
            let mut subs = &self.subs;
            loop {
                match &subs {
                    Sub::Commands(commands) => match tokens.pop_front() {
                        Some(token) if !token.quoted => {
                            if let Some(cmd) = commands.iter().find(|c| c.name == token.text && line.len() > token.end) {
                                subs = &cmd.sub;
                            } else if tokens.is_empty() {
                                for cmd in commands.iter().filter(|c| c.name.starts_with(&token.text)) {
                                    pairs.push(Pair {
                                        display: cmd.name.clone(),
                                        replacement: cmd.name.clone() + " ",
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
                            for cmd in commands {
                                pairs.push(Pair {
                                    display: cmd.name.clone(),
                                    replacement: cmd.name.clone() + " ",
                                });
                            }
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
