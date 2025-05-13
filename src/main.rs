// #![cfg_attr(
//     all(target_os = "windows", not(debug_assertions),),
//     windows_subsystem = "windows"
// )]

use crate::{app::App, ui::asset};
use clap::Parser;
use colored::Colorize;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use time::{UtcOffset, macros::format_description};
use ui::UiView;
use winit::{
    error::EventLoopError,
    event_loop::{ControlFlow, EventLoop},
};

mod app;
mod auth;
mod constants;
mod core;
mod errors;
mod test;
mod ui;
mod utils;
mod wgpu_ctx;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Reset stored login data
    #[arg(long, short, action = clap::ArgAction::SetTrue, help = "Reset Frostify config")]
    reset: bool,
    /// Ui test mode
    #[arg(long, short, default_value = None,help = "Run in UI test mode")]
    ui_test: Option<UiView>,
}

fn main() -> Result<(), EventLoopError> {
    // Parse command line arguments
    let args = Args::parse();

    // Handle reset option if specified
    if args.reset {
        match auth::token_manager::delete_tokens() {
            Ok(_) => {
                println!("{}", "Frostify config reset successfully.".green());
                return Ok(());
            }
            Err(e) => {
                println!("{}: {}", "Error resetting Frostify config".red(), e);
                return Ok(());
            }
        }
    }

    if let Some(test_ui_view) = args.ui_test {
        println!("Running in UI test mode: {:?}", test_ui_view);
    }

    // Initialize assets before creating the event loop
    asset::initialize_assets();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // Configure the logger
    let time_format = format_description!("[hour]:[minute]:[second].[subsecond digits:3]");
    let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);

    let mut builder = Builder::new();
    builder
        .format(move |buf, record| {
            let level = match record.level() {
                log::Level::Error => record.level().to_string().red(),
                log::Level::Warn => record.level().to_string().yellow(),
                log::Level::Info => record.level().to_string().green(),
                log::Level::Debug => record.level().to_string().cyan(),
                log::Level::Trace => record.level().to_string().magenta(),
            };
            let now = time::OffsetDateTime::now_utc().to_offset(offset);
            writeln!(
                buf,
                "{} [{}] - {}:{} - {}",
                now.format(&time_format).unwrap(),
                level,
                record
                    .file()
                    .unwrap_or("unknown")
                    .trim_start_matches(&format!("src{}", std::path::MAIN_SEPARATOR)),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter_level(LevelFilter::Off);

    #[cfg(debug_assertions)]
    builder.filter_module("Frostify", LevelFilter::Trace);
    #[cfg(not(debug_assertions))]
    builder.filter_module("Frostify", LevelFilter::Trace);

    builder.init();

    let mut app = App::new(args.ui_test);
    event_loop.run_app(&mut app)
}
