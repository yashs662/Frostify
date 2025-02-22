use crate::{
    app::AppEvent,
    color::Color,
    components::{
        button::Button,
        container::{Container, FlexAlign, FlexDirection},
        core::{
            Anchor, Bounds, Component, ComponentBackgroundConfig, ComponentOffset, ComponentSize, ComponentTextConfig, ComponentTextOnColorConfig, ComponentTransform
        },
        label::Label,
    },
    wgpu_ctx::WgpuCtx,
};
use log::info;
use tokio::sync::mpsc::UnboundedSender;

pub fn create_navbar(
    wgpu_ctx: &mut WgpuCtx,
    event_tx: UnboundedSender<AppEvent>,
    bounds: Bounds,
) -> Container {
    let mut nav_container = Container::new(
        wgpu_ctx,
        ComponentTransform {
            size: ComponentSize {
                width: bounds.size.width,
                height: 60.0,
            },
            offset: ComponentOffset { x: 0.0, y: 0.0 },
            anchor: Anchor::TopLeft,
        },
        Some(bounds),
        FlexDirection::Row,
        FlexAlign::SpaceBetween,
        FlexAlign::Center,
        None,
        ComponentBackgroundConfig::Color { color: Color::Bisque },
    )
    .with_padding(10.0); // Add some padding

    info!("Nav container bounds: {:#?}", nav_container.get_bounds());

    // App name label - horizontal_alignment left
    let app_name_label = Label::new(
        wgpu_ctx,
        ComponentBackgroundConfig::TextOnColor(ComponentTextOnColorConfig {
            text: "Frostify".to_string(),
            text_color: Color::Black,
            background_color: Color::OrangeRed,
            anchor: Anchor::Center,
        }),
        ComponentTransform {
            size: ComponentSize {
                width: 100.0,
                height: 40.0,
            },
            offset: ComponentOffset { x: 0.0, y: 0.0 },
            anchor: Anchor::TopLeft,
        },
        Some(nav_container.get_bounds_with_padding()),
    );

    // Window controls container - horizontal_alignment right
    let mut window_ctrl_container = Container::new(
        wgpu_ctx,
        ComponentTransform {
            size: ComponentSize {
                width: 120.0,
                height: 40.0,
            },
            offset: ComponentOffset { x: 0.0, y: 0.0 },
            anchor: Anchor::Right,
        },
        Some(nav_container.get_bounds_with_padding()),
        FlexDirection::Row,
        FlexAlign::End,
        FlexAlign::Center,
        None,
        ComponentBackgroundConfig::Color {
            color: Color::Green,
        },
    )
    .with_padding(2.0)
    .with_gap(5.0);

    let tx = event_tx.clone();
    let minimize_btn = Button::new(
        wgpu_ctx,
        ComponentBackgroundConfig::Image("./assets/minimize.png".to_string()),
        ComponentTransform {
            size: ComponentSize {
                width: 32.0,
                height: 32.0,
            },
            offset: ComponentOffset { x: 0.0, y: 0.0 },
            anchor: Anchor::Center,
        },
        Some(window_ctrl_container.get_bounds_with_padding()),
        AppEvent::Minimize,
        Some(tx),
    );

    let tx = event_tx.clone();
    let maximize_btn = Button::new(
        wgpu_ctx,
        ComponentBackgroundConfig::Image("./assets/expand.png".to_string()),
        ComponentTransform {
            size: ComponentSize {
                width: 32.0,
                height: 32.0,
            },
            offset: ComponentOffset { x: 0.0, y: 0.0 },
            anchor: Anchor::Center,
        },
        Some(window_ctrl_container.get_bounds_with_padding()),
        AppEvent::Maximize,
        Some(tx),
    );

    let close_btn = Button::new(
        wgpu_ctx,
        ComponentBackgroundConfig::Image("assets/close.png".to_string()),
        ComponentTransform {
            size: ComponentSize {
                width: 32.0,
                height: 32.0,
            },
            offset: ComponentOffset { x: 0.0, y: 0.0 },
            anchor: Anchor::Center,
        },
        Some(window_ctrl_container.get_bounds_with_padding()),
        AppEvent::Close,
        Some(event_tx),
    );

    window_ctrl_container.add_child(Box::new(minimize_btn));
    window_ctrl_container.add_child(Box::new(maximize_btn));
    window_ctrl_container.add_child(Box::new(close_btn));

    nav_container.add_child(Box::new(app_name_label));
    nav_container.add_child(Box::new(window_ctrl_container));

    nav_container
}
