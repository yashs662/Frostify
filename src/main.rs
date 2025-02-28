use crate::app::App;
use colored::*;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use time::{macros::format_description, UtcOffset};
use winit::{
    error::EventLoopError,
    event_loop::{ControlFlow, EventLoop},
};

mod app;
mod color;
mod constants;
mod img_utils;
mod text_renderer;
mod ui;
mod vertex;
mod wgpu_ctx;

fn main() -> Result<(), EventLoopError> {
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
    builder.filter_module("Frostify", LevelFilter::Warn);

    builder.init();

    let mut app = App::default();
    event_loop.run_app(&mut app)
}
