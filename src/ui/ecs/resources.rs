// ViewportResource

use super::{EcsResource, EntityId, systems::RenderGroup};
use crate::ui::layout::{ComponentPosition, Size};
use std::any::Any;

pub struct ViewportResource {
    pub size: Size,
}

impl EcsResource for ViewportResource {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// New resource to store the render order from layout context
pub struct RenderOrderResource {
    pub render_order: Vec<EntityId>,
}

impl EcsResource for RenderOrderResource {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Resource for WGPU device and queue access
#[derive(Clone)]
pub struct WgpuQueueResource {
    pub queue: std::sync::Arc<wgpu::Queue>,
}

impl EcsResource for WgpuQueueResource {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Resource to store render groups
#[derive(Clone)]
pub struct RenderGroupsResource {
    pub groups: Vec<RenderGroup>,
}

impl EcsResource for RenderGroupsResource {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct MousePositionResource {
    pub position: ComponentPosition,
}

impl EcsResource for MousePositionResource {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
