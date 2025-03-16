use crate::{
    ui::{
        Configurable, Positionable, Renderable,
        component::{Component, ComponentConfig, ComponentMetaData},
        layout::{Bounds, ComponentPosition, ComponentSize},
        text_renderer::OptionalTextUpdateData,
    },
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use log::debug;

pub struct TextComponent;

impl Configurable for TextComponent {
    fn configure(
        component: &mut Component,
        config: ComponentConfig,
        wgpu_ctx: &mut WgpuCtx,
    ) -> Vec<ComponentMetaData> {
        // we know config is of type ComponentConfig::Text
        let config = config.get_text_config().unwrap();

        wgpu_ctx.text_handler.register_text(
            component.id,
            config.text,
            config.font_size,
            config.line_height,
            Bounds::default(),
            config.color,
        );

        vec![]
    }
}

impl Renderable for TextComponent {
    fn draw(
        _component: &mut Component,
        _render_pass: &mut wgpu::RenderPass,
        _app_pipelines: &mut AppPipelines,
    ) {
        // Text rendering is done in a separate pass
    }
}

impl Positionable for TextComponent {
    fn set_position(component: &mut Component, wgpu_ctx: &mut WgpuCtx, bounds: Bounds) {
        let text_computed_bounds = wgpu_ctx.text_handler.measure_text(component.id);
        let calc_bounds = if let Some(text_size) = text_computed_bounds {
            if component.fit_to_size {
                component
                    .metadata
                    .push(ComponentMetaData::CanBeResizedTo(text_size));
            }
            if text_size.width == 0.0 || text_size.height == 0.0 {
                // Initial Layout is not yet computed, wait for next set_position call
                debug!(
                    "Text bounds not yet computed for component id: {}, waiting for next set_position call",
                    component.id
                );
                bounds
            } else {
                // center text use the x and y of bounds and the text size
                let x = bounds.position.x + (bounds.size.width - text_size.width) / 2.0;
                let y = bounds.position.y + (bounds.size.height - text_size.height) / 2.0;
                let position = ComponentPosition { x, y };
                let size = ComponentSize {
                    width: text_size.width,
                    height: text_size.height,
                };
                Bounds { position, size }
            }
        } else {
            bounds
        };

        wgpu_ctx.text_handler.update((
            component.id,
            OptionalTextUpdateData::new().with_bounds(calc_bounds),
        ));
    }
}
