use crate::{
    app::AppEvent,
    ui::{
        ecs::{EcsResource, EntityId, systems::RenderGroup},
        layout::ComponentPosition,
    },
};
use frostify_derive::EcsResource;
use tokio::sync::mpsc::UnboundedSender;

// New resource to store the render order from layout context
#[derive(EcsResource, Default)]
pub struct RenderOrderResource {
    pub render_order: Vec<EntityId>,
}

// Resource for WGPU device and queue access
#[derive(Clone, EcsResource)]
pub struct WgpuQueueResource {
    pub queue: std::sync::Arc<wgpu::Queue>,
}

// Resource to store render groups
#[derive(Clone, EcsResource, Default)]
pub struct RenderGroupsResource {
    pub groups: Vec<RenderGroup>,
}

#[derive(EcsResource, Default)]
pub struct MouseResource {
    pub position: ComponentPosition,
    pub is_pressed: bool,
    pub is_released: bool,
    pub is_dragging: bool,
    pub is_scrolling: bool,
    pub scroll_delta: f32,
    pub press_position: Option<ComponentPosition>,
}

#[derive(EcsResource)]
pub struct EventSenderResource {
    pub event_sender: UnboundedSender<AppEvent>,
}

#[derive(EcsResource, Default)]
pub struct RequestReLayoutResource {
    pub request_relayout: bool,
}
