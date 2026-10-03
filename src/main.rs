#![allow(clippy::print_stdout)]
mod control;
mod interface;

use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

use crate::control::Controller;
use crate::interface::Cli;

fn main() {
    TermLogger::init(
        LevelFilter::Warn,
        ConfigBuilder::new()
            .add_filter_allow_str("audio_cli")
            .add_filter_allow_str("audio_backend")
            .add_filter_allow_str("smart_repl")
            .build(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    )
    .unwrap();
    let ctrl = Controller::new();
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("VERSION"));
    Cli::new(&ctrl).run();
}
