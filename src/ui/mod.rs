use crate::{
    app::AppEvent,
    color::Color,
    constants::WINDOW_CONTROL_BUTTON_SIZE,
    ui::{
        component::{
            Component, ComponentConfig, ComponentMetaData, ComponentType, ImageConfig, TextConfig,
        },
        components::{
            button::{ButtonBackground, ButtonBuilder},
            container::FlexContainerBuilder,
        },
        layout::{Anchor, FlexValue, Position},
    },
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use component::{BackgroundGradientConfig, GradientColorStop, GradientType};
use layout::{AlignItems, BorderRadius, Bounds, Edges, FlexDirection, JustifyContent};
use tokio::sync::mpsc::UnboundedSender;

pub mod component;
pub mod components;
pub mod layout;
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
    let main_container_id = uuid::Uuid::new_v4();
    let mut main_container = FlexContainerBuilder::new()
        .with_debug_name("Main Container")
        .with_direction(FlexDirection::Column)
        .with_size(FlexValue::Fill, FlexValue::Fill)
        .build();

    // Background
    let background_id = uuid::Uuid::new_v4();
    let mut background = Component::new(background_id, ComponentType::BackgroundGradient);
    background.set_debug_name("Background");
    background.configure(
        ComponentConfig::BackgroundGradient(BackgroundGradientConfig {
            color_stops: vec![
                GradientColorStop {
                    color: Color::Cyan,
                    position: 0.0,
                },
                GradientColorStop {
                    color: Color::OrangeRed,
                    position: 0.3,
                },
                GradientColorStop {
                    color: Color::Black,
                    position: 0.5,
                },
            ],
            gradient_type: GradientType::Radial,
            angle: 0.0,
            center: Some((1.0, 0.0)),
            radius: Some(1.8),
        }),
        wgpu_ctx,
    );
    background.transform.position_type = Position::Absolute(Anchor::TopLeft);

    // Nav bar container
    let mut nav_bar_container = FlexContainerBuilder::new()
        .with_debug_name("Nav Bar Container")
        .with_size(FlexValue::Fill, FlexValue::Fixed(44.0))
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::End)
        .with_padding(Edges::all(10.0))
        .with_parent(main_container_id)
        .with_drag_event(AppEvent::DragWindow)
        .with_event_sender(event_tx.clone())
        .build();

    // Nav buttons container
    let mut nav_buttons_container = FlexContainerBuilder::new()
        .with_debug_name("Nav Buttons Container")
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::SpaceBetween)
        .with_width(FlexValue::Fixed(92.0))
        .with_parent(nav_bar_container.id)
        .build();

    // Minimize button
    let minimize_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Image("minimize.png".to_string()))
        .with_size(WINDOW_CONTROL_BUTTON_SIZE, WINDOW_CONTROL_BUTTON_SIZE)
        .with_debug_name("Minimize Button")
        .with_click_event(AppEvent::Minimize)
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    // Maximize button
    let maximize_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Image("maximize.png".to_string()))
        .with_size(WINDOW_CONTROL_BUTTON_SIZE, WINDOW_CONTROL_BUTTON_SIZE)
        .with_debug_name("Maximize Button")
        .with_click_event(AppEvent::Maximize)
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    // Close button
    let close_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Image("close.png".to_string()))
        .with_size(WINDOW_CONTROL_BUTTON_SIZE, WINDOW_CONTROL_BUTTON_SIZE)
        .with_debug_name("Close Button")
        .with_click_event(AppEvent::Close)
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    // Content container
    let mut content_container = FlexContainerBuilder::new()
        .with_debug_name("Content Container")
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::Center)
        .with_padding(Edges::horizontal(10.0))
        .with_parent(main_container_id)
        .build();

    // text with fixed size
    let text_id = uuid::Uuid::new_v4();
    let mut text = Component::new(text_id, ComponentType::Text);
    text.set_debug_name("text");
    text.transform.size.width = FlexValue::Fixed(200.0);
    text.transform.size.height = FlexValue::Fixed(50.0);
    text.configure(
        ComponentConfig::Text(TextConfig {
            text: "Test Text render".to_string(),
            font_size: 16.0,
            color: Color::Black,
            line_height: 1.0,
        }),
        wgpu_ctx,
    );

    // Content image
    let image_id = uuid::Uuid::new_v4();
    let mut image = Component::new(image_id, ComponentType::Image);
    image.set_debug_name("Content Image");
    image.configure(
        ComponentConfig::Image(ImageConfig {
            file_name: "test.png".to_string(),
        }),
        wgpu_ctx,
    );

    // Example button with gradient background
    let test_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Color(Color::Blue.darken(0.2)))
        .with_text("Click Me")
        .with_text_color(Color::White)
        .with_size(150.0, 50.0)
        .with_font_size(20.0)
        .with_debug_name("Button test")
        .with_border_radius(BorderRadius::all(5.0))
        .with_margin(Edges::left(10.0))
        .with_click_event(AppEvent::PrintMessage("Button clicked!".to_string()))
        .with_event_sender(event_tx.clone())
        .build(wgpu_ctx);

    // Add children to the nav buttons container
    nav_buttons_container.add_child(minimize_button);
    nav_buttons_container.add_child(maximize_button);
    nav_buttons_container.add_child(close_button);

    // Add children to the nav bar container
    nav_bar_container.add_child(nav_buttons_container);

    // Add children to the content container
    content_container.add_child(text);
    // content_container.add_child(image);
    content_container.add_child(test_button);

    // Add children to the main container
    main_container.add_child(background);
    main_container.add_child(nav_bar_container);
    main_container.add_child(content_container);

    // Add components in the correct order
    layout_context.add_component(main_container);
}
