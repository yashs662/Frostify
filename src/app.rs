use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::components::bounds::Anchor;
use crate::components::button::Button;
use crate::components::{Component, ComponentOffset, ComponentSize, ComponentTransform};
use crate::wgpu_ctx::WgpuCtx;

#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    cursor_position: Option<(f64, f64)>,
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("Spotify-rs");
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );
            self.window = Some(window.clone());
            let mut wgpu_ctx = WgpuCtx::new(window.clone());

            // Create button with anchor-based positioning
            let btn_transform = ComponentTransform {
                size: ComponentSize {
                    width: 100.0,
                    height: 50.0,
                },
                offset: ComponentOffset { x: 0.0, y: 0.0 },
                anchor: Anchor::Center,
            };
            let button = Button::new(
                &wgpu_ctx.device,
                &wgpu_ctx.queue,
                "assets/test.png",
                btn_transform,
                Some(wgpu_ctx.root.get_bounds()), // parent bounds
                Box::new(|| println!("Button clicked!")),
            );
            wgpu_ctx.add_component(Box::new(button));

            self.wgpu_ctx = Some(wgpu_ctx);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let (Some(wgpu_ctx), Some(window)) =
                    (self.wgpu_ctx.as_mut(), self.window.as_ref())
                {
                    wgpu_ctx.resize((new_size.width, new_size.height));
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    wgpu_ctx.draw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(window) = &self.window {
                    self.cursor_position = Some((position.x, position.y));
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let (Some((x, y)), Some(wgpu_ctx)) = (self.cursor_position, &mut self.wgpu_ctx) {
                    wgpu_ctx.root.handle_mouse_click(x as f32, y as f32);
                }
            }
            _ => (),
        }
    }
}
