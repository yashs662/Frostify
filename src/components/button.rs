use super::bounds::{Anchor, Bounds};
use super::{image::ImageComponent, Component};
use super::{ComponentOffset, ComponentPosition, ComponentTransform};

pub struct Button {
    image: ImageComponent,
    on_click: Box<dyn Fn()>,
    children: Vec<Box<dyn Component>>,
    anchor: Anchor,
    offset: ComponentOffset,
    parent_bounds: Option<Bounds>,
}

impl Button {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_path: &str,
        transform: ComponentTransform,
        parent_bounds: Option<Bounds>,
        on_click: Box<dyn Fn()>,
    ) -> Self {
        // Calculate initial position based on anchor and parent bounds
        let ComponentTransform {
            size,
            offset,
            anchor,
        } = transform;
        let (x, y) = if let Some(bounds) = parent_bounds {
            let (anchor_x, anchor_y) = bounds.get_anchor_position(anchor);
            (anchor_x + offset.x, anchor_y + offset.y)
        } else {
            (offset.x, offset.y)
        };

        let position = ComponentPosition { x, y };

        Self {
            image: ImageComponent::new(device, queue, texture_path, size, position),
            on_click,
            children: Vec::new(),
            anchor,
            offset,
            parent_bounds,
        }
    }

    fn check_bounds(&self, x: f32, y: f32) -> bool {
        let btn_pos = self.image.get_position();
        let btn_size = self.image.get_size();

        x >= btn_pos.x - btn_size.width / 2.0
            && x <= btn_pos.x + btn_size.width / 2.0
            && y >= btn_pos.y - btn_size.height / 2.0
            && y <= btn_pos.y + btn_size.height / 2.0
    }
}

impl Component for Button {
    fn update(&mut self, queue: &wgpu::Queue) {
        self.image.update(queue)
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.image.draw(render_pass);

        // Draw all children
        for child in &self.children {
            child.draw(render_pass);
        }
    }

    fn resize(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, width: u32, height: u32) {
        // Update position based on anchor and parent bounds
        if let Some(mut bounds) = self.parent_bounds {
            bounds.width = width as f32;
            bounds.height = height as f32;
            let (anchor_x, anchor_y) = bounds.get_anchor_position(self.anchor);
            let position = ComponentPosition {
                x: anchor_x + self.offset.x,
                y: anchor_y + self.offset.y,
            };
            self.image.set_position(queue, device, position);
        }

        self.image.resize(queue, device, width, height);

        // Resize children
        for child in &mut self.children {
            child.resize(queue, device, width, height);
        }
    }

    fn set_position(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        position: ComponentPosition,
    ) {
        self.image.set_position(queue, device, position);
    }

    fn handle_mouse_click(&mut self, x: f32, y: f32) -> bool {
        // First check if any children handle the click
        for child in &mut self.children {
            if child.handle_mouse_click(x, y) {
                return true;
            }
        }

        if self.check_bounds(x, y) {
            (self.on_click)();
            true
        } else {
            false
        }
    }

    // Implement new Component trait methods
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
        self.image.get_bounds()
    }
}
