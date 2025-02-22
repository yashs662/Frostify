use super::{Bounds, Component, ComponentPosition, ComponentSize, ComponentType};
use crate::wgpu_ctx::WgpuCtx;
use uuid::Uuid;

pub struct RootComponent {
    id: Uuid,
    children: Vec<Box<dyn Component>>,
    size: ComponentSize,
}

impl RootComponent {
    pub fn new(size: ComponentSize) -> Self {
        Self {
            id: Uuid::new_v4(),
            children: Vec::new(),
            size,
        }
    }
}

impl Default for RootComponent {
    fn default() -> Self {
        Self::new(ComponentSize {
            width: 0.0,
            height: 0.0,
        })
    }
}

impl Component for RootComponent {
    fn id(&self) -> Uuid {
        self.id
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn component_type(&self) -> crate::components::core::ComponentType {
        ComponentType::Container
    }

    fn send_event(&self, _event: crate::app::AppEvent) {
        // Root component doesn't handle events
    }

    fn update(&mut self, queue: &wgpu::Queue) {
        for child in &mut self.children {
            child.update(queue);
        }
    }

    fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        app_pipelines: &mut crate::wgpu_ctx::AppPipelines,
    ) {
        for child in &self.children {
            child.draw(render_pass, app_pipelines);
        }
    }

    fn resize(&mut self, wgpu_ctx: &WgpuCtx, width: u32, height: u32) {
        self.size.width = width as f32;
        self.size.height = height as f32;
        for child in &mut self.children {
            child.resize(wgpu_ctx, width, height);
        }
    }

    fn set_position(
        &mut self,
        _queue: &wgpu::Queue,
        _device: &wgpu::Device,
        _position: ComponentPosition,
    ) {
        // Root component doesn't move
    }

    fn handle_mouse_click(&mut self, x: f32, y: f32) -> bool {
        // Propagate click to children in reverse order (top-most first)
        for child in self.children.iter_mut().rev() {
            if child.handle_mouse_click(x, y) {
                return true;
            }
        }
        false
    }

    fn add_child(&mut self, child: Box<dyn Component>) {
        self.children.push(child);
    }

    fn get_bounds(&self) -> Bounds {
        let position = ComponentPosition { x: 0.0, y: 0.0 };
        Bounds::new(position, self.size)
    }
}
