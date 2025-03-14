use crate::{
    ui::{
        color::Color,
        component::{BorderPosition, Component, ComponentConfig, ComponentType, TextConfig},
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
    width: Option<FlexValue>,
    height: Option<FlexValue>,
    position: Option<Position>,
    margin: Option<Edges>,
    padding: Option<Edges>,
    z_index: Option<i32>,
    debug_name: Option<String>,
    border_width: Option<f32>,
    border_color: Option<Color>,
    border_position: Option<BorderPosition>,
}

#[allow(dead_code)]
impl LabelBuilder {
    /// Create a new label builder with the specified text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: Color::Black,
            font_size: 16.0,
            line_height: 1.0,
            width: None,
            height: None,
            position: None,
            margin: None,
            padding: None,
            z_index: None,
            debug_name: None,
            border_width: None,
            border_color: None,
            border_position: None,
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
    pub fn with_size(mut self, width: impl Into<FlexValue>, height: impl Into<FlexValue>) -> Self {
        self.width = Some(width.into());
        self.height = Some(height.into());
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

    /// Set both border width and color in one call
    pub fn with_border(mut self, width: f32, color: Color) -> Self {
        self.border_width = Some(width);
        self.border_color = Some(color);
        self
    }

    /// Set border width, color, and position in one call
    pub fn with_border_full(mut self, width: f32, color: Color, position: BorderPosition) -> Self {
        self.border_width = Some(width);
        self.border_color = Some(color);
        self.border_position = Some(position);
        self
    }

    /// Set the border position
    pub fn with_border_position(mut self, position: BorderPosition) -> Self {
        self.border_position = Some(position);
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

        if let Some(width) = self.width {
            component.transform.size.width = width;
        }

        if let Some(height) = self.height {
            component.transform.size.height = height;
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

        if let Some(border_width) = self.border_width {
            component.border_width = border_width;
        }

        if let Some(border_color) = self.border_color {
            component.border_color = border_color;
        }

        if let Some(border_position) = self.border_position {
            component.set_border_position(border_position);
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
