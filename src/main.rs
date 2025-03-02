use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

use command::Interface;
use control::Controller;

mod command;
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
    Interface::new(ctrl).run();
}
