use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

use control::Controller;
use interface::Cli;

mod control;
mod interface;

fn main() {
    TermLogger::init(
        LevelFilter::Debug,
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
