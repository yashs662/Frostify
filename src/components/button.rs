use crate::{
    app::AppEvent,
    components::core::{
        background::BackgroundComponent, image::ImageComponent, Bounds, Component,
        ComponentBackgroundConfig, ComponentPosition, ComponentTransform,
    },
    wgpu_ctx::{self, AppPipelines, PipelinePreference, WgpuCtx},
};
use log::{error, trace};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use super::core::{text::TextComponent, ComponentRenderable, ComponentType};

pub struct Button {
    id: Uuid,
    renderable: Option<ComponentRenderable>,
    on_click: AppEvent,
    children: Vec<Box<dyn Component>>,
    bounds: Bounds,
    event_sender: Option<UnboundedSender<AppEvent>>,
}

impl Button {
    pub fn new(
        wgpu_ctx: &mut wgpu_ctx::WgpuCtx,
        background: ComponentBackgroundConfig,
        transform: ComponentTransform,
        parent_bounds: Option<Bounds>,
        on_click: AppEvent,
        event_sender: Option<UnboundedSender<AppEvent>>,
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
        let bounds = Bounds::new(position, size);

        let renderable = match background {
            ComponentBackgroundConfig::None => None,
            ComponentBackgroundConfig::Image(path) => {
                trace!("Creating new button with id: {:?}, image path: {:?}", id, path);
                Some(ComponentRenderable::Image(ImageComponent::new(
                    &wgpu_ctx.device,
                    &wgpu_ctx.queue,
                    &path,
                    size,
                    position,
                )))
            }
            ComponentBackgroundConfig::Color { color } => {
                Some(ComponentRenderable::Background(BackgroundComponent::new(
                    &wgpu_ctx.device,
                    bounds,
                    color,
                    color,
                    0.0,
                    PipelinePreference::Color,
                )))
            }
            ComponentBackgroundConfig::Gradient {
                start_color,
                end_color,
                angle,
            } => {
                Some(ComponentRenderable::Background(BackgroundComponent::new(
                    &wgpu_ctx.device,
                    bounds,
                    start_color,
                    end_color,
                    angle.to_radians(),
                    PipelinePreference::Color,
                )))
            }
            ComponentBackgroundConfig::Text(text_config) => {
                let text_component = TextComponent::new(
                    text_config.text,
                    16.0,
                    1.0,
                    text_config.color,
                    bounds,
                    &mut wgpu_ctx.text_handler,
                    text_config.anchor,
                );
                Some(ComponentRenderable::Text(text_component))
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
                Some(ComponentRenderable::TextOnBackground(
                    text_component,
                    bg_component,
                ))
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
                Some(ComponentRenderable::TextOnBackground(
                    text_component,
                    bg_component,
                ))
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
                Some(ComponentRenderable::TextOnImage(
                    text_component,
                    img_component,
                ))
            }
        };

        Self {
            id,
            renderable,
            on_click,
            children: Vec::new(),
            bounds,
            event_sender,
        }
    }

    fn check_bounds(&self, x: f32, y: f32) -> bool {
        // Convert screen coordinates to the same coordinate space as the button
        let bounds = self.get_bounds();
        x >= bounds.position.x
            && x <= bounds.position.x + bounds.size.width
            && y >= bounds.position.y
            && y <= bounds.position.y + bounds.size.height
    }
}

impl Component for Button {
    fn id(&self) -> Uuid {
        self.id
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn component_type(&self) -> super::core::ComponentType {
        ComponentType::Other
    }

    fn send_event(&self, event: AppEvent) {
        if let Some(sender) = &self.event_sender {
            match sender.send(event.clone()) {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to send event {:?}, error: {:?}", event, err);
                }
            }
        }
    }

    fn update(&mut self, queue: &wgpu::Queue) {
        // if self renderable is of ComponentRenderable::Image, update it
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

        // Draw children
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
                    img_component.set_position(&wgpu_ctx.queue, &wgpu_ctx.device, self.bounds.position);
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

        // Resize children
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
        let mut event_to_send = None;
        match &mut self.renderable {
            Some(ComponentRenderable::Background(bg_component)) => {
                bg_component.set_position(queue, position, self.bounds);
            }
            Some(ComponentRenderable::Image(img_component)) => {
                img_component.set_position(queue, device, position);
            }
            Some(ComponentRenderable::Text(text_component)) => {
                event_to_send = Some(AppEvent::SetPositionText(text_component.id, position));
            }
            Some(ComponentRenderable::TextOnBackground(text_component, bg_component)) => {
                event_to_send = Some(AppEvent::SetPositionText(text_component.id, position));
                bg_component.set_position(queue, position, self.bounds);
            }
            Some(ComponentRenderable::TextOnImage(text_component, img_component)) => {
                event_to_send = Some(AppEvent::SetPositionText(text_component.id, position));
                img_component.set_position(queue, device, position);
            }
            None => {}
        }

        if let Some(event) = event_to_send {
            self.send_event(event);
        }
    }

    fn handle_mouse_click(&mut self, x: f32, y: f32) -> bool {
        // First check if any children handle the click
        for child in &mut self.children {
            if child.handle_mouse_click(x, y) {
                return true;
            }
        }

        if self.check_bounds(x, y) {
            self.send_event(self.on_click.clone());
            true
        } else {
            false
        }
    }

    fn add_child(&mut self, child: Box<dyn Component>) {
        self.children.push(child);
    }

    fn get_bounds(&self) -> Bounds {
        self.bounds
    }
}
