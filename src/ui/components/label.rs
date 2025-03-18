use crate::{
    ui::{
        color::Color,
        component::{Component, ComponentConfig, ComponentType, TextConfig},
    },
    wgpu_ctx::WgpuCtx,
};
use uuid::Uuid;

use super::component_builder::{CommonBuilderProps, ComponentBuilder};

/// Builder for creating and configuring text label components
pub struct LabelBuilder {
    common: CommonBuilderProps,
    text: String,
    color: Color,
    font_size: f32,
    line_height: f32,
}

impl ComponentBuilder for LabelBuilder {
    fn common_props(&mut self) -> &mut CommonBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl LabelBuilder {
    /// Create a new label builder with the specified text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            common: CommonBuilderProps::default(),
            text: text.into(),
            color: Color::Black,
            font_size: 16.0,
            line_height: 1.0,
        }
    }

    /// Set the text color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the font size
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set the line height
    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height;
        self
    }

    /// Build and return the configured text label component
    pub fn build(mut self, wgpu_ctx: &mut WgpuCtx) -> Component {
        let id = Uuid::new_v4();
        let mut component = Component::new(id, ComponentType::Text);

        // Apply common configurations
        self.apply_common_props(&mut component);

        // Set a default debug name if not specified
        if component.debug_name.is_none() {
            component.set_debug_name(format!("Label: {}", self.text));
        }

        // Configure the text properties
        component.configure(
            ComponentConfig::Text(TextConfig {
                text: self.text,
                font_size: self.font_size,
                color: self.color,
                line_height: self.line_height,
            }),
            wgpu_ctx,
        );

        component
    }
}
