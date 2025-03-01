use crate::{
    text_renderer::OptionalTextUpdateData,
    ui::{
        create_app_ui,
        layout::{self, ComponentPosition},
    },
    wgpu_ctx::WgpuCtx,
};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use uuid::Uuid;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    platform::windows::WindowAttributesExtWindows,
    window::{CursorIcon, Theme, Window, WindowId},
};

#[derive(Debug, Clone)]
pub enum AppEvent {
    Close,
    Maximize,
    Minimize,
    ChangeCursorTo(CursorIcon),
    PrintMessage(String),
    SetPositionText(Uuid, ComponentPosition),
    DragWindow(f64, f64), // Add this variant for window dragging
}

#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    cursor_position: Option<(f64, f64)>,
    event_sender: Option<UnboundedSender<AppEvent>>,
    event_receiver: Option<UnboundedReceiver<AppEvent>>,
    layout_context: layout::LayoutContext,
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
                    AppEvent::SetPositionText(id, position) => {
                        if let Some(wgpu_ctx) = &mut self.wgpu_ctx {
                            if let Some(bounds) = wgpu_ctx.text_handler.get_bounds(id) {
                                let mut updated_bounds = bounds;
                                updated_bounds.position = position;
                                wgpu_ctx.text_handler.update((
                                    id,
                                    OptionalTextUpdateData::new().with_bounds(updated_bounds),
                                ));
                                return true;
                            } else {
                                error!("Could not find text with id: {:?}", id);
                            }
                        }
                        return false;
                    }
                    AppEvent::DragWindow(x, y) => {
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
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes()
                .with_title("Frostify")
                .with_decorations(false)
                .with_undecorated_shadow(true)
                .with_transparent(true)
                .with_resizable(true)
                .with_blur(true)
                .with_inner_size(winit::dpi::PhysicalSize::new(1100, 750))
                .with_theme(Some(Theme::Dark))
                .with_system_backdrop(winit::platform::windows::BackdropType::MainWindow)
                .with_corner_preference(winit::platform::windows::CornerPreference::Round);
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
                    info!("Resized to: {:?}", new_size);
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
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some((x, y)) = self.cursor_position {
                    // Convert physical coordinates to logical coordinates
                    if let Some(window) = &self.window {
                        let scale_factor = window.scale_factor();
                        let logical_x = x / scale_factor;
                        let logical_y = y / scale_factor;

                        // Create an input event
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

                        // Process the event through layout context
                        let affected_components = self.layout_context.handle_event(input_event);

                        // Process any components that were affected
                        if !affected_components.is_empty() {
                            self.try_handle_window_event(event_loop);
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
