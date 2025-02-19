use super::bounds::Bounds;
use super::{Component, ComponentPosition, ComponentSize};

pub struct RootComponent {
    children: Vec<Box<dyn Component>>,
    size: ComponentSize,
}

impl RootComponent {
    pub fn new(size: ComponentSize) -> Self {
        Self {
            children: Vec::new(),
            size,
        }
    }
}

impl Component for RootComponent {
    fn update(&mut self, queue: &wgpu::Queue) {
        for child in &mut self.children {
            child.update(queue);
        }
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for child in &self.children {
            child.draw(render_pass);
        }
    }

    fn resize(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, width: u32, height: u32) {
        self.size.width = width as f32;
        self.size.height = height as f32;
        for child in &mut self.children {
            child.resize(queue, device, width, height);
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

    fn remove_child(&mut self, index: usize) -> Option<Box<dyn Component>> {
        if index < self.children.len() {
            Some(self.children.remove(index))
        } else {
            None
        }
    }

    fn get_children(&self) -> &Vec<Box<dyn Component>> {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn Component>> {
        &mut self.children
    }

    fn get_bounds(&self) -> Bounds {
        let position = ComponentPosition { x: 0.0, y: 0.0 };
        Bounds::new(position, self.size)
    }
}
