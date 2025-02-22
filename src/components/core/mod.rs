pub mod background;
pub mod image;
pub mod root;
pub mod text;

use crate::{
    app::AppEvent,
    color::Color,
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use background::BackgroundComponent;
use image::ImageComponent;
use text::TextComponent;
use uuid::Uuid;

pub trait Component {
    fn id(&self) -> Uuid;
    fn update(&mut self, queue: &wgpu::Queue);
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, app_pipelines: &mut AppPipelines);
    fn resize(&mut self, wgpu_ctx: &WgpuCtx, width: u32, height: u32);
    fn set_position(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        position: ComponentPosition,
    );
    fn handle_mouse_click(&mut self, x: f32, y: f32) -> bool; // Returns true if click was handled
    fn send_event(&self, event: AppEvent);
    fn add_child(&mut self, child: Box<dyn Component>);
    fn get_bounds(&self) -> Bounds;
    fn component_type(&self) -> ComponentType;
    fn as_any(&self) -> &dyn std::any::Any;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    Container,
    Other
}

// TODO: Use this in ComponentSize
pub enum FlexValue {
    Fit,
    Fill,
    Fixed(f32),
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

    pub fn to_text_bounds(self) -> glyphon::TextBounds {
        glyphon::TextBounds {
            left: (self.position.x).round() as i32,
            top: (self.position.y).round() as i32,
            right: (self.position.x + self.size.width).round() as i32,
            bottom: (self.position.y + self.size.height).round() as i32,
        }
    }
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
pub struct ComponentTextConfig {
    pub text: String,
    pub anchor: Anchor,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct ComponentTextOnImageConfig {
    pub text: String,
    pub anchor: Anchor,
    pub text_color: Color,
    pub image_path: String,
}

#[derive(Debug, Clone)]
pub struct ComponentTextOnColorConfig {
    pub text: String,
    pub anchor: Anchor,
    pub text_color: Color,
    pub background_color: Color,
}

#[derive(Debug, Clone)]
pub struct ComponentTextOnGradientConfig {
    pub text: String,
    pub anchor: Anchor,
    pub text_color: Color,
    pub start_color: Color,
    pub end_color: Color,
    pub angle: f32,
}

#[derive(Debug, Clone)]
pub enum ComponentBackgroundConfig {
    None,
    Color {
        color: Color,
    },
    Gradient {
        start_color: Color,
        end_color: Color,
        angle: f32, // angle in radians
    },
    Image(String),
    Text(ComponentTextConfig),
    TextOnImage(ComponentTextOnImageConfig),
    TextOnColor(ComponentTextOnColorConfig),
    TextOnGradient(ComponentTextOnGradientConfig),
}

pub enum ComponentRenderable {
    Background(BackgroundComponent),
    Image(ImageComponent),
    Text(TextComponent),
    TextOnBackground(TextComponent, BackgroundComponent),
    TextOnImage(TextComponent, ImageComponent),
}

impl ComponentRenderable {
    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        app_pipelines: &mut AppPipelines,
    ) {
        match self {
            ComponentRenderable::Background(bg) => bg.draw(render_pass, app_pipelines),
            ComponentRenderable::Image(img) => img.draw(render_pass, app_pipelines),
            ComponentRenderable::Text(_) => (),
            ComponentRenderable::TextOnBackground(_, bg) => bg.draw(render_pass, app_pipelines),
            ComponentRenderable::TextOnImage(_, img) => img.draw(render_pass, app_pipelines),
        }
    }
}
