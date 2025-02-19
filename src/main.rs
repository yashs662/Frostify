use crate::app::App;
use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod wgpu_ctx;
mod vertex;
mod img_utils;
mod components;

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    // #[cfg(target_os = "windows")]
    // let mut app = App::new(true);
    // #[cfg(not(target_os = "windows"))]
    let mut app = App::default();
    event_loop.run_app(&mut app)
}