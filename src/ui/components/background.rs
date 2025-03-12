use crate::{
    color::Color,
    ui::{
        component::{
            BackgroundColorConfig, BackgroundGradientConfig, Component, ComponentConfig,
            ComponentType, FrostedGlassConfig, GradientColorStop, GradientType,
        },
        layout::{Anchor, BorderRadius, Edges, FlexValue, Position},
    },
    wgpu_ctx::WgpuCtx,
};
use uuid::Uuid;

/// Builder for creating and configuring background components
pub struct BackgroundBuilder {
    background_type: BackgroundType,
    width: Option<FlexValue>,
    height: Option<FlexValue>,
    position: Option<Position>,
    border_radius: Option<BorderRadius>,
    margin: Option<Edges>,
    padding: Option<Edges>,
    z_index: Option<i32>,
    debug_name: Option<String>,
}

/// Types of background supported by the builder
#[allow(dead_code)]
pub enum BackgroundType {
    /// Solid color background
    Color(Color),
    /// Gradient background
    Gradient {
        color_stops: Vec<GradientColorStop>,
        gradient_type: GradientType,
        angle: f32,
        center: Option<(f32, f32)>,
        radius: Option<f32>,
    },
    /// Frosted glass background
    FrostedGlass {
        tint_color: Color,
        blur_radius: f32,
        opacity: f32,
    },
}

#[allow(dead_code)]
impl BackgroundBuilder {
    /// Create a new background builder with a solid color
    pub fn with_color(color: Color) -> Self {
        Self {
            background_type: BackgroundType::Color(color),
            width: None,
            height: None,
            position: None,
            border_radius: None,
            margin: None,
            padding: None,
            z_index: None,
            debug_name: None,
        }
    }

    /// Create a new background builder with a linear gradient
    pub fn with_linear_gradient(color_stops: Vec<GradientColorStop>, angle_degrees: f32) -> Self {
        Self {
            background_type: BackgroundType::Gradient {
                color_stops,
                gradient_type: GradientType::Linear,
                angle: angle_degrees,
                center: None,
                radius: None,
            },
            width: None,
            height: None,
            position: None,
            border_radius: None,
            margin: None,
            padding: None,
            z_index: None,
            debug_name: None,
        }
    }

    /// Create a new background builder with a radial gradient
    pub fn with_radial_gradient(
        color_stops: Vec<GradientColorStop>,
        center: (f32, f32),
        radius: f32,
    ) -> Self {
        Self {
            background_type: BackgroundType::Gradient {
                color_stops,
                gradient_type: GradientType::Radial,
                angle: 0.0,
                center: Some(center),
                radius: Some(radius),
            },
            width: None,
            height: None,
            position: None,
            border_radius: None,
            margin: None,
            padding: None,
            z_index: None,
            debug_name: None,
        }
    }

    /// Create a new background builder with a frosted glass effect
    pub fn with_frosted_glass(tint_color: Color, blur_radius: f32, opacity: f32) -> Self {
        Self {
            background_type: BackgroundType::FrostedGlass {
                tint_color,
                blur_radius,
                opacity,
            },
            width: None,
            height: None,
            position: None,
            border_radius: None,
            margin: None,
            padding: None,
            z_index: None,
            debug_name: None,
        }
    }

    /// Set the background size with FlexValues
    pub fn with_size(mut self, width: impl Into<FlexValue>, height: impl Into<FlexValue>) -> Self {
        self.width = Some(width.into());
        self.height = Some(height.into());
        self
    }

    /// Set the width of the background using FlexValue
    pub fn with_width(mut self, width: impl Into<FlexValue>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Set the height of the background using FlexValue
    pub fn with_height(mut self, height: impl Into<FlexValue>) -> Self {
        self.height = Some(height.into());
        self
    }

    /// Set the position of the background
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// Set background position to a fixed anchor relative to parent
    pub fn with_fixed_position(mut self, anchor: Anchor) -> Self {
        self.position = Some(Position::Fixed(anchor));
        self
    }

    /// Set the border radius for the background
    pub fn with_border_radius(mut self, radius: BorderRadius) -> Self {
        self.border_radius = Some(radius);
        self
    }

    /// Set a uniform border radius for all corners
    pub fn with_uniform_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = Some(BorderRadius::all(radius));
        self
    }

    /// Set the margin for the background
    pub fn with_margin(mut self, margin: Edges) -> Self {
        self.margin = Some(margin);
        self
    }

    /// Set the padding for the background
    pub fn with_padding(mut self, padding: Edges) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Set the z-index for the background
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = Some(z_index);
        self
    }

    /// Set a debug name for the background component
    pub fn with_debug_name(mut self, name: impl Into<String>) -> Self {
        self.debug_name = Some(name.into());
        self
    }

    /// Build and return the configured background component
    pub fn build(self, wgpu_ctx: &mut WgpuCtx) -> Component {
        let id = Uuid::new_v4();

        // Create the component with the appropriate type
        let component_type = match &self.background_type {
            BackgroundType::Color(_) => ComponentType::BackgroundColor,
            BackgroundType::Gradient { .. } => ComponentType::BackgroundGradient,
            BackgroundType::FrostedGlass { .. } => ComponentType::FrostedGlass,
        };

        let mut component = Component::new(id, component_type);

        // Apply configurations
        if let Some(debug_name) = self.debug_name {
            component.set_debug_name(debug_name);
        }

        // Set width and height if provided
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

        // Configure the background based on the specified type
        match self.background_type {
            BackgroundType::Color(color) => {
                component.configure(
                    ComponentConfig::BackgroundColor(BackgroundColorConfig { color }),
                    wgpu_ctx,
                );
            }
            BackgroundType::Gradient {
                color_stops,
                gradient_type,
                angle,
                center,
                radius,
            } => {
                component.configure(
                    ComponentConfig::BackgroundGradient(BackgroundGradientConfig {
                        color_stops,
                        gradient_type,
                        angle,
                        center,
                        radius,
                    }),
                    wgpu_ctx,
                );
            }
            BackgroundType::FrostedGlass {
                tint_color,
                blur_radius,
                opacity,
            } => {
                component.configure(
                    ComponentConfig::FrostedGlass(FrostedGlassConfig {
                        tint_color,
                        blur_radius,
                        opacity,
                    }),
                    wgpu_ctx,
                );
            }
        }

        component
    }
}

// Implement From<f32> for FlexValue to allow for convenient conversions
impl From<f32> for FlexValue {
    fn from(value: f32) -> Self {
        FlexValue::Fixed(value)
    }
}
