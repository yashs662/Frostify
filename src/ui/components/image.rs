use crate::{
    ui::{
        color::Color,
        component::{BorderPosition, Component, ComponentConfig, ComponentType, ImageConfig},
        layout::{Anchor, BorderRadius, Edges, FlexValue, Position},
    },
    wgpu_ctx::WgpuCtx,
};
use uuid::Uuid;

/// Builder for creating and configuring image components
pub struct ImageBuilder {
    file_name: String,
    width: Option<FlexValue>,
    height: Option<FlexValue>,
    position: Option<Position>,
    border_radius: Option<BorderRadius>,
    margin: Option<Edges>,
    padding: Option<Edges>,
    z_index: Option<i32>,
    debug_name: Option<String>,
    scale_mode: ScaleMode,
    border_width: Option<f32>,
    border_color: Option<Color>,
    border_position: Option<BorderPosition>,
}

/// Defines how an image should be scaled to fit its container
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum ScaleMode {
    /// Stretch the image to fill the entire container (default)
    Stretch,
    /// Maintain aspect ratio, scale to fit while ensuring entire image is visible
    Contain,
    /// Maintain aspect ratio, scale to cover entire container (may crop)
    Cover,
    /// Don't scale the image (use original dimensions)
    Original,
}

impl Default for ScaleMode {
    fn default() -> Self {
        Self::Stretch
    }
}

#[allow(dead_code)]
impl ImageBuilder {
    /// Create a new image builder with the specified image file
    pub fn new(file_name: impl Into<String>) -> Self {
        Self {
            file_name: file_name.into(),
            width: None,
            height: None,
            position: None,
            border_radius: None,
            margin: None,
            padding: None,
            z_index: None,
            debug_name: None,
            scale_mode: ScaleMode::default(),
            border_width: None,
            border_color: None,
            border_position: None,
        }
    }

    /// Set the image size
    pub fn with_size(mut self, width: impl Into<FlexValue>, height: impl Into<FlexValue>) -> Self {
        self.width = Some(width.into());
        self.height = Some(height.into());
        self
    }

    /// Set the position of the image
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// Set image position to a fixed anchor relative to parent
    pub fn with_fixed_position(mut self, anchor: Anchor) -> Self {
        self.position = Some(Position::Fixed(anchor));
        self
    }

    /// Set the border radius for the image
    pub fn with_border_radius(mut self, radius: BorderRadius) -> Self {
        self.border_radius = Some(radius);
        self
    }

    /// Set a uniform border radius for all corners
    pub fn with_uniform_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(BorderRadius::all(radius));
        self
    }

    /// Set the margin for the image
    pub fn with_margin(mut self, margin: Edges) -> Self {
        self.margin = Some(margin);
        self
    }

    /// Set the padding for the image
    pub fn with_padding(mut self, padding: Edges) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set the z-index for the image
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = Some(z_index);
        self
    }

    /// Set a debug name for the image component
    pub fn with_debug_name(mut self, name: impl Into<String>) -> Self {
        self.debug_name = Some(name.into());
        self
    }

    /// Set how the image should be scaled to fit its container
    pub fn with_scale_mode(mut self, scale_mode: ScaleMode) -> Self {
        self.scale_mode = scale_mode;
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

    /// Build and return the configured image component
    pub fn build(self, wgpu_ctx: &mut WgpuCtx) -> Component {
        let id = Uuid::new_v4();
        let mut component = Component::new(id, ComponentType::Image);

        // Apply configurations
        if let Some(debug_name) = self.debug_name {
            component.set_debug_name(debug_name);
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

        if let Some(border_radius) = self.border_radius {
            component.set_border_radius(border_radius);
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

        // Configure the image with scale mode
        component.configure(
            ComponentConfig::Image(ImageConfig {
                file_name: self.file_name,
                scale_mode: self.scale_mode,
            }),
            wgpu_ctx,
        );

        component
    }
}
