use crate::{
    app::AppEvent,
    color::Color,
    ui::layout::{
        Bounds, ComponentOffset, ComponentPosition, ComponentSize, ComponentTransform, Layout,
        Position, Size,
    },
    vertex::Vertex,
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use log::warn;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use super::{
    Configurable, Positionable, Renderable, background_color::BackgroundColorComponent,
    background_gradient::BackgroundGradientComponent, image::ImageComponent, text::TextComponent,
};

#[derive(Debug, Clone)]
pub struct Component {
    pub id: Uuid,
    pub debug_name: Option<String>,
    pub component_type: ComponentType,
    pub transform: ComponentTransform,
    pub layout: Layout,
    children: Vec<Uuid>,
    parent: Option<Uuid>,
    pub metadata: Vec<ComponentMetaData>,
    pub config: Option<ComponentConfig>,
    pub cached_vertices: Option<Vec<Vertex>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    Container,
    Text,
    Image,
    BackgroundColor,
    BackgroundGradient,
}

#[derive(Debug, Clone)]
pub enum ComponentMetaData {
    ClickEvent(AppEvent),
    VertexBuffer(wgpu::Buffer),
    IndexBuffer(wgpu::Buffer),
    BindGroup(wgpu::BindGroup),
    EventSender(UnboundedSender<AppEvent>),
    DragEvent(AppEvent), // Add this variant
    ChildComponents(Vec<Component>),
}

#[derive(Debug, Clone)]
pub enum ComponentConfig {
    BackgroundColor(BackgroundColorConfig),
    BackgroundGradient(BackgroundGradientConfig),
    Text(TextConfig),
    Image(ImageConfig),
}

#[derive(Debug, Clone)]
pub struct BackgroundGradientConfig {
    pub start_color: Color,
    pub end_color: Color,
    pub angle: f32, // Angle in degrees
}

#[derive(Debug, Clone)]
pub struct ImageConfig {
    pub image_path: String,
}

#[derive(Debug, Clone)]
pub struct BackgroundColorConfig {
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct TextConfig {
    pub text: String,
    pub font_size: f32,
    pub line_height: f32,
    pub color: Color,
}

impl ComponentConfig {
    pub fn get_text_config(self) -> Option<TextConfig> {
        match self {
            Self::Text(config) => Some(config),
            _ => None,
        }
    }

    pub fn get_image_config(self) -> Option<ImageConfig> {
        match self {
            Self::Image(config) => Some(config),
            _ => None,
        }
    }

    pub fn get_gradient_config(self) -> Option<BackgroundGradientConfig> {
        match self {
            Self::BackgroundGradient(config) => Some(config),
            _ => None,
        }
    }
}

impl Component {
    pub fn new(id: Uuid, component_type: ComponentType) -> Self {
        Self {
            id,
            debug_name: None,
            component_type,
            transform: ComponentTransform {
                size: Size::fill(),
                offset: ComponentOffset { x: 0.0, y: 0.0 },
                position_type: Position::Flex,
                z_index: 0,
            },
            layout: Layout::new(),
            children: Vec::new(),
            parent: None,
            metadata: Vec::new(),
            config: None,
            cached_vertices: None,
        }
    }

    pub fn get_parent_id(&self) -> Option<Uuid> {
        self.parent
    }

    pub fn set_debug_name(&mut self, name: impl Into<String>) {
        self.debug_name = Some(name.into());
    }

    pub fn get_all_children_ids(&self) -> Vec<Uuid> {
        let mut children = Vec::new();
        for child_id in &self.children {
            children.push(*child_id);
        }
        children
    }

    pub fn set_z_index(&mut self, z_index: i32) {
        self.transform.z_index = z_index;
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass, app_pipelines: &mut AppPipelines) {
        if self.config.is_none() {
            return;
        }

        match self.component_type {
            ComponentType::BackgroundColor => {
                BackgroundColorComponent::draw(self, render_pass, app_pipelines);
            }
            ComponentType::BackgroundGradient => {
                BackgroundGradientComponent::draw(self, render_pass, app_pipelines);
            }
            ComponentType::Text => {
                TextComponent::draw(self, render_pass, app_pipelines);
            }
            ComponentType::Image => {
                ImageComponent::draw(self, render_pass, app_pipelines);
            }
            ComponentType::Container => {
                // Containers are not drawn directly
            }
        }
    }

    pub fn configure(&mut self, config: ComponentConfig, wgpu_ctx: &mut WgpuCtx) {
        self.config = Some(config.clone());

        match config {
            ComponentConfig::BackgroundColor(_) => {
                for metadata in BackgroundColorComponent::configure(self, config, wgpu_ctx) {
                    self.metadata.push(metadata);
                }
            }
            ComponentConfig::BackgroundGradient(_) => {
                for metadata in BackgroundGradientComponent::configure(self, config, wgpu_ctx) {
                    self.metadata.push(metadata);
                }
            }
            ComponentConfig::Text(_) => {
                for metadata in TextComponent::configure(self, config, wgpu_ctx) {
                    self.metadata.push(metadata);
                }
            }
            ComponentConfig::Image(_) => {
                for metadata in ImageComponent::configure(self, config, wgpu_ctx) {
                    self.metadata.push(metadata);
                }
            }
        }
    }

    pub fn set_position(
        &mut self,
        wgpu_ctx: &mut WgpuCtx,
        bounds: Bounds,
        screen_size: ComponentSize,
    ) {
        if let Some(config) = &self.config {
            match config {
                ComponentConfig::BackgroundColor(_) => {
                    BackgroundColorComponent::set_position(self, wgpu_ctx, bounds, screen_size);
                }
                ComponentConfig::BackgroundGradient(_) => {
                    BackgroundGradientComponent::set_position(self, wgpu_ctx, bounds, screen_size);
                }
                ComponentConfig::Image(_) => {
                    ImageComponent::set_position(self, wgpu_ctx, bounds, screen_size);
                }
                ComponentConfig::Text(_) => {
                    TextComponent::set_position(self, wgpu_ctx, bounds, screen_size);
                }
            }
        };
    }

    pub fn add_child(&mut self, child_id: Uuid) {
        self.children.push(child_id);
    }

    pub fn set_parent(&mut self, parent_id: Uuid) {
        self.parent = Some(parent_id);
    }

    pub fn get_indices(&self) -> Vec<u16> {
        vec![0, 1, 2, 0, 2, 3]
    }

    pub fn calculate_vertices(
        &mut self,
        clip_bounds: Option<Bounds>,
        custom_color: Option<Color>,
    ) -> Vec<Vertex> {
        let color = if let Some(custom_color) = custom_color {
            custom_color.value()
        } else {
            match &self.config {
                Some(ComponentConfig::BackgroundColor(bg_config)) => bg_config.color.value(),
                Some(ComponentConfig::Image(_)) => Color::White.value(),
                _ => return Vec::new(),
            }
        };

        if let Some(clip_bounds) = clip_bounds {
            // Calculate vertices in clip space
            let top = clip_bounds.position.y;
            let bottom = top - clip_bounds.size.height;
            let left = clip_bounds.position.x;
            let right = left + clip_bounds.size.width;

            vec![
                // Top-left
                Vertex::new([left, top, 0.0], color, [0.0, 0.0]),
                // Top-right
                Vertex::new([right, top, 0.0], color, [1.0, 0.0]),
                // Bottom-right
                Vertex::new([right, bottom, 0.0], color, [1.0, 1.0]),
                // Bottom-left
                Vertex::new([left, bottom, 0.0], color, [0.0, 1.0]),
            ]
        } else if let Some(cached_vertices) = &self.cached_vertices {
            let cached = cached_vertices.clone();
            // replace the color with the custom color
            cached
                .iter()
                .map(|v| Vertex::new(v.position, color, v.tex_coords))
                .collect()
        } else {
            warn!(
                "No clip bounds or cached vertices found for component id: {}, debug name: {}",
                self.id,
                self.debug_name.as_deref().unwrap_or("None")
            );
            Vec::new()
        }
    }

    pub fn convert_to_ndc(&self, bounds: Bounds, screen_size: ComponentSize) -> Bounds {
        // Convert screen coordinates to clip space (NDC)
        // Important: Ensure consistent coordinate system transformation
        let clip_x = (2.0 * bounds.position.x / screen_size.width) - 1.0;
        let clip_y = 1.0 - (2.0 * bounds.position.y / screen_size.height);

        // Convert sizes to NDC scale
        let clip_width = 2.0 * bounds.size.width / screen_size.width;
        let clip_height = 2.0 * bounds.size.height / screen_size.height;

        Bounds {
            position: ComponentPosition {
                x: clip_x,
                y: clip_y,
            },
            size: ComponentSize {
                width: clip_width,
                height: clip_height,
            },
        }
    }

    fn get_metadata<T>(&self, matcher: fn(&ComponentMetaData) -> Option<&T>) -> Option<&T> {
        self.metadata.iter().find_map(matcher)
    }

    pub fn get_vertex_buffer(&self) -> Option<&wgpu::Buffer> {
        self.get_metadata(|m| match m {
            ComponentMetaData::VertexBuffer(buf) => Some(buf),
            _ => None,
        })
    }

    pub fn get_index_buffer(&self) -> Option<&wgpu::Buffer> {
        self.get_metadata(|m| match m {
            ComponentMetaData::IndexBuffer(buf) => Some(buf),
            _ => None,
        })
    }

    pub fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.get_metadata(|m| match m {
            ComponentMetaData::BindGroup(group) => Some(group),
            _ => None,
        })
    }

    pub fn get_event_sender(&self) -> Option<&UnboundedSender<AppEvent>> {
        self.metadata.iter().find_map(|m| match m {
            ComponentMetaData::EventSender(sender) => Some(sender),
            _ => None,
        })
    }

    pub fn get_click_event(&self) -> Option<&AppEvent> {
        self.metadata.iter().find_map(|m| match m {
            ComponentMetaData::ClickEvent(event) => Some(event),
            _ => None,
        })
    }

    pub fn set_click_handler(&mut self, event: AppEvent, event_tx: UnboundedSender<AppEvent>) {
        self.metadata.push(ComponentMetaData::ClickEvent(event));
        self.metadata.push(ComponentMetaData::EventSender(event_tx));
    }

    pub fn is_clickable(&self) -> bool {
        self.metadata
            .iter()
            .any(|m| matches!(m, ComponentMetaData::ClickEvent(_)))
            && self
                .metadata
                .iter()
                .any(|m| matches!(m, ComponentMetaData::EventSender(_)))
    }

    pub fn set_drag_handler(&mut self, event: AppEvent, event_tx: UnboundedSender<AppEvent>) {
        self.metadata.push(ComponentMetaData::DragEvent(event));
        self.metadata.push(ComponentMetaData::EventSender(event_tx));
    }

    pub fn get_drag_event(&self) -> Option<&AppEvent> {
        self.metadata.iter().find_map(|m| match m {
            ComponentMetaData::DragEvent(event) => Some(event),
            _ => None,
        })
    }

    pub fn is_draggable(&self) -> bool {
        self.metadata
            .iter()
            .any(|m| matches!(m, ComponentMetaData::DragEvent(_)))
            && self
                .metadata
                .iter()
                .any(|m| matches!(m, ComponentMetaData::EventSender(_)))
    }
}
