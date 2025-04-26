// ViewportResource

use std::any::Any;

use crate::ui::layout::ComponentSize;
use uuid::Uuid;

use super::EcsResource;

pub struct ViewportResource {
    pub size: ComponentSize,
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
    pub render_order: Vec<Uuid>,
}

impl EcsResource for RenderOrderResource {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
