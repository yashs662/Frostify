use super::{EcsResource, EntityId, systems::RenderGroup};
use crate::{
    app::AppEvent,
    ui::layout::{ComponentPosition, Size},
};
use frostify_derive::EcsResource;
use tokio::sync::mpsc::UnboundedSender;

#[derive(EcsResource)]
pub struct ViewportResource {
    pub size: Size,
}

// New resource to store the render order from layout context
#[derive(EcsResource)]
pub struct RenderOrderResource {
    pub render_order: Vec<EntityId>,
}

// Resource for WGPU device and queue access
#[derive(Clone, EcsResource)]
pub struct WgpuQueueResource {
    pub queue: std::sync::Arc<wgpu::Queue>,
}

// Resource to store render groups
#[derive(Clone, EcsResource)]
pub struct RenderGroupsResource {
    pub groups: Vec<RenderGroup>,
}

#[derive(EcsResource, Default)]
pub struct MouseResource {
    pub position: ComponentPosition,
    pub is_pressed: bool,
    pub is_released: bool,
    pub is_dragging: bool,
    pub press_position: Option<ComponentPosition>,
}

#[derive(EcsResource)]
pub struct EventSenderResource {
    pub event_sender: UnboundedSender<AppEvent>,
}
