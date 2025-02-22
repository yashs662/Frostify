use crate::{
    app::AppEvent,
    components::core::{
        image::ImageComponent, Anchor, Bounds, Component, ComponentOffset, ComponentPosition,
        ComponentSize, ComponentTransform,
    },
    wgpu_ctx::{AppPipelines, PipelinePreference, WgpuCtx},
};
use log::{info, trace, warn};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use super::core::{
    background::BackgroundComponent, ComponentBackgroundConfig, ComponentRenderable, ComponentType,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexAlign {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

pub struct Container {
    id: Uuid,
    children: Vec<Box<dyn Component>>,
    bounds: Bounds,
    direction: FlexDirection,
    // TODO: Differentiate between self and children alignment
    vertical_alignment: FlexAlign,
    horizontal_alignment: FlexAlign,
    padding: f32,
    gap: f32,
    anchor: Anchor,
    offset: ComponentOffset,
    parent_bounds: Option<Bounds>,
    screen_size: (u32, u32),
    event_sender: Option<Sender<AppEvent>>,
    renderable: Option<ComponentRenderable>,
    children_has_nested_containers: bool,
}

impl Container {
    pub fn new(
        wgpu_ctx: &WgpuCtx,
        transform: ComponentTransform,
        parent_bounds: Option<Bounds>,
        direction: FlexDirection,
        horizontal_alignment: FlexAlign,
        vertical_alignment: FlexAlign,
        event_sender: Option<Sender<AppEvent>>,
        background: ComponentBackgroundConfig,
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
                trace!(
                    "Creating new button with id: {:?}, image path: {:?}",
                    id,
                    path
                );
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
            } => Some(ComponentRenderable::Background(BackgroundComponent::new(
                &wgpu_ctx.device,
                bounds,
                start_color,
                end_color,
                angle.to_radians(),
                PipelinePreference::Color,
            ))),
            ComponentBackgroundConfig::Text(_)
            | ComponentBackgroundConfig::TextOnColor(_)
            | ComponentBackgroundConfig::TextOnGradient(_)
            | ComponentBackgroundConfig::TextOnImage(_) => {
                panic!("containers cannot be made with text components as backgrounds")
            }
        };

        Self {
            id,
            children: Vec::new(),
            bounds,
            direction,
            vertical_alignment,
            horizontal_alignment,
            padding: 0.0,
            gap: 0.0,
            anchor,
            offset,
            parent_bounds,
            screen_size: (0, 0),
            event_sender,
            renderable,
            children_has_nested_containers: false,
        }
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    fn layout_children(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        if self.children.is_empty() {
            return;
        }

        let content_width = self.bounds.size.width - (self.padding * 2.0);
        let content_height = self.bounds.size.height - (self.padding * 2.0);

        match self.direction {
            FlexDirection::Row => self.layout_row(queue, device, content_width, content_height),
            FlexDirection::Column => {
                self.layout_column(queue, device, content_width, content_height)
            }
        }
    }

    fn get_total_children_width(&self) -> f32 {
        self.children
            .iter()
            .map(|child| child.get_bounds().size.width)
            .sum()
    }

    fn get_total_children_height(&self) -> f32 {
        self.children
            .iter()
            .map(|child| child.get_bounds().size.height)
            .sum()
    }

    fn get_max_child_width(&self) -> f32 {
        self.children
            .iter()
            .map(|child| child.get_bounds().size.width)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    fn get_max_child_height(&self) -> f32 {
        self.children
            .iter()
            .map(|child| {
                if child.component_type() == ComponentType::Container {
                    let container = child.as_any().downcast_ref::<Container>().unwrap();
                    container.get_max_child_height()
                } else {
                    child.get_bounds().size.height
                }
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    fn layout_row(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, width: f32, height: f32) {
        // Calculate total width of all children
        let total_children_width: f32 = self.get_total_children_width();
        let max_child_height = self.get_max_child_height();
        let total_children_width = total_children_width.min(width).max(0.0);

        // Calculate total spacing between elements
        let available_space = width - total_children_width;
        let spacing_count = if self.horizontal_alignment == FlexAlign::SpaceAround {
            self.children.len() as f32 + 1.0
        } else {
            self.children.len().saturating_sub(1) as f32
        };

        if self.horizontal_alignment == FlexAlign::SpaceBetween
            || self.horizontal_alignment == FlexAlign::SpaceAround
        {
            if spacing_count > 0.0 {
                self.gap = available_space / spacing_count;
            }
        }

        let unit_spacing = (self.gap * spacing_count).min(available_space);
        let start_x = match self.anchor {
            Anchor::TopLeft | Anchor::Left | Anchor::BottomLeft => {
                let wrt_anchor = self.bounds.position.x + self.padding;
                match self.horizontal_alignment {
                    FlexAlign::Start | FlexAlign::SpaceBetween => wrt_anchor,
                    FlexAlign::End => wrt_anchor + unit_spacing,
                    FlexAlign::SpaceAround => wrt_anchor + (unit_spacing / spacing_count),
                    FlexAlign::Center => wrt_anchor + (unit_spacing / 2.0),
                }
            }
            Anchor::Top | Anchor::Center | Anchor::Bottom => {
                let wrt_anchor = self.bounds.position.x
                    + ((width - total_children_width) / 2.0).max(self.padding);
                match self.horizontal_alignment {
                    FlexAlign::Start
                    | FlexAlign::SpaceBetween
                    | FlexAlign::End
                    | FlexAlign::Center => wrt_anchor,
                    FlexAlign::SpaceAround => wrt_anchor + (unit_spacing / spacing_count),
                }
            }
            Anchor::TopRight | Anchor::Right | Anchor::BottomRight => {
                let wrt_anchor = self.bounds.position.x + width + self.padding
                    - total_children_width
                    - (unit_spacing * spacing_count);
                match self.horizontal_alignment {
                    FlexAlign::Start | FlexAlign::Center | FlexAlign::End => wrt_anchor,
                    FlexAlign::SpaceAround => {
                        wrt_anchor - unit_spacing + (unit_spacing / spacing_count)
                    }
                    FlexAlign::SpaceBetween => wrt_anchor - unit_spacing,
                }
            }
        };

        if height < max_child_height {
            warn!(
                "\nNot enough space for children, available_height_space: {}, max_child_height: {}\n",
                height, max_child_height
            );
        }
        
        // Position each child
        let mut current_x = start_x;
        for child in &mut self.children {
            let child_bounds = child.get_bounds();
            let child_y = match self.vertical_alignment {
                FlexAlign::Start => self.bounds.position.y + self.padding,
                FlexAlign::Center | FlexAlign::SpaceAround | FlexAlign::SpaceBetween => {
                    self.bounds.position.y + ((height - child_bounds.size.height) / 2.0).max(0.0) + self.padding
                }
                FlexAlign::End => {
                    self.bounds.position.y + self.bounds.size.height
                        - child_bounds.size.height
                        - self.padding
                }
            };

            // Create a new position for the child
            let new_position = ComponentPosition {
                x: current_x,
                y: child_y,
            };

            // Update the child's position
            child.set_position(queue, device, new_position);

            current_x += child_bounds.size.width + unit_spacing;
        }
    }

    fn layout_column(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        width: f32,
        height: f32,
    ) {
        let total_spacing = self.gap * (self.children.len() - 1) as f32;
        let available_height = height - total_spacing;

        let mut y_offset = match self.horizontal_alignment {
            FlexAlign::Start => self.bounds.position.y + self.padding,
            FlexAlign::Center => {
                self.bounds.position.y + (height - available_height - total_spacing) / 2.0
            }
            FlexAlign::End => self.bounds.position.y + height - available_height - total_spacing,
            FlexAlign::SpaceBetween => self.bounds.position.y + self.padding,
            FlexAlign::SpaceAround => self.bounds.position.y + self.padding,
        };

        for child in &mut self.children {
            let child_bounds = child.get_bounds();
            let x = match self.vertical_alignment {
                FlexAlign::Start => self.bounds.position.x + self.padding,
                FlexAlign::Center => {
                    self.bounds.position.x + (width - child_bounds.size.width) / 2.0
                }
                FlexAlign::End => {
                    self.bounds.position.x + width - child_bounds.size.width - self.padding
                }
                FlexAlign::SpaceBetween | FlexAlign::SpaceAround => {
                    self.bounds.position.x + self.padding
                }
            };

            child.set_position(queue, device, ComponentPosition { x, y: y_offset });
            y_offset += child_bounds.size.height + self.gap;
        }
    }

    fn resize_bounds(&mut self, width: u32, height: u32) {
        if let Some(bounds) = self.parent_bounds {
            // Update parent bounds with new dimensions
            let parent = Bounds::new(
                bounds.position,
                ComponentSize {
                    width: width as f32,
                    height: height as f32,
                },
            );

            // Recalculate container position based on anchor
            let (anchor_x, anchor_y) = parent.get_anchor_position(self.anchor);
            self.bounds.position.x = anchor_x + self.offset.x;
            self.bounds.position.y = anchor_y + self.offset.y;

            // Store updated parent bounds
            self.parent_bounds = Some(parent);
        }
    }

    pub fn get_bounds_with_padding(&self) -> Bounds {
        // adjust for padding
        let size = ComponentSize {
            width: self.bounds.size.width - self.padding * 2.0,
            height: self.bounds.size.height - self.padding * 2.0,
        };
        let position = ComponentPosition {
            x: self.bounds.position.x + self.padding,
            y: self.bounds.position.y + self.padding,
        };
        Bounds::new(position, size)
    }
}

impl Component for Container {
    fn id(&self) -> Uuid {
        self.id
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn component_type(&self) -> super::core::ComponentType {
        ComponentType::Container
    }

    fn send_event(&self, event: AppEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }

    fn update(&mut self, queue: &wgpu::Queue) {
        // Update background first
        if let Some(ComponentRenderable::Image(img)) = &mut self.renderable {
            img.update(queue);
        }

        // Then update children
        for child in &mut self.children {
            child.update(queue);
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
        self.screen_size = (width, height);
        self.resize_bounds(width, height);

        // First resize all children without layout
        for child in &mut self.children {
            child.resize(wgpu_ctx, width, height);
        }

        // Resize background
        if let Some(ComponentRenderable::Background(bg)) = &mut self.renderable {
            bg.resize(&wgpu_ctx.queue, width, height, self.bounds);
        } else if let Some(ComponentRenderable::Image(img)) = &mut self.renderable {
            img.resize(wgpu_ctx, width, height);
        }

        // Finally do layout after all children have been resized
        self.layout_children(&wgpu_ctx.queue, &wgpu_ctx.device);
    }

    fn set_position(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        position: ComponentPosition,
    ) {
        self.bounds.position = position;

        // Update background position first
        if let Some(ComponentRenderable::Background(bg)) = &mut self.renderable {
            bg.set_position(queue, position, self.bounds);
        } else if let Some(ComponentRenderable::Image(img)) = &mut self.renderable {
            img.set_position(queue, device, position);
        }

        // Then update children layout
        self.layout_children(queue, device);
    }

    fn handle_mouse_click(&mut self, x: f32, y: f32) -> bool {
        for child in self.children.iter_mut().rev() {
            if child.handle_mouse_click(x, y) {
                return true;
            }
        }
        false
    }

    fn add_child(&mut self, child: Box<dyn Component>) {
        if child.component_type() == ComponentType::Container {
            self.children_has_nested_containers = true;
        }
        self.children.push(child);
    }

    fn get_bounds(&self) -> Bounds {
        self.bounds
    }
}
