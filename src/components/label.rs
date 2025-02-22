use crate::{
    app::AppEvent,
    components::core::{
        background::BackgroundComponent, image::ImageComponent, Bounds, Component,
        ComponentBackgroundConfig, ComponentPosition, ComponentTransform,
    },
    wgpu_ctx::{self, AppPipelines, PipelinePreference, WgpuCtx},
};
use log::{info, trace};
use uuid::Uuid;

use super::core::{text::TextComponent, ComponentRenderable, ComponentSize, ComponentType};

pub struct Label {
    id: Uuid,
    renderable: Option<ComponentRenderable>,
    children: Vec<Box<dyn Component>>,
    bounds: Bounds,
}

impl Label {
    pub fn new(
        wgpu_ctx: &mut wgpu_ctx::WgpuCtx,
        background: ComponentBackgroundConfig,
        transform: ComponentTransform,
        parent_bounds: Option<Bounds>,
    ) -> Self {
        let id = Uuid::new_v4();
        let ComponentTransform {
            size,
            offset,
            anchor,
        } = transform;
        let (x, y) = if let Some(bounds) = parent_bounds {
            let (anchor_x, anchor_y) = bounds.get_anchor_position(anchor);
            (anchor_x + offset.x, anchor_y + offset.y)
        } else {
            (offset.x, offset.y)
        };

        let position = ComponentPosition { x, y };
        let mut bounds = Bounds::new(position, size);
        let (renderable, text_component_id) = match background {
            ComponentBackgroundConfig::None
            | ComponentBackgroundConfig::Image(_)
            | ComponentBackgroundConfig::Color { .. }
            | ComponentBackgroundConfig::Gradient { .. } => {
                panic!("labels cannot be made without a text component")
            }
            ComponentBackgroundConfig::Text(text_config) => {
                trace!("Creating label with id: {}, text: {}", id, text_config.text);
                let text_component = TextComponent::new(
                    text_config.text,
                    16.0,
                    1.0,
                    text_config.color,
                    bounds,
                    &mut wgpu_ctx.text_handler,
                    text_config.anchor,
                );
                let text_component_id = text_component.id;
                (
                    Some(ComponentRenderable::Text(text_component)),
                    text_component_id,
                )
            }
            ComponentBackgroundConfig::TextOnColor(text_on_color_config) => {
                let text_component = TextComponent::new(
                    text_on_color_config.text,
                    16.0,
                    1.0,
                    text_on_color_config.text_color,
                    bounds,
                    &mut wgpu_ctx.text_handler,
                    text_on_color_config.anchor,
                );
                let bg_component = BackgroundComponent::new(
                    &wgpu_ctx.device,
                    bounds,
                    text_on_color_config.background_color,
                    text_on_color_config.background_color,
                    0.0,
                    PipelinePreference::Color,
                );
                let text_component_id = text_component.id;
                (
                    Some(ComponentRenderable::TextOnBackground(
                        text_component,
                        bg_component,
                    )),
                    text_component_id,
                )
            }
            ComponentBackgroundConfig::TextOnGradient(text_on_gradient_config) => {
                let text_component = TextComponent::new(
                    text_on_gradient_config.text,
                    16.0,
                    1.0,
                    text_on_gradient_config.text_color,
                    bounds,
                    &mut wgpu_ctx.text_handler,
                    text_on_gradient_config.anchor,
                );
                let bg_component = BackgroundComponent::new(
                    &wgpu_ctx.device,
                    bounds,
                    text_on_gradient_config.start_color,
                    text_on_gradient_config.end_color,
                    text_on_gradient_config.angle.to_radians(),
                    PipelinePreference::Color,
                );
                let text_component_id = text_component.id;
                (
                    Some(ComponentRenderable::TextOnBackground(
                        text_component,
                        bg_component,
                    )),
                    text_component_id,
                )
            }
            ComponentBackgroundConfig::TextOnImage(text_on_img_config) => {
                let text_component = TextComponent::new(
                    text_on_img_config.text,
                    16.0,
                    1.0,
                    text_on_img_config.text_color,
                    bounds,
                    &mut wgpu_ctx.text_handler,
                    text_on_img_config.anchor,
                );
                let img_component = ImageComponent::new(
                    &wgpu_ctx.device,
                    &wgpu_ctx.queue,
                    &text_on_img_config.image_path,
                    size,
                    position,
                );
                let text_component_id = text_component.id;
                (
                    Some(ComponentRenderable::TextOnImage(
                        text_component,
                        img_component,
                    )),
                    text_component_id,
                )
            }
        };

        let desired_width = wgpu_ctx.text_handler.measure_text(text_component_id);
        if let Some(desired_width) = desired_width {
            let new_size = ComponentSize {
                width: desired_width.width,
                height: size.height,
            };
            let new_bounds = Bounds::new(position, new_size);
            info!(
                "Resizing label with id: {}, old size: {:#?}, new size: {:#?}",
                id, size, new_size
            );
            bounds = new_bounds;
        }
        // TODO: also resize the color, gradient, and image components

        Self {
            id,
            renderable,
            children: Vec::new(),
            bounds,
        }
    }
}

impl Component for Label {
    fn id(&self) -> Uuid {
        self.id
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn component_type(&self) -> super::core::ComponentType {
        ComponentType::Other
    }

    fn send_event(&self, _event: AppEvent) {
        // Labels don't send events
    }

    fn update(&mut self, queue: &wgpu::Queue) {
        if let Some(ComponentRenderable::Image(img)) = &mut self.renderable {
            img.update(queue);
        }
    }

    fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        app_pipelines: &mut AppPipelines,
    ) {
        if let Some(renderable) = &self.renderable {
            renderable.draw(render_pass, app_pipelines);
        }

        for child in &self.children {
            child.draw(render_pass, app_pipelines);
        }
    }

    fn resize(&mut self, wgpu_ctx: &WgpuCtx, width: u32, height: u32) {
        if let Some(renderable) = &mut self.renderable {
            match renderable {
                ComponentRenderable::Background(bg_component) => {
                    bg_component.resize(&wgpu_ctx.queue, width, height, self.bounds);
                }
                ComponentRenderable::Image(img_component) => {
                    img_component.set_position(
                        &wgpu_ctx.queue,
                        &wgpu_ctx.device,
                        self.bounds.position,
                    );
                    img_component.resize(wgpu_ctx, width, height);
                }
                ComponentRenderable::Text(_) => {}
                ComponentRenderable::TextOnBackground(_, bg_component) => {
                    bg_component.resize(&wgpu_ctx.queue, width, height, self.bounds);
                }
                ComponentRenderable::TextOnImage(_, img_component) => {
                    img_component.resize(wgpu_ctx, width, height);
                }
            }
        }

        for child in &mut self.children {
            child.resize(wgpu_ctx, width, height);
        }
    }

    fn set_position(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        position: ComponentPosition,
    ) {
        self.bounds.position = position;

        match &mut self.renderable {
            Some(ComponentRenderable::Background(bg_component)) => {
                bg_component.set_position(queue, position, self.bounds);
            }
            Some(ComponentRenderable::Image(img_component)) => {
                img_component.set_position(queue, device, position);
            }
            Some(ComponentRenderable::Text(_)) => {
                // Text position is handled by the text renderer
            }
            Some(ComponentRenderable::TextOnBackground(_, bg_component)) => {
                bg_component.set_position(queue, position, self.bounds);
            }
            Some(ComponentRenderable::TextOnImage(_, img_component)) => {
                img_component.set_position(queue, device, position);
            }
            None => {}
        }
    }

    fn handle_mouse_click(&mut self, _x: f32, _y: f32) -> bool {
        // Labels don't handle clicks
        false
    }

    fn add_child(&mut self, child: Box<dyn Component>) {
        self.children.push(child);
    }

    fn get_bounds(&self) -> Bounds {
        self.bounds
    }
}
