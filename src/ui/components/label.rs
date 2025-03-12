use crate::{
    color::Color,
    ui::{
        component::{Component, ComponentConfig, ComponentType, TextConfig},
        layout::{Anchor, Edges, FlexValue, Position},
    },
    wgpu_ctx::WgpuCtx,
};
use uuid::Uuid;

/// Builder for creating and configuring text label components
pub struct LabelBuilder {
    text: String,
    color: Color,
    font_size: f32,
    line_height: f32,
    size: Option<(f32, f32)>,
    position: Option<Position>,
    margin: Option<Edges>,
    padding: Option<Edges>,
    z_index: Option<i32>,
    debug_name: Option<String>,
}

impl LabelBuilder {
    /// Create a new label builder with the specified text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: Color::Black,
            font_size: 16.0,
            line_height: 1.0,
            size: None,
            position: None,
            margin: None,
            padding: None,
            z_index: None,
            debug_name: None,
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

    /// Set the text label size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Some((width, height));
        self
    }

    /// Set the width of the text label
    pub fn with_width(mut self, width: f32) -> Self {
        let height = self.size.map_or(0.0, |(_, h)| h);
        self.size = Some((width, height));
        self
    }

    /// Set the height of the text label
    pub fn with_height(mut self, height: f32) -> Self {
        let width = self.size.map_or(0.0, |(w, _)| w);
        self.size = Some((width, height));
        self
    }

    /// Set the position of the text label
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// Set text position to a fixed anchor relative to parent
    pub fn with_fixed_position(mut self, anchor: Anchor) -> Self {
        self.position = Some(Position::Fixed(anchor));
        self
    }

    /// Set the margin for the text label
    pub fn with_margin(mut self, margin: Edges) -> Self {
        self.margin = Some(margin);
        self
    }

    /// Set the padding for the text label
    pub fn with_padding(mut self, padding: Edges) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set the z-index for the text label
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = Some(z_index);
        self
    }

    /// Set a debug name for the text label component
    pub fn with_debug_name(mut self, name: impl Into<String>) -> Self {
        self.debug_name = Some(name.into());
        self
    }

    /// Build and return the configured text label component
    pub fn build(self, wgpu_ctx: &mut WgpuCtx) -> Component {
        let id = Uuid::new_v4();
        let mut component = Component::new(id, ComponentType::Text);

        // Apply configurations
        if let Some(debug_name) = self.debug_name {
            component.set_debug_name(debug_name);
        } else {
            component.set_debug_name(format!("Label: {}", self.text));
        }

        if let Some((width, height)) = self.size {
            component.transform.size.width = FlexValue::Fixed(width);
            component.transform.size.height = FlexValue::Fixed(height);
        }

        if let Some(position) = self.position {
            component.transform.position_type = position;
        }

        if let Some(z_index) = self.z_index {
            component.set_z_index(z_index);
        }

        if let Some(margin) = self.margin {
            component.layout.margin = margin;
        }

        if let Some(padding) = self.padding {
            component.layout.padding = padding;
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
