use crate::{
    color::Color,
    components::{
        button::Button,
        core::{
            root::RootComponent, Anchor, Component, ComponentBackgroundConfig, ComponentOffset,
            ComponentPosition, ComponentSize, ComponentTextOnGradientConfig, ComponentTransform,
        },
    },
    text_renderer::OptionalTextUpdateData,
    ui::navbar::create_navbar,
    wgpu_ctx::WgpuCtx,
};
use std::sync::Arc;
use log::{error, info};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use uuid::Uuid;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::ActiveEventLoop,
    platform::windows::WindowAttributesExtWindows,
    window::{CursorIcon, Theme, Window, WindowId},
};
// use wgpu::rwh::HasWindowHandle;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Close,
    Maximize,
    Minimize,
    ChangeCursorTo(CursorIcon),
    PrintMessage(String),
    SetPositionText(Uuid, ComponentPosition),
}

#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    cursor_position: Option<(f64, f64)>,
    // #[cfg(target_os = "windows")]
    // initial_cloaked: bool,
    event_sender: Option<UnboundedSender<AppEvent>>,
    event_receiver: Option<UnboundedReceiver<AppEvent>>,
    root: RootComponent,
}

impl App<'_> {
    // #[cfg(target_os = "windows")]
    // pub fn new(initial_cloaked: bool) -> Self {
    //     Self {
    //         initial_cloaked,
    //         ..Default::default()
    //     }
    // }

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

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes()
                .with_title("Frostify")
                .with_decorations(false)
                .with_undecorated_shadow(true)
                .with_transparent(true)
                .with_resizable(true)
                .with_theme(Some(Theme::Dark))
                .with_system_backdrop(winit::platform::windows::BackdropType::MainWindow)
                .with_corner_preference(winit::platform::windows::CornerPreference::Round);
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );

            // For Windows, cloak the window if requested
            // set_cloak(true, window.window_handle());

            self.window = Some(window.clone());
            let mut wgpu_ctx = WgpuCtx::new(window.clone());
            self.root.resize(
                &wgpu_ctx,
                wgpu_ctx.surface_config.width,
                wgpu_ctx.surface_config.height,
            );
            // Create event channel
            let (event_tx, event_rx) = unbounded_channel();
            self.event_sender = Some(event_tx.clone());
            self.event_receiver = Some(event_rx);

            let navbar = create_navbar(&mut wgpu_ctx, event_tx.clone(), self.root.get_bounds());
            let normal_btn = Button::new(
                &mut wgpu_ctx,
                ComponentBackgroundConfig::TextOnGradient(ComponentTextOnGradientConfig {
                    text: "Hello".to_string(),
                    text_color: Color::Black,
                    start_color: Color::Bisque,
                    end_color: Color::Beige,
                    anchor: Anchor::Center,
                    angle: 90.0,
                }),
                ComponentTransform {
                    size: ComponentSize {
                        width: self.root.get_bounds().size.width,
                        height: self.root.get_bounds().size.height - 60.0,
                    },
                    offset: ComponentOffset { x: 0.0, y: 60.0 },
                    anchor: Anchor::TopLeft,
                },
                Some(self.root.get_bounds()),
                AppEvent::PrintMessage("Hello".to_string()),
                Some(event_tx),
            );

            self.root.add_child(Box::new(navbar));
            self.root.add_child(Box::new(normal_btn));
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
                    wgpu_ctx.resize((new_size.width, new_size.height), &mut self.root);
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    wgpu_ctx.draw(&mut self.root);
                    wgpu_ctx.text_handler.trim_atlas();
                    // #[cfg(target_os = "windows")]
                    // if self.initial_cloaked {
                    //     set_cloak(false, self.window.as_ref().unwrap().window_handle());
                    // }
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
                if let Some((x, y)) = self.cursor_position {
                    // Use physical coordinates for click handling
                    if let Some(window) = &self.window {
                        let scale_factor = window.scale_factor();
                        let logical_x = x / scale_factor;
                        let logical_y = y / scale_factor;
                        self.root
                            .handle_mouse_click(logical_x as f32, logical_y as f32);
                        if !self.try_handle_window_event(event_loop) {
                            info!("Click at: ({}, {})", logical_x, logical_y);
                            error!("Task failed to handle click");
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

// Very hacky way to cloak the window on Windows, debatable if it's worth it
// #[cfg(target_os = "windows")]
// pub(crate) fn set_cloak(
//     state: bool,
//     window_handle: Result<wgpu::rwh::WindowHandle, wgpu::rwh::HandleError>,
// ) -> bool {
//     use wgpu::rwh::{self};
//     use winapi::shared::minwindef::{BOOL, FALSE, TRUE};
//     use winapi::um::dwmapi::{DwmSetWindowAttribute, DWMWA_CLOAK};

//     let mut result = 1;

//     if let Ok(window_handle) = window_handle {
//         if let rwh::RawWindowHandle::Win32(handle) = window_handle.as_raw() {
//             let value = if state { TRUE } else { FALSE };
//             result = unsafe {
//                 DwmSetWindowAttribute(
//                     handle.hwnd.get() as _, // HWND
//                     DWMWA_CLOAK,
//                     &value as *const BOOL as *const _,
//                     std::mem::size_of::<BOOL>() as u32,
//                 )
//             };
//         }
//     } else {
//         unreachable!();
//     };

//     result == 0 // success
// }
