use helper::{Command, CommandHelper, Param, Sub};
use log::debug;
use rustyline::config::Builder;
use rustyline::error::ReadlineError;
use rustyline::history::MemHistory;
use rustyline::{CompletionType, Editor};

mod helper;

#[derive(Clone, Copy, Debug)]
enum Id {
    Help,
    Network,
    List,
    Add,
    Delete,
}

pub struct CommandLine {
    rl: Editor<CommandHelper<Id>, MemHistory>,
}

impl CommandLine {
    pub fn new() -> Self {
        // TODO let (backend, receiver) = Backend::new();
        let commands = vec![
            Command::new(Id::Help, "help", Sub::None),
            Command::new(
                Id::Network,
                "network",
                Sub::Commands(vec![
                    Command::new(Id::List, "list", Sub::None),
                    Command::new(Id::Add, "add", Sub::Params(vec![Param::string("ssid"), Param::string("key")])),
                    Command::new(Id::Delete, "delete", Sub::Params(vec![Param::string("ssid")])),
                ]),
            ),
        ];
        let mut rl = Editor::with_config(Builder::new().completion_type(CompletionType::List).build()).unwrap();
        rl.set_helper(Some(CommandHelper::new(commands)));
        Self {
            rl,
        }
    }

    pub fn run(&mut self) -> () {
        loop {
            let readline = self.rl.readline(">> ");
            match readline {
                Ok(line) => {
                    //rl.add_history_entry(line.as_str());
                    println!("Line: {}", line);
                    let p = self.rl.helper().unwrap().parse(&line);
                    debug!("{:?}", p);
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }
}
