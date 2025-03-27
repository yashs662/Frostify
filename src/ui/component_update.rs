use crate::{
    ui::{component::Component, layout::Bounds},
    wgpu_ctx::WgpuCtx,
};
use uuid::Uuid;

/// A trait for any component update data that can be applied to components
pub trait ComponentUpdate {
    /// Apply this update to the target component
    fn apply(&self, component: &mut Component, wgpu_ctx: &mut WgpuCtx);

    /// Get the UUID of the component this update targets
    fn target_id(&self) -> Uuid;

    /// Get additional target IDs if this update affects multiple components
    /// Returns an empty vector by default
    fn additional_target_ids(&self) -> Vec<Uuid> {
        Vec::new()
    }
}

/// A trait that components can implement to provide update data
pub trait CanProvideUpdates {
    /// Get update data from this component if any is available
    fn get_update_data(&self) -> Option<Box<dyn ComponentUpdate>>;

    /// Check if this component has updates to provide
    fn has_updates(&self) -> bool;

    /// Reset the update state after updates are processed
    fn reset_update_state(&mut self);
}

/// Represents a simple position or size update for any component
pub struct BoundsUpdate {
    pub target_id: Uuid,
    pub new_bounds: Bounds,
}

impl ComponentUpdate for BoundsUpdate {
    fn apply(&self, component: &mut Component, wgpu_ctx: &mut WgpuCtx) {
        component.computed_bounds = self.new_bounds;

        // Update GPU buffer with new position
        if let Some(buffer) = component.get_render_data_buffer() {
            wgpu_ctx.queue.write_buffer(
                buffer,
                0,
                bytemuck::cast_slice(&[component.get_render_data(self.new_bounds)]),
            );
        }
    }

    fn target_id(&self) -> Uuid {
        self.target_id
    }

    // We don't need to override additional_target_ids for BoundsUpdate
    // as it only affects a single component
}

/// Represents a property update that changes a value without affecting the render data
pub struct PropertyUpdate<T: Clone + 'static> {
    pub target_id: Uuid,
    pub new_value: T,
    pub apply_fn: fn(&mut Component, T),
}

impl<T: Clone + 'static> ComponentUpdate for PropertyUpdate<T> {
    fn apply(&self, component: &mut Component, _wgpu_ctx: &mut WgpuCtx) {
        (self.apply_fn)(component, self.new_value.clone());
    }

    fn target_id(&self) -> Uuid {
        self.target_id
    }
}
