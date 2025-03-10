use crate::{
    app::AppEvent,
    color::Color,
    ui::{
        component::{
            BackgroundColorConfig, BackgroundGradientConfig, Component, ComponentConfig,
            ComponentType, GradientColorStop, ImageConfig, TextConfig,
        },
        layout::{Anchor, Edges, FlexValue, Position},
    },
    wgpu_ctx::WgpuCtx,
};
use log::error;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use super::container::FlexContainerBuilder;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ButtonBackground {
    None,
    Color(Color),
    Gradient {
        color_stops: Vec<GradientColorStop>,
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
    pub margin: Option<Edges>,
    pub debug_name: Option<String>,
    pub border_radius: Option<f32>,
    pub click_event: Option<AppEvent>,
    pub event_sender: Option<UnboundedSender<AppEvent>>,
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
            margin: None,
            debug_name: None,
            border_radius: None,
            click_event: None,
            event_sender: None,
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

    pub fn with_margin(mut self, margin: Edges) -> Self {
        self.config.margin = Some(margin);
        self
    }

    pub fn with_click_event(mut self, event: AppEvent) -> Self {
        self.config.click_event = Some(event);
        self
    }

    pub fn with_event_sender(mut self, event_tx: UnboundedSender<AppEvent>) -> Self {
        self.config.event_sender = Some(event_tx);
        self
    }

    pub fn build(self, wgpu_ctx: &mut WgpuCtx) -> Component {
        create_button(wgpu_ctx, self.config)
    }
}

fn create_button(wgpu_ctx: &mut WgpuCtx, config: ButtonConfig) -> Component {
    let mut container_builder = FlexContainerBuilder::new();
    // Set fixed size if specified
    if let Some(width) = config.width {
        container_builder = container_builder.with_width(FlexValue::Fixed(width));
    }
    if let Some(height) = config.height {
        container_builder = container_builder.with_height(FlexValue::Fixed(height));
    }
    if let Some(name) = config.debug_name.clone() {
        container_builder = container_builder.with_debug_name(name);
    }
    if let Some(event) = config.click_event.clone() {
        container_builder = container_builder.with_click_event(event);
    }
    if let Some(event_sender) = config.event_sender.clone() {
        container_builder = container_builder.with_event_sender(event_sender);
    }
    if let Some(z_index) = config.z_index {
        container_builder = container_builder.with_z_index(z_index);
    }
    if let Some(margin) = config.margin {
        container_builder = container_builder.with_margin(margin);
    }
    let mut container = container_builder.build();
    container.flag_children_extraction();

    // Create background if specified
    match config.background {
        ButtonBackground::None => {}
        ButtonBackground::Color(color) => {
            let bg_id = Uuid::new_v4();
            let mut bg = Component::new(bg_id, ComponentType::BackgroundColor);
            bg.transform.position_type = Position::Fixed(Anchor::Center);
            bg.set_debug_name("Button Background");
            bg.set_z_index(0);
            if let Some(radius) = config.border_radius {
                bg.set_border_radius(radius);
            }
            bg.configure(
                ComponentConfig::BackgroundColor(BackgroundColorConfig { color }),
                wgpu_ctx,
            );
            container.add_child(bg);
        }
        ButtonBackground::Gradient { color_stops, angle } => {
            let bg_id = Uuid::new_v4();
            let mut bg = Component::new(bg_id, ComponentType::BackgroundGradient);
            bg.transform.position_type = Position::Fixed(Anchor::Center);
            bg.set_debug_name("Button Gradient Background");
            bg.set_z_index(0);
            if let Some(radius) = config.border_radius {
                bg.set_border_radius(radius);
            }
            bg.configure(
                ComponentConfig::BackgroundGradient(BackgroundGradientConfig {
                    color_stops,
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

    if (container.get_click_event().is_some() || container.get_drag_event().is_some())
        && container.get_event_sender().is_none()
    {
        let identifier = if let Some(debug_name) = container.debug_name.as_ref() {
            format!("{} ({})", debug_name, container.id)
        } else {
            container.id.to_string()
        };
        error!(
            "Button {} has click/drag event but no event sender, this will cause the event to not be propagated",
            identifier
        );
    }

    container
}
