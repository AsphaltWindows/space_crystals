//! `cargo run -- --help` lists the flags.

use std::process::ExitCode;

use bevy::app::AppExit;
use bevy::log::LogPlugin;
use space_crystals::platform::DisplayMode;
use space_crystals::{build_app, cli};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let options = match cli::parse(&args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("error: {error}\n\n{}", cli::usage());
            return ExitCode::from(2);
        }
    };
    if options.help {
        println!("{}", cli::usage());
        return ExitCode::SUCCESS;
    }
    let config = match cli::into_config(options) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::from(2);
        }
    };
    let headless = config.display == DisplayMode::Headless;
    let mut app = build_app(config);
    if headless {
        // The other display modes log through `DefaultPlugins`. Tests build headless apps without it: the logger is
        // global, and every app after the first would report that it is already set.
        app.add_plugins(LogPlugin::default());
    }
    match app.run() {
        AppExit::Success => ExitCode::SUCCESS,
        AppExit::Error(code) => ExitCode::from(code.get()),
    }
}
