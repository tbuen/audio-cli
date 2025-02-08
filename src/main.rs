use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

use cli::CommandLine;
use control::Controller;

mod cli;
mod control;

fn main() {
    let _ = TermLogger::init(
        LevelFilter::Debug,
        ConfigBuilder::new().add_filter_allow_str("audio_cli").add_filter_allow_str("audio_backend").build(),
        TerminalMode::Stdout,
        ColorChoice::Auto,
    );
    let ctrl = Controller::new();
    println!("{} {}, backend {}", env!("CARGO_PKG_NAME"), env!("VERSION"), ctrl.backend_version());
    CommandLine::new(ctrl).run();
}
