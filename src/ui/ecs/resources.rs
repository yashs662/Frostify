// ViewportResource

use std::any::Any;

use crate::ui::layout::ComponentSize;

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