pub mod button;
pub mod container;
pub mod image;
pub mod root;

use crate::vertex::Vertex;
use wgpu::Buffer;

pub trait Component {
    fn update(&mut self, queue: &wgpu::Queue);
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
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
