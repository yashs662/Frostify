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
use components::button::{ButtonBackground, ButtonBuilder};
use components::core::component::BackgroundGradientConfig;
use layout::{AlignItems, Edges, JustifyContent, Layout};
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
    let mut main_container = Component::new(main_container_id, ComponentType::Container);
    main_container.set_debug_name("Main Container");
    main_container.transform.position_type = Position::Absolute(Anchor::TopLeft);
    main_container.layout = Layout::flex_column();

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
    let nav_bar_container_id = uuid::Uuid::new_v4();
    let mut nav_bar_container = Component::new(nav_bar_container_id, ComponentType::Container);
    nav_bar_container.set_debug_name("Nav Bar Container");
    nav_bar_container.transform.size.width = FlexValue::Fill;
    nav_bar_container.transform.size.height = FlexValue::Fixed(44.0);
    nav_bar_container.layout = Layout::flex_row();
    nav_bar_container.layout.align_items = AlignItems::Center;
    nav_bar_container.layout.justify_content = JustifyContent::End;
    nav_bar_container.layout.padding = Edges::all(10.0);
    nav_bar_container.set_z_index(1);
    nav_bar_container.set_parent(main_container_id);
    nav_bar_container.set_drag_handler(AppEvent::DragWindow, event_tx.clone());

    let nav_buttons_container_id = uuid::Uuid::new_v4();
    let mut nav_buttons_container =
        Component::new(nav_buttons_container_id, ComponentType::Container);
    nav_buttons_container.set_debug_name("Nav Buttons Container");
    nav_buttons_container.layout = Layout::flex_row();
    nav_buttons_container.layout.align_items = AlignItems::Center;
    nav_buttons_container.layout.justify_content = JustifyContent::SpaceBetween;
    nav_buttons_container.transform.size.width = FlexValue::Fixed(92.0);
    nav_buttons_container.set_parent(nav_bar_container_id);

    // Minimize button
    let minimize_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Image("minimize.png".to_string()))
        .with_size(WINDOW_CONTROL_BUTTON_SIZE, WINDOW_CONTROL_BUTTON_SIZE)
        .with_debug_name("Minimize Button")
        .with_click_handler(AppEvent::Minimize, event_tx.clone())
        .with_z_index(2)
        .with_parent(nav_buttons_container_id)
        .build(wgpu_ctx);

    // Maximize button
    let maximize_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Image("maximize.png".to_string()))
        .with_size(WINDOW_CONTROL_BUTTON_SIZE, WINDOW_CONTROL_BUTTON_SIZE)
        .with_debug_name("Maximize Button")
        .with_click_handler(AppEvent::Maximize, event_tx.clone())
        .with_z_index(2)
        .with_parent(nav_buttons_container_id)
        .build(wgpu_ctx);

    // Close button
    let close_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Image("close.png".to_string()))
        .with_size(WINDOW_CONTROL_BUTTON_SIZE, WINDOW_CONTROL_BUTTON_SIZE)
        .with_debug_name("Close Button")
        .with_click_handler(AppEvent::Close, event_tx.clone())
        .with_z_index(2)
        .with_parent(nav_buttons_container_id)
        .build(wgpu_ctx);

    // Content container
    let content_container_id = uuid::Uuid::new_v4();
    let mut content_container = Component::new(content_container_id, ComponentType::Container);
    content_container.set_debug_name("Content Container");
    content_container.layout = Layout::flex_row();
    content_container.layout.align_items = AlignItems::Center;
    content_container.set_parent(main_container_id);
    content_container.layout.padding = Edges::horizontal(10.0);

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
    text.set_parent(content_container_id);

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
    image.set_parent(content_container_id);

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
        .with_parent(content_container_id)
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
