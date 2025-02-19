pub mod button;
pub mod container;
pub mod image;
pub mod root;

use crate::vertex::Vertex;
use wgpu::{util::DeviceExt, Buffer};

pub trait Component {
    fn update(&mut self, queue: &wgpu::Queue);
    fn draw<'a>(&'a self, render_pass: &mut dyn RenderPassExt<'a>);
    fn resize(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, width: u32, height: u32);
    fn set_position(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        position: ComponentPosition,
    );
    fn handle_mouse_click(&mut self, x: f32, y: f32) -> bool; // Returns true if click was handled

    // New methods for managing child components
    fn add_child(&mut self, child: Box<dyn Component>);
    fn remove_child(&mut self, index: usize) -> Option<Box<dyn Component>>;
    fn get_children(&self) -> &Vec<Box<dyn Component>>;
    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn Component>>;

    fn get_bounds(&self) -> Bounds;
    fn get_anchor_position(&self, anchor: Anchor) -> (f32, f32) {
        self.get_bounds().get_anchor_position(anchor)
    }
}

pub trait RenderPassExt<'a> {
    fn parent_pipeline(&self) -> &'a wgpu::RenderPipeline;
    fn texture_pipeline(&self) -> &'a wgpu::RenderPipeline;
    fn set_pipeline(&mut self, pipeline: &'a wgpu::RenderPipeline);
    fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &'a wgpu::BindGroup,
        offsets: &[wgpu::DynamicOffset],
    );
    fn set_vertex_buffer(&mut self, slot: u32, buffer: wgpu::BufferSlice<'a>);
    fn set_index_buffer(&mut self, buffer: wgpu::BufferSlice<'a>, index_format: wgpu::IndexFormat);
    fn draw_indexed(
        &mut self,
        indices: std::ops::Range<u32>,
        base_vertex: i32,
        instances: std::ops::Range<u32>,
    );
}

pub struct DrawableComponent {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub bind_group: wgpu::BindGroup,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentOffset {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentTransform {
    pub size: ComponentSize,
    pub offset: ComponentOffset,
    pub anchor: Anchor,
}

#[derive(Debug, Copy, Clone)]
pub struct Bounds {
    pub position: ComponentPosition,
    pub size: ComponentSize,
}

#[derive(Debug, Copy, Clone)]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Debug, Clone)]
pub enum ComponentBackground {
    None,
    Color {
        color: [f32; 4],
    },
    Gradient {
        start_color: [f32; 4],
        end_color: [f32; 4],
        angle: f32, // angle in radians
    },
    Image(String),
}

impl Bounds {
    pub fn new(position: ComponentPosition, size: ComponentSize) -> Self {
        Self { position, size }
    }

    pub fn get_anchor_position(&self, anchor: Anchor) -> (f32, f32) {
        match anchor {
            Anchor::TopLeft => (self.position.x, self.position.y),
            Anchor::Top => (self.position.x + self.size.width / 2.0, self.position.y),
            Anchor::TopRight => (self.position.x + self.size.width, self.position.y),
            Anchor::Left => (self.position.x, self.position.y + self.size.height / 2.0),
            Anchor::Center => (
                self.position.x + self.size.width / 2.0,
                self.position.y + self.size.height / 2.0,
            ),
            Anchor::Right => (
                self.position.x + self.size.width,
                self.position.y + self.size.height / 2.0,
            ),
            Anchor::BottomLeft => (self.position.x, self.position.y + self.size.height),
            Anchor::Bottom => (
                self.position.x + self.size.width / 2.0,
                self.position.y + self.size.height,
            ),
            Anchor::BottomRight => (
                self.position.x + self.size.width,
                self.position.y + self.size.height,
            ),
        }
    }
}

pub fn create_background_component(
    device: &wgpu::Device,
    bounds: Bounds,
    start_color: [f32; 4],
    end_color: [f32; 4],
    angle: f32,
) -> DrawableComponent {
    // Calculate gradient direction vector based on angle
    let (sin, cos) = angle.sin_cos();
    let direction = [cos, sin];

    // Calculate colors for each corner
    let corners = [
        [0.0, 0.0], // Top-left
        [1.0, 0.0], // Top-right
        [1.0, 1.0], // Bottom-right
        [0.0, 1.0], // Bottom-left
    ];

    let colors: Vec<[f32; 4]> = corners
        .iter()
        .map(|corner| {
            let projection = corner[0] * direction[0] + corner[1] * direction[1];
            let t = (projection + 1.0) / 2.0;
            [
                start_color[0] * (1.0 - t) + end_color[0] * t,
                start_color[1] * (1.0 - t) + end_color[1] * t,
                start_color[2] * (1.0 - t) + end_color[2] * t,
                start_color[3] * (1.0 - t) + end_color[3] * t,
            ]
        })
        .collect();

    // Initial vertices in non-NDC space (will be converted during resize)
    let vertices = vec![
        Vertex::new(
            [bounds.position.x, bounds.position.y, 0.0],
            colors[0],
            [0.0, 0.0],
        ),
        Vertex::new(
            [
                bounds.position.x + bounds.size.width,
                bounds.position.y,
                0.0,
            ],
            colors[1],
            [1.0, 0.0],
        ),
        Vertex::new(
            [
                bounds.position.x + bounds.size.width,
                bounds.position.y + bounds.size.height,
                0.0,
            ],
            colors[2],
            [1.0, 1.0],
        ),
        Vertex::new(
            [
                bounds.position.x,
                bounds.position.y + bounds.size.height,
                0.0,
            ],
            colors[3],
            [0.0, 1.0],
        ),
    ];

    let indices = vec![0, 1, 2, 0, 2, 3];

    // Create buffers
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Background Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Background Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    // Create an empty bind group for solid colors and gradients
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[],
        label: Some("Background Bind Group Layout"),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[],
        label: Some("Background Bind Group"),
    });

    DrawableComponent {
        vertex_buffer,
        index_buffer,
        bind_group,
        vertices,
        indices,
    }
}
