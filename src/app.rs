use crate::{
    auth::oauth::SpotifyAuthResponse,
    constants::{BACKGROUND_FPS, WINDOW_RESIZE_BORDER_WIDTH},
    core::worker::{Worker, WorkerResponse},
    ui::{
        self, UiView,
        ecs::{
            ModalEntity, NamedRef,
            resources::{MouseResource, RequestReLayoutResource},
            systems::{
                AnimationSystem, ComponentActivationSystem, ComponentHoverResetSystem,
                ComponentHoverSystem, MouseInputSystem, MouseScrollSystem,
            },
        },
        layout::{self, ComponentPosition, Size},
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

#[derive(Debug, Clone, Copy)]
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
    OpenModal(NamedRef),
    CloseModal(NamedRef),
}

#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    wgpu_ctx: Option<WgpuCtx<'window>>,
    event_sender: Option<UnboundedSender<AppEvent>>,
    event_receiver: Option<UnboundedReceiver<AppEvent>>,
    layout_context: layout::LayoutContext,
    app_state: AppState,
    app_config: AppConfig,
    worker: Option<Worker>,
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

#[derive(Default)]
pub struct AppConfig {
    pub test_ui_view: Option<UiView>,
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
    pub fn new(test_ui_view: Option<UiView>) -> Self {
        Self {
            app_config: AppConfig { test_ui_view },
            ..Default::default()
        }
    }

    fn update(&mut self) {
        if let Some(wgpu_ctx) = &mut self.wgpu_ctx {
            // check if re-layout is requested
            let request_relayout_resource = self
                .layout_context
                .world
                .resources
                .get_resource_mut::<RequestReLayoutResource>()
                .expect("Expected RequestReLayoutResource to be present");

            if request_relayout_resource.request_relayout {
                // Reset the request flag
                request_relayout_resource.request_relayout = false;
                // Recompute layout
                self.layout_context.compute_layout_and_sync(wgpu_ctx);
            }

            // Run animation system
            self.layout_context
                .world
                .run_system::<AnimationSystem>(AnimationSystem {
                    frame_time: self.frame_counter.get_frame_time(),
                });
        }
    }

    fn try_handle_app_event(&mut self, event_loop: &ActiveEventLoop) -> bool {
        if let Some(receiver) = &mut self.event_receiver {
            if let Ok(event) = receiver.try_recv() {
                log::debug!("Received app event: {:?}", event);
                match event {
                    AppEvent::Close => {
                        event_loop.exit();
                        return true;
                    }
                    AppEvent::Maximize => {
                        if let Some(window) = &self.window {
                            window.set_maximized(!window.is_maximized());
                            self.layout_context
                                .world
                                .run_system(ComponentHoverResetSystem);
                            return true;
                        }
                    }
                    AppEvent::Minimize => {
                        if let Some(window) = &self.window {
                            window.set_minimized(true);
                            self.layout_context
                                .world
                                .run_system(ComponentHoverResetSystem);
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
                    AppEvent::OpenModal(modal_entity) => {
                        if let Some(entity_id) = self
                            .layout_context
                            .world
                            .named_entities
                            .iter()
                            .find_map(|(named_entity, id)| {
                                // check if the named entity is of some AppNamedModals type
                                if *named_entity == modal_entity {
                                    if named_entity.is_modal() {
                                        debug!("Received OpenModal event for: {}", named_entity);
                                        Some(*id)
                                    } else {
                                        panic!(
                                            "Received OpenModal event for non-modal entity: {}",
                                            named_entity
                                        );
                                    }
                                } else {
                                    None
                                }
                            })
                        {
                            // Open the modal through the modal management system
                            self.layout_context.z_index_manager.open_modal(entity_id);

                            // Activate the modal component
                            self.layout_context
                                .world
                                .run_system(ComponentActivationSystem {
                                    entity_id,
                                    activate: true,
                                    affect_children: true,
                                });
                        } else {
                            panic!(
                                "Received OpenModal event for non-existent modal entity: {}. Did you forget to use with_named_ref() on the modal entity?",
                                modal_entity
                            );
                        }
                        return true;
                    }
                    AppEvent::CloseModal(modal_entity) => {
                        if let Some(entity_id) = self
                            .layout_context
                            .world
                            .named_entities
                            .iter()
                            .find_map(|(named_entity, id)| {
                                // check if the named entity is of some AppNamedModals type
                                if *named_entity == modal_entity {
                                    if named_entity.is_modal() {
                                        debug!("Received CloseModal event for: {}", named_entity);
                                        Some(*id)
                                    } else {
                                        panic!(
                                            "Received CloseModal event for non-modal entity: {}",
                                            named_entity
                                        );
                                    }
                                } else {
                                    None
                                }
                            })
                        {
                            // Close the modal through the modal management system
                            self.layout_context.z_index_manager.close_modal(entity_id);

                            // Deactivate the modal component
                            self.layout_context
                                .world
                                .run_system(ComponentActivationSystem {
                                    entity_id,
                                    activate: false,
                                    affect_children: true,
                                });
                        } else {
                            panic!(
                                "Received CloseModal event for non-existent modal entity: {}. Did you forget to use with_named_ref() on the modal entity?",
                                modal_entity
                            );
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
        view: UiView,
    ) {
        if let Some(wgpu_ctx) = wgpu_ctx {
            layout_context.clear();
            match view {
                UiView::Splash => {
                    ui::create_splash_ui(wgpu_ctx, layout_context);
                }
                UiView::Login => {
                    ui::create_login_ui(wgpu_ctx, layout_context);
                }
                UiView::Home => {
                    ui::create_app_ui(wgpu_ctx, layout_context);
                }
                UiView::Test => {
                    ui::create_test_ui(wgpu_ctx, layout_context);
                }
            }
            layout_context.find_root_component();
            layout_context.compute_layout_and_sync(wgpu_ctx);
            app_state.current_view = Some(view);
        }
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
            let viewport_size = Size {
                width: wgpu_ctx.surface_config.width,
                height: wgpu_ctx.surface_config.height,
            };
            // Create event channel
            let (event_tx, event_rx) = unbounded_channel();
            self.event_sender = Some(event_tx.clone());
            self.event_receiver = Some(event_rx);

            // Initialize layout context before creating UI
            self.layout_context
                .initialize(viewport_size, &mut wgpu_ctx, &event_tx);

            self.wgpu_ctx = Some(wgpu_ctx);

            // Initialize the worker thread
            if self.app_config.test_ui_view.is_none() {
                self.worker = Some(Worker::new());
            }

            if let Some(text_ui_view) = self.app_config.test_ui_view {
                App::change_view(
                    &mut self.wgpu_ctx,
                    &mut self.layout_context,
                    &mut self.app_state,
                    text_ui_view,
                );
            } else {
                // Start with splash screen
                App::change_view(
                    &mut self.wgpu_ctx,
                    &mut self.layout_context,
                    &mut self.app_state,
                    UiView::Splash,
                );
            }
            // This is required due to the timing of the winit window creation causing
            // incorrect layout while changing the view
            self.layout_context
                .compute_layout_and_sync(self.wgpu_ctx.as_mut().unwrap());

            // Check for stored tokens as soon as the app starts
            if let Some(worker) = &self.worker {
                info!("Checking for stored tokens...");
                self.app_state.is_checking_auth = true;
                worker.try_load_tokens();
            }

            // Draw the first frame before making the window visible
            self.update();
            if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                wgpu_ctx.draw(&mut self.layout_context.world);
            }

            // Now that we've drawn the first frame, make the window visible
            if let Some(window) = &self.window {
                window.set_visible(true);
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
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if self.app_state.is_checking_auth
                    || self.app_state.current_view == Some(UiView::Login)
                {
                    self.check_worker_responses();
                }
                self.try_handle_app_event(event_loop);

                self.update();
                self.wgpu_ctx
                    .as_mut()
                    .expect("WgpuCtx Should have been initialized before drawing")
                    .draw(&mut self.layout_context.world);

                // request redraw after drawing - do this if we have any animations to continuously draw,
                // if not focused limit to BACKGROUND_FPS else allow winit to do vsync
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

                let mouse_resource = self
                    .layout_context
                    .world
                    .resources
                    .get_resource_mut::<MouseResource>()
                    .expect("Expected MouseResource to exist");
                let curr_pos = ComponentPosition {
                    x: position.x as f32,
                    y: position.y as f32,
                };

                // Only update dragging state if mouse is currently pressed
                if mouse_resource.is_pressed {
                    if let Some(press_pos) = mouse_resource.press_position {
                        // Only set dragging flag if we've moved at least a few pixels
                        let dx = curr_pos.x - press_pos.x;
                        let dy = curr_pos.y - press_pos.y;
                        let distance_squared = dx * dx + dy * dy;
                        if distance_squared > 4.0 {
                            mouse_resource.is_dragging = true;
                        }
                    }
                }

                mouse_resource.position = curr_pos;
                self.layout_context.world.run_system(ComponentHoverSystem);
            }
            WindowEvent::MouseInput { state, .. } => {
                if let Some((x, y)) = self.app_state.cursor_position {
                    let is_pressed = state == ElementState::Pressed;
                    let is_released = state == ElementState::Released;
                    match state {
                        winit::event::ElementState::Pressed => {
                            if let Some(direction) = self.get_resize_direction(x, y) {
                                self.app_state.resize_state = Some(ResizeState { direction });
                            }
                        }
                        winit::event::ElementState::Released => {
                            if self.app_state.resize_state.is_some() {
                                self.app_state.resize_state = None;
                                self.update_resize_cursor(x, y);
                            }
                        }
                    }
                    let mouse_resource = self
                        .layout_context
                        .world
                        .resources
                        .get_resource_mut::<MouseResource>()
                        .expect("Expected MouseResource to exist");
                    mouse_resource.position = ComponentPosition {
                        x: x as f32,
                        y: y as f32,
                    };
                    mouse_resource.is_pressed = is_pressed;
                    mouse_resource.is_released = is_released;
                    if is_pressed {
                        mouse_resource.press_position = Some(ComponentPosition {
                            x: x as f32,
                            y: y as f32,
                        });
                    } else {
                        mouse_resource.press_position = None;
                        mouse_resource.is_dragging = false;
                    }
                    self.layout_context.world.run_system(MouseInputSystem);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some((x, y)) = self.app_state.cursor_position {
                    if let MouseScrollDelta::LineDelta(_, scroll_y) = delta {
                        let mouse_resource = self
                            .layout_context
                            .world
                            .resources
                            .get_resource_mut::<MouseResource>()
                            .expect("Expected MouseResource to exist");
                        mouse_resource.scroll_delta = -scroll_y;
                        mouse_resource.is_scrolling = true;
                        mouse_resource.position = ComponentPosition {
                            x: x as f32,
                            y: y as f32,
                        };
                        self.layout_context.world.run_system(MouseScrollSystem);
                        self.layout_context.world.run_system(ComponentHoverSystem);
                    }
                }
            }
            WindowEvent::Focused(focused) => {
                if focused {
                    self.wgpu_ctx
                        .as_mut()
                        .expect("WgpuCtx Should have been initialized before drawing")
                        .draw(&mut self.layout_context.world);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::CursorLeft { .. } | WindowEvent::CursorEntered { .. } => {
                if let Some(window) = &self.window {
                    window.set_cursor(CursorIcon::Default);
                }

                // reset mouse resource
                let mouse_resource = self
                    .layout_context
                    .world
                    .resources
                    .get_resource_mut::<MouseResource>()
                    .expect("Expected MouseResource to exist");
                *mouse_resource = MouseResource::default();

                // reset hover state
                self.layout_context
                    .world
                    .run_system(ComponentHoverResetSystem);
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
