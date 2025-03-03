use crate::{
    app::AppEvent,
    color::Color,
    constants::WINDOW_CONTROL_BUTTON_SIZE,
    ui::{
        components::core::component::{
            Component, ComponentConfig, ComponentType, ImageConfig, TextConfig,
        },
        layout::{Anchor, FlexValue, Position},
    },
    wgpu_ctx::WgpuCtx,
};
use components::core::component::BackgroundGradientConfig;
use components::{
    button::{ButtonBackground, ButtonBuilder},
    container::FlexContainerBuilder,
};
use layout::{AlignItems, Edges, FlexDirection, JustifyContent};
use tokio::sync::mpsc::UnboundedSender;

pub mod components;
pub mod layout;

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
    let mut background = Component::new(background_id, ComponentType::BackgroundColor);
    background.set_debug_name("Background");
    background.configure(
        ComponentConfig::BackgroundGradient(BackgroundGradientConfig {
            start_color: Color::Blue,
            end_color: Color::Red,
            angle: 90.0,
        }),
        wgpu_ctx,
    );
    background.set_z_index(0);
    background.set_parent(main_container_id);
    background.transform.position_type = Position::Absolute(Anchor::TopLeft);

    // Nav bar container
    let mut nav_bar_container = FlexContainerBuilder::new()
        .with_debug_name("Nav Bar Container")
        .with_size(FlexValue::Fill, FlexValue::Fixed(44.0))
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::End)
        .with_padding(Edges::all(10.0))
        .with_z_index(1)
        .with_parent(main_container_id)
        .with_drag_handler(AppEvent::DragWindow, event_tx.clone())
        .build();

    // Nav buttons container
    let mut nav_buttons_container = FlexContainerBuilder::new()
        .with_debug_name("Nav Buttons Container")
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_justify_content(JustifyContent::SpaceBetween)
        .with_width(FlexValue::Fixed(92.0))
        .with_parent(nav_bar_container.id)
        .with_z_index(1)
        .build();

    // Minimize button
    let minimize_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Image("minimize.png".to_string()))
        .with_size(WINDOW_CONTROL_BUTTON_SIZE, WINDOW_CONTROL_BUTTON_SIZE)
        .with_debug_name("Minimize Button")
        .with_click_handler(AppEvent::Minimize, event_tx.clone())
        .with_z_index(1)
        .build(wgpu_ctx);

    // Maximize button
    let maximize_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Image("maximize.png".to_string()))
        .with_size(WINDOW_CONTROL_BUTTON_SIZE, WINDOW_CONTROL_BUTTON_SIZE)
        .with_debug_name("Maximize Button")
        .with_click_handler(AppEvent::Maximize, event_tx.clone())
        .with_z_index(1)
        .build(wgpu_ctx);

    // Close button
    let close_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Image("close.png".to_string()))
        .with_size(WINDOW_CONTROL_BUTTON_SIZE, WINDOW_CONTROL_BUTTON_SIZE)
        .with_debug_name("Close Button")
        .with_click_handler(AppEvent::Close, event_tx.clone())
        .with_z_index(1)
        .build(wgpu_ctx);

    // Content container
    let mut content_container = FlexContainerBuilder::new()
        .with_debug_name("Content Container")
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_padding(Edges::horizontal(10.0))
        .with_parent(main_container_id)
        .build();

    // text with fixed size
    let text_id = uuid::Uuid::new_v4();
    let mut text = Component::new(text_id, ComponentType::Text);
    text.set_debug_name("text");
    text.transform.size.width = FlexValue::Fixed(200.0); // Fixed width
    text.transform.size.height = FlexValue::Fixed(50.0); // Fixed height
    text.configure(
        ComponentConfig::Text(TextConfig {
            text: "Test Text render".to_string(),
            font_size: 16.0,
            color: Color::Black,
            line_height: 1.0,
        }),
        wgpu_ctx,
    );
    text.set_z_index(1);
    text.set_parent(content_container.id);

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
    image.set_z_index(1);
    image.set_parent(content_container.id);

    // Example button with gradient background
    let test_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Color(Color::Blue))
        .with_text("Click Me")
        .with_text_color(Color::White)
        .with_size(150.0, 50.0) // Make button bigger
        .with_font_size(20.0) // Make text bigger
        .with_debug_name("Button test")
        .with_border_radius(20.0)
        .with_click_handler(
            AppEvent::PrintMessage("Button clicked!".to_string()),
            event_tx.clone(),
        )
        .with_z_index(2)
        .build(wgpu_ctx);

    // Add children to the nav buttons container
    nav_buttons_container.add_child(minimize_button);
    nav_buttons_container.add_child(maximize_button);
    nav_buttons_container.add_child(close_button);

    // Add children to the nav bar container
    nav_bar_container.add_child(nav_buttons_container);

    // Add children to the content container
    content_container.add_child(text);
    content_container.add_child(image);
    content_container.add_child(test_button);

    // Add children to the main container
    main_container.add_child(background);
    main_container.add_child(nav_bar_container);
    main_container.add_child(content_container);

    // Add components in the correct order
    layout_context.add_component(main_container);
}
