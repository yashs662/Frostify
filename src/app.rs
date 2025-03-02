use crate::{
    constants::WINDOW_RESIZE_BORDER_WIDTH,
    ui::{
        create_app_ui,
        layout::{self},
    },
    wgpu_ctx::WgpuCtx,
};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{CursorIcon, ResizeDirection, Theme, Window, WindowId},
};

#[derive(Debug, Clone)]
pub enum AppEvent {
    Close,
    Maximize,
    Minimize,
    ChangeCursorTo(CursorIcon),
    PrintMessage(String),
    DragWindow,
}

#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    cursor_position: Option<(f64, f64)>,
    event_sender: Option<UnboundedSender<AppEvent>>,
    event_receiver: Option<UnboundedReceiver<AppEvent>>,
    layout_context: layout::LayoutContext,
    resize_state: Option<ResizeState>,
}

struct ResizeState {
    direction: ResizeDirection,
}

impl App<'_> {
    fn try_handle_window_event(&mut self, event_loop: &ActiveEventLoop) -> bool {
        if let Some(receiver) = &mut self.event_receiver {
            if let Ok(event) = receiver.try_recv() {
                match event {
                    AppEvent::Close => {
                        event_loop.exit();
                        return true;
                    }
                    AppEvent::Maximize => {
                        if let Some(window) = &self.window {
                            window.set_maximized(!window.is_maximized());
                            return true;
                        }
                    }
                    AppEvent::Minimize => {
                        if let Some(window) = &self.window {
                            window.set_minimized(true);
                            return true;
                        }
                    }
                    AppEvent::ChangeCursorTo(cursor) => {
                        if let Some(window) = &self.window {
                            window.set_cursor(cursor);
                            return true;
                        }
                    }
                    AppEvent::PrintMessage(msg) => {
                        info!("{}", msg);
                        return true;
                    }
                    AppEvent::DragWindow => {
                        if let Some(window) = &self.window {
                            window.drag_window().unwrap_or_else(|e| {
                                error!("Failed to drag window: {}", e);
                            });
                            return true;
                        }
                    }
                }
            } else {
                // no event = success
                return true;
            }
        } else {
            error!("No event receiver");
        }
        false
    }

    fn get_resize_direction(&self, x: f64, y: f64) -> Option<ResizeDirection> {
        if let Some(window) = &self.window {
            let size = window.outer_size();

            let is_left = x <= WINDOW_RESIZE_BORDER_WIDTH;
            let is_right = x >= size.width as f64 - WINDOW_RESIZE_BORDER_WIDTH;
            let is_top = y <= WINDOW_RESIZE_BORDER_WIDTH;
            let is_bottom = y >= size.height as f64 - WINDOW_RESIZE_BORDER_WIDTH;

            match (is_left, is_right, is_top, is_bottom) {
                (true, false, true, false) => Some(ResizeDirection::NorthWest),
                (false, true, true, false) => Some(ResizeDirection::NorthEast),
                (true, false, false, true) => Some(ResizeDirection::SouthWest),
                (false, true, false, true) => Some(ResizeDirection::SouthEast),
                (true, false, false, false) => Some(ResizeDirection::West),
                (false, true, false, false) => Some(ResizeDirection::East),
                (false, false, true, false) => Some(ResizeDirection::North),
                (false, false, false, true) => Some(ResizeDirection::South),
                _ => None,
            }
        } else {
            None
        }
    }

    fn update_resize_cursor(&self, x: f64, y: f64) {
        if let Some(window) = &self.window {
            let cursor = match self.get_resize_direction(x, y) {
                Some(ResizeDirection::NorthWest) => CursorIcon::NwResize,
                Some(ResizeDirection::NorthEast) => CursorIcon::NeResize,
                Some(ResizeDirection::SouthWest) => CursorIcon::SwResize,
                Some(ResizeDirection::SouthEast) => CursorIcon::SeResize,
                Some(ResizeDirection::West) => CursorIcon::WResize,
                Some(ResizeDirection::East) => CursorIcon::EResize,
                Some(ResizeDirection::North) => CursorIcon::NResize,
                Some(ResizeDirection::South) => CursorIcon::SResize,
                None => CursorIcon::Default,
            };
            window.set_cursor(cursor);
        }
    }

    fn is_in_resize_zone(&self, x: f64, y: f64) -> bool {
        self.get_resize_direction(x, y).is_some()
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let mut win_attr = Window::default_attributes()
                .with_title("Frostify")
                .with_decorations(false)
                .with_transparent(true)
                .with_resizable(true)
                .with_blur(true)
                .with_inner_size(winit::dpi::PhysicalSize::new(1100, 750))
                .with_theme(Some(Theme::Dark));

            #[cfg(target_os = "windows")]
            {
                use winit::platform::windows::WindowAttributesExtWindows;

                win_attr = win_attr
                    .with_system_backdrop(winit::platform::windows::BackdropType::TransientWindow)
                    .with_corner_preference(winit::platform::windows::CornerPreference::Round);
            }

            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );

            self.window = Some(window.clone());
            let mut wgpu_ctx = WgpuCtx::new(window.clone());

            // Initialize layout context before creating UI
            self.layout_context.initialize(
                wgpu_ctx.surface_config.width as f32,
                wgpu_ctx.surface_config.height as f32,
            );

            // Create event channel
            let (event_tx, event_rx) = unbounded_channel();
            self.event_sender = Some(event_tx.clone());
            self.event_receiver = Some(event_rx);

            create_app_ui(&mut wgpu_ctx, event_tx, &mut self.layout_context);

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
                    self.layout_context.resize_viewport(
                        new_size.width as f32,
                        new_size.height as f32,
                        wgpu_ctx,
                    );
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    wgpu_ctx.draw(&mut self.layout_context);
                    wgpu_ctx.text_handler.trim_atlas();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(window) = &self.window {
                    self.cursor_position = Some((position.x, position.y));

                    if let Some(resize_state) = &self.resize_state {
                        window
                            .drag_resize_window(resize_state.direction)
                            .unwrap_or_else(|e| {
                                error!("Failed to resize window: {}", e);
                            });
                    } else {
                        self.update_resize_cursor(position.x, position.y);
                    }

                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some((x, y)) = self.cursor_position {
                    match state {
                        winit::event::ElementState::Pressed => {
                            if let Some(direction) = self.get_resize_direction(x, y) {
                                self.resize_state = Some(ResizeState { direction });
                                return;
                            }
                        }
                        winit::event::ElementState::Released => {
                            if self.resize_state.is_some() {
                                self.resize_state = None;
                                self.update_resize_cursor(x, y);
                                return;
                            }
                        }
                    }

                    if !self.is_in_resize_zone(x, y) {
                        // Convert physical coordinates to logical coordinates for UI interactions
                        if let Some(window) = &self.window {
                            let scale_factor = window.scale_factor();
                            let logical_x = x / scale_factor;
                            let logical_y = y / scale_factor;

                            let input_event = layout::InputEvent {
                                event_type: layout::EventType::from(state),
                                position: Some(layout::ComponentPosition {
                                    x: logical_x as f32,
                                    y: logical_y as f32,
                                }),
                                button,
                                key: None,
                                text: None,
                            };

                            let affected_components = self.layout_context.handle_event(input_event);
                            if !affected_components.is_empty() {
                                self.try_handle_window_event(event_loop);
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
