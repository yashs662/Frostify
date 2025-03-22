use crate::{
    ui::{
        color::Color,
        component::{
            BackgroundColorConfig, BackgroundGradientConfig, Component, ComponentConfig,
            ComponentType, FrostedGlassConfig, GradientColorStop, GradientType,
        },
        components::component_builder::{CommonBuilderProps, ComponentBuilder},
    },
    wgpu_ctx::WgpuCtx,
};
use uuid::Uuid;

/// Builder for creating and configuring background components
pub struct BackgroundBuilder {
    common: CommonBuilderProps,
    background_type: BackgroundType,
}

#[allow(dead_code)]
pub enum BackgroundType {
    Color(Color),
    Gradient {
        color_stops: Vec<GradientColorStop>,
        gradient_type: GradientType,
        angle: f32,
        center: Option<(f32, f32)>,
        radius: Option<f32>,
    },
    FrostedGlass {
        tint_color: Color,
        blur_radius: f32,
        opacity: f32,
    },
}

impl ComponentBuilder for BackgroundBuilder {
    fn common_props(&mut self) -> &mut CommonBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl BackgroundBuilder {
    pub fn with_color(color: Color) -> Self {
        Self {
            common: CommonBuilderProps::default(),
            background_type: BackgroundType::Color(color),
        }
    }

    pub fn with_linear_gradient(color_stops: Vec<GradientColorStop>, angle_degrees: f32) -> Self {
        Self {
            common: CommonBuilderProps::default(),
            background_type: BackgroundType::Gradient {
                color_stops,
                gradient_type: GradientType::Linear,
                angle: angle_degrees,
                center: None,
                radius: None,
            },
        }
    }

    pub fn with_radial_gradient(
        color_stops: Vec<GradientColorStop>,
        center: (f32, f32),
        radius: f32,
    ) -> Self {
        Self {
            common: CommonBuilderProps::default(),
            background_type: BackgroundType::Gradient {
                color_stops,
                gradient_type: GradientType::Radial,
                angle: 0.0,
                center: Some(center),
                radius: Some(radius),
            },
        }
    }

    pub fn with_frosted_glass(tint_color: Color, blur_radius: f32, opacity: f32) -> Self {
        Self {
            common: CommonBuilderProps::default(),
            background_type: BackgroundType::FrostedGlass {
                tint_color,
                blur_radius,
                opacity,
            },
        }
    }

    pub fn build(mut self, wgpu_ctx: &mut WgpuCtx) -> Component {
        let component_type = match &self.background_type {
            BackgroundType::Color(_) => ComponentType::BackgroundColor,
            BackgroundType::Gradient { .. } => ComponentType::BackgroundGradient,
            BackgroundType::FrostedGlass { .. } => ComponentType::FrostedGlass,
        };

        let id = Uuid::new_v4();
        let mut component = Component::new(id, component_type);

        self.apply_common_props(&mut component, wgpu_ctx);

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
