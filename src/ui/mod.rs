use crate::{
    app::AppEvent,
    color::Color,
    ui::component::{
        BackgroundColorConfig, Component, ComponentConfig, ComponentType, ImageConfig, LabelConfig,
    },
    ui::layout::{Anchor, FlexValue, Position},
    wgpu_ctx::WgpuCtx,
};
use layout::{AlignItems, Edges, JustifyContent, Layout};
use tokio::sync::mpsc::UnboundedSender;

pub mod component;
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
    let mut background = Component::new(background_id, ComponentType::Background);
    background.set_debug_name("Background");
    background.configure(
        ComponentConfig::BackgroundColor(BackgroundColorConfig {
            color: Color::White,
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
    nav_bar_container.set_drag_handler(AppEvent::DragWindow(0.0, 0.0), event_tx.clone());

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
            image_path: "assets/minimize.png".to_string(),
        }),
        wgpu_ctx,
    );
    minimize_icon.set_click_handler(AppEvent::Minimize, event_tx.clone());
    minimize_icon.set_z_index(2);
    minimize_icon.set_parent(nav_buttons_container_id);

    // Expand button
    let expand_icon_id = uuid::Uuid::new_v4();
    let mut expand_icon = Component::new(expand_icon_id, ComponentType::Image);
    expand_icon.set_debug_name("Expand Icon");
    expand_icon.transform.size.width = FlexValue::Fixed(button_size);
    expand_icon.transform.size.height = FlexValue::Fixed(button_size);
    expand_icon.configure(
        ComponentConfig::Image(ImageConfig {
            image_path: "assets/expand.png".to_string(),
        }),
        wgpu_ctx,
    );
    expand_icon.set_click_handler(AppEvent::Maximize, event_tx.clone());
    expand_icon.set_z_index(2);
    expand_icon.set_parent(nav_buttons_container_id);

    // Close button
    let close_icon_id = uuid::Uuid::new_v4();
    let mut close_icon = Component::new(close_icon_id, ComponentType::Image);
    close_icon.set_debug_name("Close Icon");
    close_icon.transform.size.width = FlexValue::Fixed(button_size);
    close_icon.transform.size.height = FlexValue::Fixed(button_size);
    close_icon.configure(
        ComponentConfig::Image(ImageConfig {
            image_path: "assets/close.png".to_string(),
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

    // Label with fixed size
    let label_id = uuid::Uuid::new_v4();
    let mut label = Component::new(label_id, ComponentType::Label);
    label.set_debug_name("Label");
    label.transform.size.width = FlexValue::Fixed(200.0); // Fixed width
    label.transform.size.height = FlexValue::Fixed(50.0); // Fixed height
    label.configure(
        ComponentConfig::Label(LabelConfig {
            text: "Test Text render".to_string(),
            font_size: 16.0,
            color: Color::Black,
            line_height: 1.0,
        }),
        wgpu_ctx,
    );
    label.set_z_index(1);
    label.set_parent(content_container_id);

    // Content image
    let image_id = uuid::Uuid::new_v4();
    let mut image = Component::new(image_id, ComponentType::Image);
    image.set_debug_name("Content Image");
    image.configure(
        ComponentConfig::Image(ImageConfig {
            image_path: "assets/test.png".to_string(),
        }),
        wgpu_ctx,
    );
    image.set_z_index(1);
    image.set_parent(content_container_id);

    // Add children to the main container
    main_container.add_child(background_id);
    main_container.add_child(nav_bar_container_id);
    main_container.add_child(content_container_id);

    // Add children to the nav bar container
    nav_bar_container.add_child(nav_buttons_container_id);

    // Add children to the nav buttons container
    nav_buttons_container.add_child(minimize_icon_id);
    nav_buttons_container.add_child(expand_icon_id);
    nav_buttons_container.add_child(close_icon_id);

    // Add children to the content container
    content_container.add_child(label_id);
    content_container.add_child(image_id);

    // Add components in the correct order
    layout_context.add_component(main_container);
    layout_context.add_component(background);
    layout_context.add_component(nav_bar_container);
    layout_context.add_component(nav_buttons_container);
    layout_context.add_component(minimize_icon);
    layout_context.add_component(expand_icon);
    layout_context.add_component(close_icon);
    layout_context.add_component(content_container);
    layout_context.add_component(label);
    layout_context.add_component(image);
}
