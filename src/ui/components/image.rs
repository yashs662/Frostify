use crate::{
    ui::{
        component::{Component, ComponentConfig, ComponentType, ImageConfig},
        components::component_builder::{CommonBuilderProps, ComponentBuilder},
    },
    wgpu_ctx::WgpuCtx,
};
use uuid::Uuid;

/// Builder for creating and configuring image components
pub struct ImageBuilder {
    common: CommonBuilderProps,
    file_name: String,
    scale_mode: ScaleMode,
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

impl ComponentBuilder for ImageBuilder {
    fn common_props(&mut self) -> &mut CommonBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl ImageBuilder {
    pub fn new(file_name: impl Into<String>) -> Self {
        Self {
            common: CommonBuilderProps::default(),
            file_name: file_name.into(),
            scale_mode: ScaleMode::default(),
        }
    }

    pub fn with_scale_mode(mut self, scale_mode: ScaleMode) -> Self {
        self.scale_mode = scale_mode;
        self
    }

    pub fn build(mut self, wgpu_ctx: &mut WgpuCtx) -> Component {
        let id = Uuid::new_v4();
        let mut component = Component::new(id, ComponentType::Image);

        self.apply_common_props(&mut component, wgpu_ctx);

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
