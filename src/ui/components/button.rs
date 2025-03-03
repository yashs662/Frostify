use crate::{
    app::AppEvent,
    color::Color,
    ui::{
        components::core::component::{
            BackgroundColorConfig, BackgroundGradientConfig, Component, ComponentConfig,
            ComponentType, ImageConfig, TextConfig,
        },
        layout::{Anchor, FlexValue, Position},
    },
    wgpu_ctx::WgpuCtx,
};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ButtonBackground {
    None,
    Color(Color),
    Gradient {
        start: Color,
        end: Color,
        angle: f32,
    },
    Image(String),
}

#[derive(Debug, Clone)]
pub struct ButtonConfig {
    pub background: ButtonBackground,
    pub text: Option<String>,
    pub text_color: Option<Color>,
    pub font_size: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub debug_name: Option<String>,
    pub border_radius: Option<f32>,
    pub click_handler: Option<(AppEvent, UnboundedSender<AppEvent>)>,
    pub z_index: Option<i32>,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            background: ButtonBackground::None,
            text: None,
            text_color: None,
            font_size: Some(16.0),
            width: None,
            height: None,
            debug_name: None,
            border_radius: None,
            click_handler: None,
            z_index: None,
        }
    }
}

pub struct ButtonBuilder {
    config: ButtonConfig,
}

impl ButtonBuilder {
    pub fn new() -> Self {
        Self {
            config: ButtonConfig::default(),
        }
    }

    pub fn with_background(mut self, background: ButtonBackground) -> Self {
        self.config.background = background;
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.config.text = Some(text.into());
        self
    }

    pub fn with_text_color(mut self, color: Color) -> Self {
        self.config.text_color = Some(color);
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.config.font_size = Some(size);
        self
    }

    pub fn with_debug_name(mut self, name: impl Into<String>) -> Self {
        self.config.debug_name = Some(name.into());
        self
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.config.width = Some(width);
        self.config.height = Some(height);
        self
    }

    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.config.border_radius = Some(radius);
        self
    }

    pub fn with_click_handler(
        mut self,
        event: AppEvent,
        event_tx: UnboundedSender<AppEvent>,
    ) -> Self {
        self.config.click_handler = Some((event, event_tx));
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.config.z_index = Some(z_index);
        self
    }

    pub fn build(self, wgpu_ctx: &mut WgpuCtx) -> Component {
        create_button(wgpu_ctx, self.config)
    }
}

fn create_button(wgpu_ctx: &mut WgpuCtx, config: ButtonConfig) -> Component {
    let container_id = Uuid::new_v4();
    let mut container = Component::new(container_id, ComponentType::Container);
    container.flag_children_extraction();
    // Set fixed size if specified
    if let Some(width) = config.width {
        container.transform.size.width = FlexValue::Fixed(width);
    }
    if let Some(height) = config.height {
        container.transform.size.height = FlexValue::Fixed(height);
    }
    if let Some(name) = config.debug_name {
        container.set_debug_name(&name);
    }
    if let Some((event, event_tx)) = config.click_handler {
        container.set_click_handler(event, event_tx);
    }
    if let Some(z_index) = config.z_index {
        container.set_z_index(z_index);
    }

    // Create background if specified
    match config.background {
        ButtonBackground::None => {}
        ButtonBackground::Color(color) => {
            let bg_id = Uuid::new_v4();
            let mut bg = Component::new(bg_id, ComponentType::BackgroundColor);
            bg.transform.position_type = Position::Fixed(Anchor::Center);
            bg.set_debug_name("Button Background");
            bg.set_z_index(0);
            bg.set_parent(container_id);
            if let Some(radius) = config.border_radius {
                bg.set_border_radius(radius);
            }
            bg.configure(
                ComponentConfig::BackgroundColor(BackgroundColorConfig { color }),
                wgpu_ctx,
            );
            container.add_child(bg);
        }
        ButtonBackground::Gradient { start, end, angle } => {
            let bg_id = Uuid::new_v4();
            let mut bg = Component::new(bg_id, ComponentType::BackgroundGradient);
            bg.transform.position_type = Position::Fixed(Anchor::Center);
            bg.set_debug_name("Button Gradient Background");
            bg.set_z_index(0);
            bg.set_parent(container_id);
            if let Some(radius) = config.border_radius {
                bg.set_border_radius(radius);
            }
            bg.configure(
                ComponentConfig::BackgroundGradient(BackgroundGradientConfig {
                    start_color: start,
                    end_color: end,
                    angle,
                }),
                wgpu_ctx,
            );
            container.add_child(bg);
        }
        ButtonBackground::Image(file_name) => {
            let bg_id = Uuid::new_v4();
            let mut bg = Component::new(bg_id, ComponentType::Image);
            bg.transform.position_type = Position::Fixed(Anchor::Center);
            bg.set_debug_name("Button Image Background");
            bg.set_z_index(0);
            bg.set_parent(container_id);
            if let Some(radius) = config.border_radius {
                bg.set_border_radius(radius);
            }
            bg.configure(ComponentConfig::Image(ImageConfig { file_name }), wgpu_ctx);
            container.add_child(bg);
        }
    }

    // Create text if specified
    if let Some(text) = config.text {
        let text_id = Uuid::new_v4();
        let mut text_component = Component::new(text_id, ComponentType::Text);
        text_component.transform.position_type = Position::Fixed(Anchor::Center);
        text_component.set_debug_name("Button Text");
        text_component.set_z_index(1);
        text_component.set_parent(container_id);
        text_component.configure(
            ComponentConfig::Text(TextConfig {
                text,
                font_size: config.font_size.unwrap_or(16.0),
                color: config.text_color.unwrap_or(Color::Black),
                line_height: 1.0,
            }),
            wgpu_ctx,
        );
        container.add_child(text_component);
    }

    container
}
