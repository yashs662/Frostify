use crate::{
    app::AppEvent,
    color::Color,
    ui::component::{
        BackgroundColorConfig, Component, ComponentConfig, ComponentType, ImageConfig, LabelConfig,
    },
    wgpu_ctx::WgpuCtx,
};
use log::trace;
use tokio::sync::mpsc::UnboundedSender;

pub mod component;
pub mod layout;

pub fn create_app_ui(
    wgpu_ctx: &mut WgpuCtx,
    _event_tx: UnboundedSender<AppEvent>,
    layout_context: &mut layout::LayoutContext,
) {
    let container_id = uuid::Uuid::new_v4();
    trace!("Creating container with id: {}", container_id);
    let mut container = Component::new(container_id, ComponentType::Container);

    let text_id = uuid::Uuid::new_v4();
    trace!("Creating text with id: {}", text_id);
    let mut label = Component::new(text_id, ComponentType::Label);
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
    // Add text as a child of the button
    container.add_child(text_id);
    label.set_parent(container_id);

    // create a background color for the button
    let background_id = uuid::Uuid::new_v4();
    trace!("Creating background with id: {}", background_id);
    let mut background = Component::new(background_id, ComponentType::Background);
    background.configure(
        ComponentConfig::BackgroundColor(BackgroundColorConfig { color: Color::Red }),
        wgpu_ctx,
    );
    background.set_z_index(0);

    let image_id = uuid::Uuid::new_v4();
    trace!("Creating image with id: {}", image_id);
    let mut image = Component::new(image_id, ComponentType::Image);
    image.configure(
        ComponentConfig::Image(ImageConfig {
            image_path: "assets/test.png".to_string(),
        }),
        wgpu_ctx,
    );
    image.set_z_index(1);
    container.add_child(image_id);
    image.set_parent(container_id);

    // Add components to layout context
    // layout_context.add_component(button);
    layout_context.add_component(container);
    layout_context.add_component(label);
    layout_context.add_component(background);
    layout_context.add_component(image);

    layout_context.compute_layout();
}
