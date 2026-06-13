mod control;
mod interface;

use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

use self::control::Controller;
use self::interface::Cli;

fn main() {
    TermLogger::init(
        LevelFilter::Debug,
        ConfigBuilder::new()
            .add_filter_allow_str("audio_cli")
            .add_filter_allow_str("audio_backend")
            .add_filter_allow_str("smart_repl")
            .add_filter_ignore_str("audio_backend::com::mdns")
            .add_filter_ignore_str("audio_backend::com::websocket")
            .build(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    )
    .unwrap();
    let ctrl = Controller::new();
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("VERSION"));
    Cli::new(&ctrl).run();
}
