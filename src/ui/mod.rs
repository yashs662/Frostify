use crate::{
    app::AppEvent,
    constants::WINDOW_CONTROL_BUTTON_SIZE,
    ui::{
        color::Color,
        component::{Component, ComponentConfig, ComponentMetaData, ImageConfig},
        components::{
            button::{ButtonBackground, ButtonBuilder},
            container::FlexContainerBuilder,
            image::ScaleMode,
        },
        layout::{Anchor, FlexValue},
    },
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use component::BorderPosition;
use components::{background::BackgroundBuilder, image::ImageBuilder};
use layout::{AlignItems, BorderRadius, Bounds, Edges, FlexDirection, JustifyContent, Position};
use tokio::sync::mpsc::UnboundedSender;

pub mod asset;
pub mod color;
pub mod component;
pub mod components;
pub mod img_utils;
pub mod layout;
pub mod text_renderer;
pub mod z_index_manager;

pub trait Configurable {
    fn configure(
        component: &mut Component,
        config: ComponentConfig,
        wgpu_ctx: &mut WgpuCtx,
    ) -> Vec<ComponentMetaData>;
}

pub trait Renderable {
    fn draw(
        component: &mut Component,
        render_pass: &mut wgpu::RenderPass,
        app_pipelines: &mut AppPipelines,
    );
}

pub trait Positionable {
    fn set_position(component: &mut Component, wgpu_ctx: &mut WgpuCtx, bounds: Bounds);
}

pub fn create_app_ui(
    wgpu_ctx: &mut WgpuCtx,
    event_tx: UnboundedSender<AppEvent>,
    layout_context: &mut layout::LayoutContext,
) {
    // Main container
    let mut main_container = FlexContainerBuilder::new()
        .with_debug_name("Main Container")
        .with_direction(FlexDirection::Column)
        .with_size(FlexValue::Fill, FlexValue::Fill)
        .build();

    // Background
    let background = ImageBuilder::new("album_art.png")
        .with_scale_mode(ScaleMode::Cover)
        .with_debug_name("Background")
        .with_fixed_position(Anchor::Center)
        .build(wgpu_ctx);

    let frosted_glass = BackgroundBuilder::with_frosted_glass(Color::Black, 10.0, 1.0)
        .with_debug_name("Frosted Glass")
        .with_position(Position::Absolute(Anchor::Center))
        .with_z_index(1)
        .build(wgpu_ctx);

    // Create nav bar using the extracted function
    let nav_bar_container = create_nav_bar(wgpu_ctx, event_tx.clone());
    let player_container = create_player_bar(wgpu_ctx, event_tx.clone());
    // let border_demo_container = create_border_demo(wgpu_ctx);

    // Add children to the main container
    main_container.add_child(background);
    main_container.add_child(frosted_glass);
    main_container.add_child(nav_bar_container);
    main_container.add_child(player_container);
    // main_container.add_child(border_demo_container);

    // Add components in the correct order
    layout_context.add_component(main_container);
}

fn create_nav_bar(wgpu_ctx: &mut WgpuCtx, event_tx: UnboundedSender<AppEvent>) -> Component {
    // Nav bar container
    let mut nav_bar_container = FlexContainerBuilder::new()
        .with_debug_name("Nav Bar Container")
        .with_size(FlexValue::Fill, FlexValue::Fixed(64.0))
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::End)
        .with_padding(Edges::all(10.0))
        .build();

    let nav_bar_background = BackgroundBuilder::with_frosted_glass(Color::Black, 100.0, 1.0)
        .with_debug_name("Nav Bar Background")
        .with_border_radius(BorderRadius::all(5.0))
        .with_margin(Edges::all(10.0))
        .with_border(1.0, Color::Black.lighten(0.01))
        .with_fixed_position(Anchor::Center)
        .with_drag_event(AppEvent::DragWindow)
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    // Nav buttons container
    let mut nav_buttons_container = FlexContainerBuilder::new()
        .with_debug_name("Nav Buttons Container")
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::SpaceBetween)
        .with_width(FlexValue::Fixed(128.0))
        .with_margin(Edges::right(10.0))
        .with_parent(nav_bar_container.id)
        .build();

    // Minimize button
    let minimize_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Image(ImageConfig {
            file_name: "minimize.png".to_string(),
            scale_mode: ScaleMode::default(),
        }))
        .with_size(WINDOW_CONTROL_BUTTON_SIZE, WINDOW_CONTROL_BUTTON_SIZE)
        .with_debug_name("Minimize Button")
        .with_click_event(AppEvent::Minimize)
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    // Maximize button
    let maximize_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Image(ImageConfig {
            file_name: "maximize.png".to_string(),
            scale_mode: ScaleMode::default(),
        }))
        .with_size(WINDOW_CONTROL_BUTTON_SIZE, WINDOW_CONTROL_BUTTON_SIZE)
        .with_debug_name("Maximize Button")
        .with_click_event(AppEvent::Maximize)
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    // Close button
    let close_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Image(ImageConfig {
            file_name: "close.png".to_string(),
            scale_mode: ScaleMode::default(),
        }))
        .with_size(WINDOW_CONTROL_BUTTON_SIZE, WINDOW_CONTROL_BUTTON_SIZE)
        .with_debug_name("Close Button")
        .with_click_event(AppEvent::Close)
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    // Add children to the nav buttons container
    nav_buttons_container.add_child(minimize_button);
    nav_buttons_container.add_child(maximize_button);
    nav_buttons_container.add_child(close_button);

    // Add children to the nav bar container
    nav_bar_container.add_child(nav_buttons_container);
    nav_bar_container.add_child(nav_bar_background);

    nav_bar_container
}

fn create_player_bar(wgpu_ctx: &mut WgpuCtx, _event_tx: UnboundedSender<AppEvent>) -> Component {
    // Player container
    let mut player_container = FlexContainerBuilder::new()
        .with_debug_name("Player Container")
        .with_size(FlexValue::Fill, FlexValue::Fraction(0.15))
        .with_position(Position::Absolute(Anchor::Bottom))
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::SpaceBetween)
        .with_padding(Edges::all(10.0))
        .with_margin(Edges::all(10.0))
        .build();

    // Create frosted glass background with an outside border
    let player_container_background =
        BackgroundBuilder::with_frosted_glass(Color::Black, 100.0, 1.0)
            .with_debug_name("Player Container Background")
            .with_border_radius(BorderRadius::all(5.0))
            .with_border_full(1.0, Color::Black.lighten(0.01), BorderPosition::Inside)
            .build(wgpu_ctx);

    player_container.add_child(player_container_background);

    player_container
}

// Create a demonstration section showing different border positions
fn create_border_demo(wgpu_ctx: &mut WgpuCtx) -> Component {
    let mut demo_container = FlexContainerBuilder::new()
        .with_debug_name("Border Demo Container")
        .with_size(FlexValue::Fill, FlexValue::Fixed(200.0))
        .with_direction(FlexDirection::Row)
        .with_justify_content(JustifyContent::SpaceEvenly)
        .with_align_items(AlignItems::Center)
        .with_padding(Edges::all(20.0))
        .build();

    // Inside border example
    let inside_border = BackgroundBuilder::with_color(Color::White)
        .with_debug_name("Inside Border Example")
        .with_size(120.0, 120.0)
        .with_border_full(15.0, Color::Red, BorderPosition::Inside)
        .with_uniform_border_radius(20.0)
        .build(wgpu_ctx);

    // Center border example
    let center_border = BackgroundBuilder::with_color(Color::White)
        .with_debug_name("Center Border Example")
        .with_size(120.0, 120.0)
        .with_border_full(15.0, Color::Green, BorderPosition::Center)
        .with_uniform_border_radius(20.0)
        .build(wgpu_ctx);

    // Outside border example
    let outside_border = BackgroundBuilder::with_color(Color::White)
        .with_debug_name("Outside Border Example")
        .with_size(120.0, 120.0)
        .with_border_full(15.0, Color::Blue, BorderPosition::Outside)
        .with_uniform_border_radius(20.0)
        .build(wgpu_ctx);

    demo_container.add_child(inside_border);
    demo_container.add_child(center_border);
    demo_container.add_child(outside_border);

    demo_container
}
