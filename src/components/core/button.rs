use super::{
    create_background_component, image::ImageComponent, Bounds, Component, ComponentBackground,
    ComponentPosition, ComponentTransform, DrawableComponent, RenderPassExt,
};
use crate::vertex::Vertex;

pub struct Button {
    background: Option<DrawableComponent>,
    image: Option<ImageComponent>,
    on_click: Box<dyn Fn()>,
    children: Vec<Box<dyn Component>>,
    bounds: Bounds,
}

impl Button {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        background: ComponentBackground,
        transform: ComponentTransform,
        parent_bounds: Option<Bounds>,
        on_click: Box<dyn Fn()>,
    ) -> Self {
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
        let bounds = Bounds::new(position, size);

        let (background_component, image_component) = match background {
            ComponentBackground::None => (None, None),
            ComponentBackground::Image(path) => (
                None,
                Some(ImageComponent::new(device, queue, &path, size, position)),
            ),
            ComponentBackground::Color { color } => {
                let drawable = create_background_component(device, bounds, color, color, 0.0);
                (Some(drawable), None)
            }
            ComponentBackground::Gradient {
                start_color,
                end_color,
                angle,
            } => {
                let drawable = create_background_component(
                    device,
                    bounds,
                    start_color,
                    end_color,
                    angle.to_radians(),
                );
                (Some(drawable), None)
            }
        };

        Self {
            background: background_component,
            image: image_component,
            on_click,
            children: Vec::new(),
            bounds,
        }
    }

    fn check_bounds(&self, x: f32, y: f32) -> bool {
        // Convert screen coordinates to the same coordinate space as the button
        let bounds = self.get_bounds();
        x >= bounds.position.x
            && x <= bounds.position.x + bounds.size.width
            && y >= bounds.position.y
            && y <= bounds.position.y + bounds.size.height
    }
}

impl Component for Button {
    fn update(&mut self, queue: &wgpu::Queue) {
        if let Some(img) = &mut self.image {
            img.update(queue);
        }
    }

    fn draw<'a>(&'a self, render_pass: &mut dyn RenderPassExt<'a>) {
        if let Some(bg) = &self.background {
            // Set the color pipeline before drawing background
            render_pass.set_pipeline(render_pass.parent_pipeline());
            render_pass.set_vertex_buffer(0, bg.vertex_buffer.slice(..));
            render_pass.set_index_buffer(bg.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..bg.indices.len() as u32, 0, 0..1);
        }

        if let Some(img) = &self.image {
            // Reset to texture pipeline for image
            render_pass.set_pipeline(render_pass.texture_pipeline());
            img.draw(render_pass);
        }

        // Draw children
        for child in &self.children {
            child.draw(render_pass);
        }
    }

    fn resize(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, width: u32, height: u32) {
        // Convert pixel coordinates to NDC coordinates
        let ndc_x = (self.bounds.position.x / width as f32) * 2.0 - 1.0;
        let ndc_y = 1.0 - (self.bounds.position.y / height as f32) * 2.0;
        let ndc_width = (self.bounds.size.width / width as f32) * 2.0;
        let ndc_height = (self.bounds.size.height / height as f32) * 2.0;

        if let Some(bg) = &mut self.background {
            bg.vertices = vec![
                Vertex::new([ndc_x, ndc_y, 0.0], bg.vertices[0].color, [0.0, 0.0]),
                Vertex::new(
                    [ndc_x + ndc_width, ndc_y, 0.0],
                    bg.vertices[1].color,
                    [1.0, 0.0],
                ),
                Vertex::new(
                    [ndc_x + ndc_width, ndc_y - ndc_height, 0.0],
                    bg.vertices[2].color,
                    [1.0, 1.0],
                ),
                Vertex::new(
                    [ndc_x, ndc_y - ndc_height, 0.0],
                    bg.vertices[3].color,
                    [0.0, 1.0],
                ),
            ];
            queue.write_buffer(&bg.vertex_buffer, 0, bytemuck::cast_slice(&bg.vertices));
        }

        if let Some(img) = &mut self.image {
            img.resize(queue, device, width, height);
        }

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
        self.bounds.position = position;

        if let Some(img) = &mut self.image {
            img.set_position(queue, device, position);
        }

        if let Some(bg) = &mut self.background {
            // Update background vertices with new position
            bg.vertices = vec![
                Vertex::new(
                    [position.x, position.y, 0.0],
                    bg.vertices[0].color,
                    [0.0, 0.0],
                ),
                Vertex::new(
                    [position.x + self.bounds.size.width, position.y, 0.0],
                    bg.vertices[1].color,
                    [1.0, 0.0],
                ),
                Vertex::new(
                    [
                        position.x + self.bounds.size.width,
                        position.y + self.bounds.size.height,
                        0.0,
                    ],
                    bg.vertices[2].color,
                    [1.0, 1.0],
                ),
                Vertex::new(
                    [position.x, position.y + self.bounds.size.height, 0.0],
                    bg.vertices[3].color,
                    [0.0, 1.0],
                ),
            ];

            queue.write_buffer(&bg.vertex_buffer, 0, bytemuck::cast_slice(&bg.vertices));
        }
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
        self.bounds
    }
}
