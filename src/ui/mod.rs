use crate::{
    app::AppEvent,
    color::Color,
    ui::components::core::component::{
        Component, ComponentConfig, ComponentType, ImageConfig, TextConfig,
    },
    ui::layout::{Anchor, FlexValue, Position},
    wgpu_ctx::WgpuCtx,
};
use components::button::{ButtonBackground, ButtonBuilder};
use components::core::component::{BackgroundGradientConfig, ComponentMetaData};
use layout::{AlignItems, Edges, JustifyContent, Layout};
use tokio::sync::mpsc::UnboundedSender; // Add this import

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
    // Add drag event handling to nav bar
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

    // Nav bar buttons with fixed size and spacing
    let button_size = 24.0; // Fixed size for all buttons

    // Minimize button
    let minimize_icon_id = uuid::Uuid::new_v4();
    let mut minimize_icon = Component::new(minimize_icon_id, ComponentType::Image);
    minimize_icon.set_debug_name("Minimize Icon");
    minimize_icon.transform.size.width = FlexValue::Fixed(button_size);
    minimize_icon.transform.size.height = FlexValue::Fixed(button_size);
    minimize_icon.configure(
        ComponentConfig::Image(ImageConfig {
            file_name: "minimize.png".to_string(),
        }),
        wgpu_ctx,
    );
    minimize_icon.set_click_handler(AppEvent::Minimize, event_tx.clone());
    minimize_icon.set_z_index(2);
    minimize_icon.set_parent(nav_buttons_container_id);

    // Maximize button
    let maximize_icon_id = uuid::Uuid::new_v4();
    let mut maximize_icon = Component::new(maximize_icon_id, ComponentType::Image);
    maximize_icon.set_debug_name("Maximize Icon");
    maximize_icon.transform.size.width = FlexValue::Fixed(button_size);
    maximize_icon.transform.size.height = FlexValue::Fixed(button_size);
    maximize_icon.configure(
        ComponentConfig::Image(ImageConfig {
            file_name: "maximize.png".to_string(),
        }),
        wgpu_ctx,
    );
    maximize_icon.set_click_handler(AppEvent::Maximize, event_tx.clone());
    maximize_icon.set_z_index(2);
    maximize_icon.set_parent(nav_buttons_container_id);

    // Close button
    let close_icon_id = uuid::Uuid::new_v4();
    let mut close_icon = Component::new(close_icon_id, ComponentType::Image);
    close_icon.set_debug_name("Close Icon");
    close_icon.transform.size.width = FlexValue::Fixed(button_size);
    close_icon.transform.size.height = FlexValue::Fixed(button_size);
    close_icon.configure(
        ComponentConfig::Image(ImageConfig {
            file_name: "close.png".to_string(),
        }),
        wgpu_ctx,
    );
    close_icon.set_click_handler(AppEvent::Close, event_tx.clone());
    close_icon.set_z_index(2);
    close_icon.set_parent(nav_buttons_container_id);

    // Content container
    let content_container_id = uuid::Uuid::new_v4();
    let mut content_container = Component::new(content_container_id, ComponentType::Container);
    content_container.set_debug_name("Content Container");
    content_container.layout = Layout::flex_row();
    content_container.layout.align_items = AlignItems::Center;
    content_container.set_parent(main_container_id);

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
    let mut test_button = ButtonBuilder::new()
        .with_background(ButtonBackground::Color(Color::Blue))
        .with_text("Click Me")
        .with_text_color(Color::White)
        .with_size(150.0, 50.0) // Make button bigger
        .with_font_size(20.0) // Make text bigger
        .with_debug_name("Button test")
        .with_border_radius(1000.1)
        .build(wgpu_ctx);
    let test_button_id = test_button.id;
    test_button.set_z_index(2); // Ensure button is visible above other content
    test_button.set_click_handler(
        AppEvent::PrintMessage("Button clicked!".to_string()),
        event_tx.clone(),
    );
    test_button.set_parent(content_container_id);

    // Add children to the main container
    main_container.add_child(background_id);
    main_container.add_child(nav_bar_container_id);
    main_container.add_child(content_container_id);

    // Add children to the nav bar container
    nav_bar_container.add_child(nav_buttons_container_id);

    // Add children to the nav buttons container
    nav_buttons_container.add_child(minimize_icon_id);
    nav_buttons_container.add_child(maximize_icon_id);
    nav_buttons_container.add_child(close_icon_id);

    // Add children to the content container
    content_container.add_child(text_id);
    content_container.add_child(image_id);
    content_container.add_child(test_button_id);

    // Add components in the correct order
    layout_context.add_component(main_container);
    layout_context.add_component(background);
    layout_context.add_component(nav_bar_container);
    layout_context.add_component(nav_buttons_container);
    layout_context.add_component(minimize_icon);
    layout_context.add_component(maximize_icon);
    layout_context.add_component(close_icon);
    layout_context.add_component(content_container);
    layout_context.add_component(text);
    layout_context.add_component(image);

    // Extract and add child components before adding the button
    let child_components = if let Some(ComponentMetaData::ChildComponents(children)) = test_button
        .metadata
        .iter()
        .find(|m| matches!(m, ComponentMetaData::ChildComponents(_)))
    {
        children.clone()
    } else {
        Vec::new()
    };

    // Add the button and its children to the layout context
    layout_context.add_component(test_button);
    for child in child_components {
        layout_context.add_component(child);
    }
}
