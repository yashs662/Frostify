use crate::{
    ui::{
        color::Color,
        component::{
            BackgroundColorConfig, BackgroundGradientConfig, Component, ComponentConfig,
            ComponentType, FrostedGlassConfig, GradientColorStop, GradientType, ImageConfig,
            TextConfig,
        },
        layout::{Anchor, Position},
    },
    wgpu_ctx::WgpuCtx,
};
use log::error;
use uuid::Uuid;

use super::{
    component_builder::{CommonBuilderProps, ComponentBuilder},
    container::FlexContainerBuilder,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ButtonBackground {
    None,
    Color(Color),
    Gradient {
        color_stops: Vec<GradientColorStop>,
        gradient_type: GradientType,
        angle: f32,
        center: Option<(f32, f32)>,
        radius: Option<f32>,
    },
    Image(ImageConfig),
    FrostedGlass {
        tint_color: Color,
        blur_radius: f32,
        opacity: f32,
    },
}

#[derive(Debug, Clone)]
pub struct ButtonConfig {
    pub background: ButtonBackground,
    pub text: Option<String>,
    pub text_color: Option<Color>,
    pub font_size: Option<f32>,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            background: ButtonBackground::None,
            text: None,
            text_color: None,
            font_size: Some(16.0),
        }
    }
}

pub struct ButtonBuilder {
    common: CommonBuilderProps,
    config: ButtonConfig,
}

impl ComponentBuilder for ButtonBuilder {
    fn common_props(&mut self) -> &mut CommonBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl ButtonBuilder {
    pub fn new() -> Self {
        Self {
            common: CommonBuilderProps::default(),
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

    pub fn build(self, wgpu_ctx: &mut WgpuCtx) -> Component {
        let mut container_builder = FlexContainerBuilder::new();

        // Transfer common properties to container builder
        if let Some(width) = self.common.width {
            container_builder = container_builder.with_width(width);
        }
        if let Some(height) = self.common.height {
            container_builder = container_builder.with_height(height);
        }
        if let Some(name) = self.common.debug_name {
            container_builder = container_builder.with_debug_name(name);
        }
        if let Some(event) = self.common.click_event {
            container_builder = container_builder.with_click_event(event);
        }
        if let Some(event_sender) = self.common.event_sender {
            container_builder = container_builder.with_event_sender(event_sender);
        }
        if let Some(z_index) = self.common.z_index {
            container_builder = container_builder.with_z_index(z_index);
        }
        if let Some(margin) = self.common.margin {
            container_builder = container_builder.with_margin(margin);
        }
        if self.common.fit_to_size {
            container_builder = container_builder.set_fit_to_size();
        }
        if let Some(position) = self.common.position {
            container_builder = container_builder.with_position(position);
        }
        if let Some(padding) = self.common.padding {
            container_builder = container_builder.with_padding(padding);
        }

        let mut container = container_builder.build();
        container.flag_children_extraction();

        // Create background if specified
        match self.config.background {
            ButtonBackground::None => {}
            ButtonBackground::Color(color) => {
                let bg_id = Uuid::new_v4();
                let mut bg = Component::new(bg_id, ComponentType::BackgroundColor);
                bg.transform.position_type = Position::Fixed(Anchor::Center);
                bg.set_debug_name("Button Color Background");
                bg.set_z_index(0);
                if let Some(border_radius) = self.common.border_radius {
                    bg.set_border_radius(border_radius);
                }
                if let Some(border_width) = self.common.border_width {
                    bg.border_width = border_width;
                }
                if let Some(border_color) = self.common.border_color {
                    bg.border_color = border_color;
                }
                if let Some(border_position) = self.common.border_position {
                    bg.set_border_position(border_position);
                }
                bg.configure(
                    ComponentConfig::BackgroundColor(BackgroundColorConfig { color }),
                    wgpu_ctx,
                );
                container.add_child(bg);
            }
            ButtonBackground::Gradient {
                color_stops,
                gradient_type,
                angle,
                center,
                radius,
            } => {
                let bg_id = Uuid::new_v4();
                let mut bg = Component::new(bg_id, ComponentType::BackgroundGradient);
                bg.transform.position_type = Position::Fixed(Anchor::Center);
                bg.set_debug_name("Button Gradient Background");
                bg.set_z_index(0);
                if let Some(border_radius) = self.common.border_radius {
                    bg.set_border_radius(border_radius);
                }
                if let Some(border_width) = self.common.border_width {
                    bg.border_width = border_width;
                }
                if let Some(border_color) = self.common.border_color {
                    bg.border_color = border_color;
                }
                if let Some(border_position) = self.common.border_position {
                    bg.set_border_position(border_position);
                }
                bg.configure(
                    ComponentConfig::BackgroundGradient(BackgroundGradientConfig {
                        color_stops,
                        gradient_type,
                        angle,
                        center,
                        radius,
                    }),
                    wgpu_ctx,
                );
                container.add_child(bg);
            }
            ButtonBackground::Image(img_config) => {
                let bg_id = Uuid::new_v4();
                let mut bg = Component::new(bg_id, ComponentType::Image);
                bg.transform.position_type = Position::Fixed(Anchor::Center);
                bg.set_debug_name("Button Image Background");
                bg.set_z_index(0);
                if let Some(border_radius) = self.common.border_radius {
                    bg.set_border_radius(border_radius);
                }
                if let Some(border_width) = self.common.border_width {
                    bg.border_width = border_width;
                }
                if let Some(border_color) = self.common.border_color {
                    bg.border_color = border_color;
                }
                if let Some(border_position) = self.common.border_position {
                    bg.set_border_position(border_position);
                }
                bg.configure(ComponentConfig::Image(img_config), wgpu_ctx);
                container.add_child(bg);
            }
            ButtonBackground::FrostedGlass {
                tint_color,
                blur_radius,
                opacity,
            } => {
                let bg_id = Uuid::new_v4();
                let mut bg = Component::new(bg_id, ComponentType::FrostedGlass);
                bg.transform.position_type = Position::Fixed(Anchor::Center);
                bg.set_debug_name("Button Frosted Glass Background");
                bg.set_z_index(0);
                if let Some(border_radius) = self.common.border_radius {
                    bg.set_border_radius(border_radius);
                }
                if let Some(border_width) = self.common.border_width {
                    bg.border_width = border_width;
                }
                if let Some(border_color) = self.common.border_color {
                    bg.border_color = border_color;
                }
                if let Some(border_position) = self.common.border_position {
                    bg.set_border_position(border_position);
                }
                bg.configure(
                    ComponentConfig::FrostedGlass(FrostedGlassConfig {
                        tint_color,
                        blur_radius,
                        opacity,
                    }),
                    wgpu_ctx,
                );
                container.add_child(bg);
            }
        }

        // Create text if specified
        if let Some(text) = self.config.text {
            let text_id = Uuid::new_v4();
            let mut text_component = Component::new(text_id, ComponentType::Text);
            text_component.transform.position_type = Position::Fixed(Anchor::Center);
            text_component.set_debug_name("Button Text");
            text_component.set_z_index(1);
            text_component.configure(
                ComponentConfig::Text(TextConfig {
                    text,
                    font_size: self.config.font_size.unwrap_or(16.0),
                    color: self.config.text_color.unwrap_or(Color::Black),
                    line_height: 1.0,
                }),
                wgpu_ctx,
            );
            if container.fit_to_size {
                text_component.set_fit_to_size(true);
            }
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
}
