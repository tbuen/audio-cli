use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

use cli::CommandLine;

mod cli;

fn main() {
    let _ = TermLogger::init(LevelFilter::Debug, ConfigBuilder::new().add_filter_allow_str("audio_cli").build(), TerminalMode::Stdout, ColorChoice::Auto);
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    CommandLine::new().run();
}
