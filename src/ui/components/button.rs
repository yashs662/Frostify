use crate::{
    ui::{
        color::Color,
        component::{
            BackgroundColorConfig, BackgroundGradientConfig, Component, ComponentConfig,
            ComponentHoverEffects, ComponentType, FrostedGlassConfig, GradientColorStop,
            GradientType, ImageConfig, TextConfig,
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
    pub backgrounds: Vec<(ButtonBackground, Option<ComponentHoverEffects>)>,
    pub text: Option<String>,
    pub text_color: Option<Color>,
    pub font_size: Option<f32>,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            backgrounds: Vec::new(),
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
        self.config.backgrounds.push((background, None));
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

    pub fn with_background_with_hover_effects(
        mut self,
        background: ButtonBackground,
        hover_effects: ComponentHoverEffects,
    ) -> Self {
        self.config
            .backgrounds
            .push((background, Some(hover_effects)));
        self
    }

    pub fn build(self, wgpu_ctx: &mut WgpuCtx) -> Component {
        let mut container_builder = FlexContainerBuilder::new();
        let common_props = self.common.clone();
        let config = self.config.clone();

        // Transfer common properties to container builder
        if let Some(width) = common_props.width {
            container_builder = container_builder.with_width(width);
        }
        if let Some(height) = common_props.height {
            container_builder = container_builder.with_height(height);
        }
        if let Some(name) = common_props.debug_name {
            container_builder = container_builder.with_debug_name(name);
        }
        if let Some(event) = common_props.click_event {
            container_builder = container_builder.with_click_event(event);
        }
        if let Some(event_sender) = common_props.event_sender {
            container_builder = container_builder.with_event_sender(event_sender);
        }
        if let Some(z_index) = common_props.z_index {
            container_builder = container_builder.with_z_index(z_index);
        }
        if let Some(margin) = common_props.margin {
            container_builder = container_builder.with_margin(margin);
        }
        if common_props.fit_to_size {
            container_builder = container_builder.set_fit_to_size();
        }
        if let Some(position) = common_props.position {
            container_builder = container_builder.with_position(position);
        }
        if let Some(padding) = common_props.padding {
            container_builder = container_builder.with_padding(padding);
        }

        let mut container = container_builder.build();
        container.flag_children_extraction();

        // Create background if specified
        for (background, hover_effects) in config.backgrounds {
            match background {
                ButtonBackground::None => {}
                ButtonBackground::Color(color) => {
                    self.configure_color_background(wgpu_ctx, &mut container, color, hover_effects);
                }
                ButtonBackground::Gradient {
                    color_stops,
                    gradient_type,
                    angle,
                    center,
                    radius,
                } => {
                    self.configure_gradient_background(
                        wgpu_ctx,
                        &mut container,
                        color_stops,
                        gradient_type,
                        angle,
                        center,
                        radius,
                        hover_effects,
                    );
                }
                ButtonBackground::Image(img_config) => {
                    self.configure_image_background(
                        wgpu_ctx,
                        &mut container,
                        img_config,
                        hover_effects,
                    );
                }
                ButtonBackground::FrostedGlass {
                    tint_color,
                    blur_radius,
                    opacity,
                } => {
                    self.configure_frosted_glass(
                        wgpu_ctx,
                        &mut container,
                        tint_color,
                        blur_radius,
                        opacity,
                        hover_effects,
                    );
                }
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

    fn configure_frosted_glass(
        &self,
        wgpu_ctx: &mut WgpuCtx<'_>,
        container: &mut Component,
        tint_color: Color,
        blur_radius: f32,
        opacity: f32,
        hover_effects: Option<ComponentHoverEffects>,
    ) {
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
        if let Some(hover_effects) = hover_effects {
            bg.set_hover_effects(hover_effects);
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

    fn configure_image_background(
        &self,
        wgpu_ctx: &mut WgpuCtx<'_>,
        container: &mut Component,
        img_config: ImageConfig,
        hover_effects: Option<ComponentHoverEffects>,
    ) {
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
        if let Some(hover_effects) = hover_effects {
            bg.set_hover_effects(hover_effects);
        }
        bg.configure(ComponentConfig::Image(img_config), wgpu_ctx);
        container.add_child(bg);
    }

    fn configure_gradient_background(
        &self,
        wgpu_ctx: &mut WgpuCtx<'_>,
        container: &mut Component,
        color_stops: Vec<GradientColorStop>,
        gradient_type: GradientType,
        angle: f32,
        center: Option<(f32, f32)>,
        radius: Option<f32>,
        hover_effects: Option<ComponentHoverEffects>,
    ) {
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
        if let Some(hover_effects) = hover_effects {
            bg.set_hover_effects(hover_effects);
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

    fn configure_color_background(
        &self,
        wgpu_ctx: &mut WgpuCtx<'_>,
        container: &mut Component,
        color: Color,
        hover_effects: Option<ComponentHoverEffects>,
    ) {
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
        if let Some(hover_effects) = hover_effects {
            bg.set_hover_effects(hover_effects);
        }
        bg.configure(
            ComponentConfig::BackgroundColor(BackgroundColorConfig { color }),
            wgpu_ctx,
        );
        container.add_child(bg);
    }
}
