use component::Component;

use crate::{
    ui::{
        components::core::component::{ComponentConfig, ComponentMetaData},
        layout::{Bounds, ComponentSize},
    },
    wgpu_ctx::{AppPipelines, WgpuCtx},
};

pub mod background_color;
pub mod background_gradient;
pub mod component;
pub mod image;
pub mod text;

pub trait Configurable {
    fn configure(
        component: &mut Component,
        config: ComponentConfig,
        wgpu_ctx: &mut WgpuCtx,
    ) -> Vec<ComponentMetaData>;
}

pub trait Renderable {
    fn draw(
        component: &Component,
        render_pass: &mut wgpu::RenderPass,
        app_pipelines: &mut AppPipelines,
    );
}

pub trait Positionable {
    fn set_position(
        component: &mut Component,
        wgpu_ctx: &mut WgpuCtx,
        bounds: Bounds,
        screen_size: ComponentSize,
    );
}
