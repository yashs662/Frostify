use crate::components::bounds::Anchor;
use crate::components::button::Button;
use crate::components::container::{Container, FlexAlign, FlexDirection};
use crate::components::{Component, ComponentOffset, ComponentSize, ComponentTransform};
use crate::wgpu_ctx::WgpuCtx;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
// use wgpu::rwh::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::{Theme, Window, WindowId};

#[derive(Debug)]
pub enum AppWindowEvents {
    Close,
    Maximize,
    Minimize,
}

#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    cursor_position: Option<(f64, f64)>,
    // #[cfg(target_os = "windows")]
    // initial_cloaked: bool,
    event_sender: Option<Sender<AppWindowEvents>>,
    event_receiver: Option<Receiver<AppWindowEvents>>,
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
                    AppWindowEvents::Close => {
                        event_loop.exit();
                        return true;
                    }
                    AppWindowEvents::Maximize => {
                        if let Some(window) = &self.window {
                            window.set_maximized(!window.is_maximized());
                            return true;
                        }
                    }
                    AppWindowEvents::Minimize => {
                        if let Some(window) = &self.window {
                            window.set_minimized(true);
                            return true;
                        }
                    }
                }
            }
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

            // Create event channel
            let (event_tx, event_rx) = channel(1);
            self.event_sender = Some(event_tx.clone());
            self.event_receiver = Some(event_rx);

            let close_tx = event_tx.clone();
            let maximize_tx = event_tx.clone();

            let mut window_ctrl_container = Container::new(
                ComponentTransform {
                    size: ComponentSize {
                        width: 120.0,
                        height: 40.0,
                    },
                    offset: ComponentOffset { x: 0.0, y: 0.0 },
                    anchor: Anchor::TopRight,
                },
                Some(wgpu_ctx.root.get_bounds()),
                FlexDirection::Row,
                FlexAlign::End,
                FlexAlign::SpaceBetween,
            ).with_padding(2.0).with_spacing(5.0);

            let minimize_btn = Button::new(
                &wgpu_ctx.device,
                &wgpu_ctx.queue,
                "assets/minus.png",
                ComponentTransform {
                    size: ComponentSize {
                        width: 32.0,
                        height: 32.0,
                    },
                    offset: ComponentOffset { x: 0.0, y: 0.0 },
                    anchor: Anchor::Center,
                },
                Some(window_ctrl_container.get_bounds()),
                Box::new(move || {
                    let _ = event_tx.blocking_send(AppWindowEvents::Minimize);
                }),
            );

            let maximize_btn = Button::new(
                &wgpu_ctx.device,
                &wgpu_ctx.queue,
                "assets/expand.png",
                ComponentTransform {
                    size: ComponentSize {
                        width: 32.0,
                        height: 32.0,
                    },
                    offset: ComponentOffset { x: 0.0, y: 0.0 },
                    anchor: Anchor::Center,
                },
                Some(window_ctrl_container.get_bounds()),
                Box::new(move || {
                    let _ = maximize_tx.blocking_send(AppWindowEvents::Maximize);
                }),
            );

            let close_btn = Button::new(
                &wgpu_ctx.device,
                &wgpu_ctx.queue,
                "assets/close.png",
                ComponentTransform {
                    size: ComponentSize {
                        width: 32.0,
                        height: 32.0,
                    },
                    offset: ComponentOffset { x: 0.0, y: 0.0 },
                    anchor: Anchor::Center,
                },
                Some(window_ctrl_container.get_bounds()),
                Box::new(move || {
                    let _ = close_tx.blocking_send(AppWindowEvents::Close);
                }),
            );

            window_ctrl_container.add_child(Box::new(minimize_btn));
            window_ctrl_container.add_child(Box::new(maximize_btn));
            window_ctrl_container.add_child(Box::new(close_btn));

            let normal_btn = Button::new(
                &wgpu_ctx.device,
                &wgpu_ctx.queue,
                "assets/test.png",
                ComponentTransform {
                    size: ComponentSize {
                        width: 100.0,
                        height: 100.0,
                    },
                    offset: ComponentOffset { x: 0.0, y: 0.0 },
                    anchor: Anchor::Center,
                },
                Some(wgpu_ctx.root.get_bounds()),
                Box::new(|| println!("Button clicked!")),
            );

            wgpu_ctx.root.add_child(Box::new(window_ctrl_container));
            wgpu_ctx.add_component(Box::new(normal_btn));

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
                if let (Some((x, y)), Some(wgpu_ctx)) = (self.cursor_position, &mut self.wgpu_ctx) {
                    wgpu_ctx.root.handle_mouse_click(x as f32, y as f32);
                    if self.try_handle_window_event(event_loop) {
                        return;
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
