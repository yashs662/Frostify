use crate::{
    auth::oauth::SpotifyAuthResponse,
    constants::{BACKGROUND_FPS, WINDOW_RESIZE_BORDER_WIDTH},
    core::worker::{Worker, WorkerResponse},
    ui::{
        UiView, create_app_ui, create_login_ui,
        layout::{self, ComponentPosition, EventType},
    },
    wgpu_ctx::WgpuCtx,
};
use log::{debug, error, info};
use std::{sync::Arc, time::Instant};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{CursorIcon, Icon, ResizeDirection, Theme, Window, WindowId},
};

#[derive(Debug, Clone)]
pub enum AppEvent {
    Close,
    Maximize,
    Minimize,
    DragWindow,
    Login,
    Shuffle,
    Repeat,
    PlayPause,
    PreviousTrack,
    NextTrack,
}

#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    event_sender: Option<UnboundedSender<AppEvent>>,
    event_receiver: Option<UnboundedReceiver<AppEvent>>,
    layout_context: layout::LayoutContext,
    app_state: AppState,
    worker: Option<Worker>,
    last_cursor_input: (Option<ElementState>, ComponentPosition),
    frame_counter: FrameCounter,
}

#[derive(Default)]
pub struct AppState {
    resize_state: Option<ResizeState>,
    auth_state: Option<SpotifyAuthResponse>,
    current_view: Option<UiView>,
    is_checking_auth: bool,
    cursor_position: Option<(f64, f64)>,
}

struct FrameCounter {
    last_printed_instant: Instant,
    last_draw_instant: Instant,
    frame_count: u32,
    frame_time: f32,
    avg_fps: f32,
    report_interval: f32,
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl FrameCounter {
    fn new(report_interval: f32) -> Self {
        Self {
            last_printed_instant: Instant::now(),
            last_draw_instant: Instant::now(),
            frame_count: 0,
            frame_time: 0.0,
            avg_fps: 0.0,
            report_interval,
        }
    }

    fn update(&mut self) {
        self.frame_count += 1;
        let new_instant = Instant::now();
        let frame_delta = (new_instant - self.last_draw_instant).as_secs_f32();
        self.frame_time = frame_delta;

        let elapsed_secs = (new_instant - self.last_printed_instant).as_secs_f32();
        if elapsed_secs > self.report_interval {
            let fps = self.frame_count as f32 / elapsed_secs;
            log::info!("Frame time {:.2}ms ({:.1} FPS)", frame_delta * 1000.0, fps);

            self.last_printed_instant = new_instant;
            self.frame_count = 0;
            self.avg_fps = fps;
        }
        self.last_draw_instant = new_instant;
    }

    fn limit_fps(&self, target_fps: u32) {
        let target_frame_time = 1000 / target_fps;
        let elapsed = self.last_draw_instant.elapsed();
        if elapsed.as_millis() < target_frame_time as u128 {
            std::thread::sleep(std::time::Duration::from_millis(
                (target_frame_time - elapsed.as_millis() as u32) as u64,
            ));
        }
    }

    fn get_frame_time(&self) -> f32 {
        self.frame_time
    }
}

struct ResizeState {
    direction: ResizeDirection,
}

impl App<'_> {
    fn try_handle_app_event(&mut self, event_loop: &ActiveEventLoop) -> bool {
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
                            // Reset hover states when window state changes
                            self.layout_context.reset_all_hover_states();
                            return true;
                        }
                    }
                    AppEvent::Minimize => {
                        if let Some(window) = &self.window {
                            window.set_minimized(true);
                            // Reset hover states when window state changes
                            self.layout_context.reset_all_hover_states();
                            return true;
                        }
                    }
                    AppEvent::DragWindow => {
                        if let Some(window) = &self.window {
                            if window.is_maximized() {
                                if let Some(cursor_position) = self.app_state.cursor_position {
                                    let old_window_size = window.inner_size();
                                    let x_ratio = cursor_position.0 / old_window_size.width as f64;
                                    window.set_maximized(false);
                                    let new_window_size = window.inner_size();
                                    window.set_outer_position(winit::dpi::PhysicalPosition::new(
                                        cursor_position.0
                                            - (new_window_size.width as f64 * x_ratio),
                                        cursor_position.1 - 20.0,
                                    ));
                                }
                            }
                            window.drag_window().unwrap_or_else(|e| {
                                error!("Failed to drag window: {}", e);
                            });
                            return true;
                        }
                    }
                    AppEvent::Login => {
                        debug!("Login event received");
                        if let Some(worker) = &self.worker {
                            worker.start_oauth();
                        }
                        return true;
                    }
                    AppEvent::Shuffle => {
                        debug!("Shuffle event received");
                        return true;
                    }
                    AppEvent::Repeat => {
                        debug!("Repeat event received");
                        return true;
                    }
                    AppEvent::PlayPause => {
                        debug!("Play/Pause event received");
                        return true;
                    }
                    AppEvent::PreviousTrack => {
                        debug!("Previous track event received");
                        return true;
                    }
                    AppEvent::NextTrack => {
                        debug!("Next track event received");
                        return true;
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

    pub fn change_view(
        wgpu_ctx: &mut Option<WgpuCtx>,
        layout_context: &mut layout::LayoutContext,
        app_state: &mut AppState,
        event_sender: UnboundedSender<AppEvent>,
        view: UiView,
    ) {
        if let Some(wgpu_ctx) = wgpu_ctx {
            layout_context.clear();
            match view {
                UiView::Login => {
                    create_login_ui(wgpu_ctx, event_sender, layout_context);
                }
                UiView::Home => {
                    create_app_ui(wgpu_ctx, event_sender, layout_context);
                }
            }

            // Apply multiple viewport resizes to ensure correct positioning
            App::apply_layout_updates(wgpu_ctx, layout_context);
            app_state.current_view = Some(view);
        }
    }

    /// Helper method to apply multiple viewport resizes to ensure proper layout
    fn apply_layout_updates(wgpu_ctx: &mut WgpuCtx, layout_context: &mut layout::LayoutContext) {
        // Done 3 times to ensure components with FlexValue::Fit have their positions calculated correctly
        for _ in 0..3 {
            layout_context.compute_layout_and_update_components(wgpu_ctx);
        }
    }

    /// Handles UI events and returns whether any components were affected
    fn handle_ui_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        x: f64,
        y: f64,
        state: Option<winit::event::ElementState>,
        button: Option<winit::event::MouseButton>,
        scroll_delta: Option<f32>,
    ) {
        // Convert physical coordinates to logical coordinates for UI interactions
        let mouse_position = ComponentPosition {
            x: x as f32,
            y: y as f32,
        };

        // Improved event detection logic
        let event_type = if scroll_delta.is_some() {
            // Prioritize scroll events over others when scroll_delta is provided
            if let Some(delta) = scroll_delta {
                if delta > 0.0 {
                    EventType::ScrollDown
                } else {
                    EventType::ScrollUp
                }
            } else {
                EventType::None
            }
        } else if button.is_some() {
            // Mouse button events
            if state == Some(ElementState::Pressed) {
                EventType::Press
            } else {
                EventType::Release
            }
        } else if self.last_cursor_input.0 == Some(ElementState::Pressed) {
            // This is a drag event - mouse is moving while button is pressed
            EventType::Drag
        } else if button.is_none() && state.is_none() {
            // No button or scroll events - must be hover
            EventType::Hover
        } else {
            EventType::None
        };

        let input_event = layout::InputEvent {
            event_type: event_type.clone(),
            position: Some(mouse_position),
            button,
            key: None,
            text: None,
        };

        // Update the last known input state
        if button.is_some() {
            // Only update button state for actual button events
            self.last_cursor_input = (state, mouse_position);
        } else {
            // For non-button events, preserve the button state but update position
            self.last_cursor_input.1 = mouse_position;
        }

        let affected_component = self.layout_context.handle_event(input_event);

        if let Some((_affected_component_id, _event_type, app_event)) = affected_component {
            if app_event.is_some() {
                if let Some(event_sender) = &self.event_sender {
                    event_sender.send(app_event.unwrap()).unwrap_or_else(|e| {
                        error!("Failed to send app event: {}", e);
                    });
                }
            }
        }

        // Always check for events regardless of affected components
        self.try_handle_app_event(event_loop);
    }

    fn check_worker_responses(&mut self) {
        if let Some(worker) = &mut self.worker {
            while let Some(response) = worker.poll_responses() {
                match response {
                    WorkerResponse::OAuthStarted { auth_url } => {
                        debug!("OAuth flow started, URL: {}", auth_url);
                        webbrowser::open(&auth_url).unwrap_or_else(|e| {
                            error!("Failed to open browser: {}", e);
                        });
                    }
                    WorkerResponse::OAuthComplete { auth_response } => {
                        info!("OAuth flow completed successfully");
                        self.app_state.auth_state = Some(auth_response);
                        App::change_view(
                            &mut self.wgpu_ctx,
                            &mut self.layout_context,
                            &mut self.app_state,
                            self.event_sender.as_ref().unwrap().clone(),
                            UiView::Home,
                        );

                        // focus on the window after login
                        if let Some(window) = &self.window {
                            window.focus_window();
                        }
                    }
                    WorkerResponse::OAuthFailed { error } => {
                        error!("OAuth flow failed: {}", error);
                        // Stay on the login screen
                        self.app_state.is_checking_auth = false;
                    }
                    WorkerResponse::TokensLoaded { auth_response } => {
                        info!("Loaded stored tokens");
                        self.app_state.auth_state = Some(auth_response);
                        self.app_state.is_checking_auth = false;

                        App::change_view(
                            &mut self.wgpu_ctx,
                            &mut self.layout_context,
                            &mut self.app_state,
                            self.event_sender.as_ref().unwrap().clone(),
                            UiView::Home,
                        );
                    }
                    WorkerResponse::NoStoredTokens => {
                        debug!("No stored tokens found, showing login screen");
                        self.app_state.is_checking_auth = false;

                        // Make sure we're in the login view
                        if self.app_state.current_view != Some(UiView::Login) {
                            App::change_view(
                                &mut self.wgpu_ctx,
                                &mut self.layout_context,
                                &mut self.app_state,
                                self.event_sender.as_ref().unwrap().clone(),
                                UiView::Login,
                            );
                        }
                    }
                }
            }
        }
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
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let icon = load_icon(include_bytes!("../assets/frostify_logo.ico"));
            // allow unused_mut to avoid warnings on systems other than windows
            #[allow(unused_mut)]
            let mut win_attr = Window::default_attributes()
                .with_title("Frostify")
                .with_window_icon(Some(icon.clone()))
                .with_decorations(false)
                .with_transparent(true)
                .with_resizable(true)
                .with_min_inner_size(winit::dpi::PhysicalSize::new(800, 600))
                .with_blur(true)
                .with_inner_size(winit::dpi::PhysicalSize::new(1100, 750))
                .with_visible(false) // Start with window invisible
                .with_theme(Some(Theme::Dark));

            #[cfg(target_os = "windows")]
            {
                use winit::platform::windows::WindowAttributesExtWindows;

                win_attr = win_attr
                    .with_taskbar_icon(Some(icon))
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

            // Initialize the worker thread
            self.worker = Some(Worker::new());

            // Start with login UI by default
            create_login_ui(&mut wgpu_ctx, event_tx, &mut self.layout_context);

            self.wgpu_ctx = Some(wgpu_ctx);

            // Check for stored tokens as soon as the app starts
            if let Some(worker) = &self.worker {
                info!("Checking for stored tokens...");
                self.app_state.is_checking_auth = true;
                worker.try_load_tokens();
            }

            // Draw the first frame before making the window visible
            if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                self.layout_context.update_components(wgpu_ctx, 0.0);
                wgpu_ctx.draw(&mut self.layout_context);

                // Now that we've drawn the first frame, make the window visible
                if let Some(window) = &self.window {
                    window.set_visible(true);
                }
            }
        }

        // Check for any pending events after resuming
        self.check_worker_responses();
        self.try_handle_app_event(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                // Clean up worker thread before exit
                if let Some(mut worker) = self.worker.take() {
                    debug!("Shutting down worker thread");
                    worker.shutdown();
                }
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let (Some(wgpu_ctx), Some(window)) =
                    (self.wgpu_ctx.as_mut(), self.window.as_ref())
                {
                    wgpu_ctx.resize((new_size.width, new_size.height));
                    self.layout_context.resize_viewport(wgpu_ctx);

                    // Explicitly refresh all sliders to ensure proper visual sync after resize
                    self.layout_context.refresh_all_sliders();

                    // Ensure component updates are applied immediately
                    self.layout_context.update_components(wgpu_ctx, 0.0);

                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                // Always check for worker responses during loading phase
                if self.app_state.is_checking_auth
                    || self.app_state.current_view == Some(UiView::Login)
                {
                    self.check_worker_responses();
                    self.try_handle_app_event(event_loop);
                }

                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    self.layout_context
                        .update_components(wgpu_ctx, self.frame_counter.get_frame_time());
                    wgpu_ctx.draw(&mut self.layout_context);
                    wgpu_ctx.text_handler.trim_atlas();
                }

                // request redraw after drawing - do this if we have any animations to continuously draw,
                // if not focused limit to 30fps else allow winit to do vsync
                if let Some(window) = &self.window {
                    if !window.has_focus() {
                        self.frame_counter.limit_fps(BACKGROUND_FPS);
                    }
                    window.request_redraw();
                }
                self.frame_counter.update();
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(window) = &self.window {
                    self.app_state.cursor_position = Some((position.x, position.y));

                    if let Some(resize_state) = &self.app_state.resize_state {
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

                if self.window.is_some() {
                    self.handle_ui_event(event_loop, position.x, position.y, None, None, None);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some((x, y)) = self.app_state.cursor_position {
                    match state {
                        winit::event::ElementState::Pressed => {
                            if let Some(direction) = self.get_resize_direction(x, y) {
                                self.app_state.resize_state = Some(ResizeState { direction });
                                return;
                            }
                        }
                        winit::event::ElementState::Released => {
                            if self.app_state.resize_state.is_some() {
                                self.app_state.resize_state = None;
                                self.update_resize_cursor(x, y);
                                return;
                            }
                        }
                    }

                    self.handle_ui_event(event_loop, x, y, Some(state), Some(button), None);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some((x, y)) = self.app_state.cursor_position {
                    if let MouseScrollDelta::LineDelta(_, scroll_y) = delta {
                        self.handle_ui_event(event_loop, x, y, None, None, Some(scroll_y));
                    }
                }
            }
            WindowEvent::Focused(focused) => {
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    self.last_cursor_input = (None, ComponentPosition { x: 0.0, y: 0.0 });

                    // First reset all drag states
                    self.layout_context.reset_all_drag_states(wgpu_ctx);

                    // If the window regained focus, ensure sliders are properly refreshed
                    if focused {
                        // Explicitly refresh all sliders to ensure visual sync
                        self.layout_context.refresh_all_sliders();

                        // Complete layout update and redraw
                        self.layout_context.update_components(wgpu_ctx, 0.0);
                        wgpu_ctx.draw(&mut self.layout_context);
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

fn load_icon(bytes: &[u8]) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(bytes).unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
