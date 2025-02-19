use crate::{app::AppWindowEvents, wgpu_ctx};

use super::core::{
    button::Button,
    container::{Container, FlexAlign, FlexDirection},
    Anchor, Component, ComponentOffset, ComponentSize, ComponentTransform,
};

pub fn create_window_controls(
    wgpu_ctx: &wgpu_ctx::WgpuCtx,
    event_tx: tokio::sync::mpsc::Sender<crate::app::AppWindowEvents>,
) -> Container {
    let mut window_ctrl_container = Container::new(
        ComponentTransform {
            size: ComponentSize {
                width: 120.0,
                height: 40.0,
            },
            offset: ComponentOffset { x: 0.0, y: 0.0 },
            anchor: Anchor::TopRight,
        },
        Some(wgpu_ctx.root.get_bounds()),
        FlexDirection::Row,
        FlexAlign::End,
        FlexAlign::SpaceBetween,
    )
    .with_padding(2.0)
    .with_spacing(5.0);

    let tx = event_tx.clone();
    let minimize_btn = Button::new(
        &wgpu_ctx.device,
        &wgpu_ctx.queue,
        "assets/minus.png",
        ComponentTransform {
            size: ComponentSize {
                width: 32.0,
                height: 32.0,
            },
            offset: ComponentOffset { x: 0.0, y: 0.0 },
            anchor: Anchor::Center,
        },
        Some(window_ctrl_container.get_bounds()),
        Box::new(move || {
            let _ = tx.blocking_send(AppWindowEvents::Minimize);
        }),
    );

    let tx = event_tx.clone();
    let maximize_btn = Button::new(
        &wgpu_ctx.device,
        &wgpu_ctx.queue,
        "assets/expand.png",
        ComponentTransform {
            size: ComponentSize {
                width: 32.0,
                height: 32.0,
            },
            offset: ComponentOffset { x: 0.0, y: 0.0 },
            anchor: Anchor::Center,
        },
        Some(window_ctrl_container.get_bounds()),
        Box::new(move || {
            let _ = tx.blocking_send(AppWindowEvents::Maximize);
        }),
    );

    let close_btn = Button::new(
        &wgpu_ctx.device,
        &wgpu_ctx.queue,
        "assets/close.png",
        ComponentTransform {
            size: ComponentSize {
                width: 32.0,
                height: 32.0,
            },
            offset: ComponentOffset { x: 0.0, y: 0.0 },
            anchor: Anchor::Center,
        },
        Some(window_ctrl_container.get_bounds()),
        Box::new(move || {
            let _ = event_tx.blocking_send(AppWindowEvents::Close);
        }),
    );

    window_ctrl_container.add_child(Box::new(minimize_btn));
    window_ctrl_container.add_child(Box::new(maximize_btn));
    window_ctrl_container.add_child(Box::new(close_btn));

    window_ctrl_container
}
