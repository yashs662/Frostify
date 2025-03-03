use crate::{
    app::AppEvent,
    color::Color,
    constants::ROUNDED_CORNER_SEGMENT_COUNT,
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
    pub children_ids: Vec<Uuid>,
    parent_id: Option<Uuid>,
    pub metadata: Vec<ComponentMetaData>,
    pub config: Option<ComponentConfig>,
    pub cached_indices: Option<Vec<u16>>,
    requires_children_extraction: bool,
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
    DragEvent(AppEvent),
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
    pub file_name: String,
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
                border_radius: 0.0,
            },
            layout: Layout::new(),
            children_ids: Vec::new(),
            parent_id: None,
            metadata: Vec::new(),
            config: None,
            cached_indices: None,
            requires_children_extraction: false,
        }
    }

    pub fn get_parent_id(&self) -> Option<Uuid> {
        self.parent_id
    }

    pub fn requires_to_be_drawn(&self) -> bool {
        !matches!(
            self.component_type,
            ComponentType::Text | ComponentType::Container
        )
    }

    pub fn flag_children_extraction(&mut self) {
        self.requires_children_extraction = true;
    }

    pub fn requires_children_extraction(&self) -> bool {
        self.requires_children_extraction
    }

    pub fn set_debug_name(&mut self, name: impl Into<String>) {
        self.debug_name = Some(name.into());
    }

    pub fn set_border_radius(&mut self, radius: f32) {
        self.transform.border_radius = radius;
    }

    pub fn get_all_children(&self) -> Vec<Uuid> {
        self.children_ids.clone()
    }

    pub fn set_z_index(&mut self, z_index: i32) {
        self.transform.z_index = z_index;
    }

    pub fn draw(&mut self, render_pass: &mut wgpu::RenderPass, app_pipelines: &mut AppPipelines) {
        if self.config.is_some() {
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

    pub fn set_position(&mut self, wgpu_ctx: &mut WgpuCtx, bounds: Bounds) {
        if let Some(config) = &self.config {
            match config {
                ComponentConfig::BackgroundColor(_) => {
                    BackgroundColorComponent::set_position(self, wgpu_ctx, bounds);
                }
                ComponentConfig::BackgroundGradient(_) => {
                    BackgroundGradientComponent::set_position(self, wgpu_ctx, bounds);
                }
                ComponentConfig::Image(_) => {
                    ImageComponent::set_position(self, wgpu_ctx, bounds);
                }
                ComponentConfig::Text(_) => {
                    TextComponent::set_position(self, wgpu_ctx, bounds);
                }
            }
        };
    }

    pub fn add_child(&mut self, child: Component) {
        let mut child = child;
        child.set_parent(self.id);
        if let Some(ComponentMetaData::ChildComponents(existing_children)) = self
            .metadata
            .iter_mut()
            .find(|m| matches!(m, ComponentMetaData::ChildComponents(_)))
        {
            existing_children.push(child);
        } else {
            self.metadata
                .push(ComponentMetaData::ChildComponents(vec![child]));
        }
        self.flag_children_extraction();
    }

    pub fn set_parent(&mut self, parent_id: Uuid) {
        self.parent_id = Some(parent_id);
    }

    pub fn calculate_vertices(
        &mut self,
        clip_bounds: Option<Bounds>,
        custom_color: Option<Color>,
        screen_size: ComponentSize,
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
            let top = clip_bounds.position.y;
            let bottom = top - clip_bounds.size.height;
            let left = clip_bounds.position.x;
            let right = left + clip_bounds.size.width;

            if self.transform.border_radius > 0.0 {
                let radius = self
                    .transform
                    .border_radius
                    .min((clip_bounds.size.width * screen_size.width) / 2.0)
                    .min((clip_bounds.size.height * screen_size.height) / 2.0);

                // convert radius to ndc
                let radius = radius / screen_size.width;

                let top_left_arc_center = [left + radius, top - radius];
                let top_right_arc_center = [right - radius, top - radius];
                let bottom_right_arc_center = [right - radius, bottom + radius];
                let bottom_left_arc_center = [left + radius, bottom + radius];

                let top_left_arc_start = [left, top - radius];
                let top_right_arc_start = [right - radius, top];
                let bottom_right_arc_start = [right, bottom - radius];
                let bottom_left_arc_start = [left + radius, bottom];

                let top_left_arc_end = [left + radius, top];
                let top_right_arc_end = [right, top - radius];
                let bottom_right_arc_end = [right - radius, bottom];
                let bottom_left_arc_end = [left, bottom - radius];

                // all arcs are constructed going clockwise
                let mut vertices = vec![];

                // Top-left arc
                // Center vertex first
                vertices.push(Vertex::new(
                    [top_left_arc_center[0], top_left_arc_center[1], 0.0],
                    color,
                    [0.0, 0.0],
                ));

                for i in 0..=ROUNDED_CORNER_SEGMENT_COUNT {
                    let t = i as f32 / ROUNDED_CORNER_SEGMENT_COUNT as f32;
                    let angle = std::f32::consts::PI / 2.0 + (t * std::f32::consts::PI / 2.0);
                    let x = top_left_arc_center[0] + radius * angle.cos();
                    let y = top_left_arc_center[1] + radius * angle.sin();
                    vertices.push(Vertex::new([x, y, 0.0], color, [0.0, 0.0]));
                }

                // Top edge is a rectangle made up of the points top left arc end, top right arc start,
                // top left arc center, top right arc center
                vertices.push(Vertex::new(
                    [top_left_arc_end[0], top_left_arc_end[1], 0.0],
                    color,
                    [0.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [top_right_arc_start[0], top_right_arc_start[1], 0.0],
                    color,
                    [1.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [top_left_arc_center[0], top_left_arc_center[1], 0.0],
                    color,
                    [0.0, 1.0],
                ));
                vertices.push(Vertex::new(
                    [top_right_arc_center[0], top_right_arc_center[1], 0.0],
                    color,
                    [1.0, 1.0],
                ));

                // Top-right arc
                // Center vertex first
                vertices.push(Vertex::new(
                    [top_right_arc_center[0], top_right_arc_center[1], 0.0],
                    color,
                    [0.0, 0.0],
                ));

                for i in 0..=ROUNDED_CORNER_SEGMENT_COUNT {
                    let t = i as f32 / ROUNDED_CORNER_SEGMENT_COUNT as f32;
                    let angle = t * std::f32::consts::PI / 2.0;
                    let x = top_right_arc_center[0] + radius * angle.cos();
                    let y = top_right_arc_center[1] + radius * angle.sin();
                    vertices.push(Vertex::new([x, y, 0.0], color, [0.0, 0.0]));
                }

                // Right edge is a rectangle made up of the points top right arc end, bottom right arc start,
                // top right arc center, bottom right arc center
                vertices.push(Vertex::new(
                    [top_right_arc_center[0], top_right_arc_center[1], 0.0],
                    color,
                    [0.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [top_right_arc_end[0], top_right_arc_end[1], 0.0],
                    color,
                    [1.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_right_arc_center[0], bottom_right_arc_center[1], 0.0],
                    color,
                    [0.0, 1.0],
                ));
                vertices.push(Vertex::new(
                    [
                        bottom_right_arc_start[0],
                        bottom_right_arc_start[1] + radius * 2.0,
                        0.0,
                    ],
                    color,
                    [1.0, 1.0],
                ));

                // Bottom-right arc
                // Center vertex first
                vertices.push(Vertex::new(
                    [bottom_right_arc_center[0], bottom_right_arc_center[1], 0.0],
                    color,
                    [0.0, 0.0],
                ));

                for i in 0..=ROUNDED_CORNER_SEGMENT_COUNT {
                    let t = i as f32 / ROUNDED_CORNER_SEGMENT_COUNT as f32;
                    let angle = -std::f32::consts::PI / 2.0 + (t * std::f32::consts::PI / 2.0);
                    let x = bottom_right_arc_center[0] + radius * angle.cos();
                    let y = bottom_right_arc_center[1] + radius * angle.sin();
                    vertices.push(Vertex::new([x, y, 0.0], color, [0.0, 0.0]));
                }

                // Bottom edge is a rectangle made up of the points bottom right arc end, bottom left arc start,
                // bottom right arc center, bottom left arc center
                vertices.push(Vertex::new(
                    [bottom_right_arc_end[0], bottom_right_arc_end[1], 0.0],
                    color,
                    [0.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_left_arc_start[0], bottom_left_arc_start[1], 0.0],
                    color,
                    [1.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_right_arc_center[0], bottom_right_arc_center[1], 0.0],
                    color,
                    [0.0, 1.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_left_arc_center[0], bottom_left_arc_center[1], 0.0],
                    color,
                    [1.0, 1.0],
                ));

                // Bottom-left arc
                // Center vertex first
                vertices.push(Vertex::new(
                    [bottom_left_arc_center[0], bottom_left_arc_center[1], 0.0],
                    color,
                    [0.0, 0.0],
                ));

                for i in 0..=ROUNDED_CORNER_SEGMENT_COUNT {
                    let t = i as f32 / ROUNDED_CORNER_SEGMENT_COUNT as f32;
                    let angle = (t * std::f32::consts::PI / 2.0) + std::f32::consts::PI;
                    let x = bottom_left_arc_center[0] + radius * angle.cos();
                    let y = bottom_left_arc_center[1] + radius * angle.sin();
                    vertices.push(Vertex::new([x, y, 0.0], color, [0.0, 0.0]));
                }

                // Left edge is a rectangle made up of the points bottom left arc end, top left arc start,
                // bottom left arc center, top left arc center
                vertices.push(Vertex::new(
                    [top_left_arc_start[0], top_left_arc_start[1], 0.0],
                    color,
                    [0.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [top_left_arc_center[0], top_left_arc_center[1], 0.0],
                    color,
                    [1.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [
                        bottom_left_arc_end[0],
                        bottom_left_arc_end[1] + radius * 2.0,
                        0.0,
                    ],
                    color,
                    [0.0, 1.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_left_arc_center[0], bottom_left_arc_center[1], 0.0],
                    color,
                    [1.0, 1.0],
                ));

                // Center rectangle
                vertices.push(Vertex::new(
                    [top_left_arc_center[0], top_left_arc_center[1], 0.0],
                    color,
                    [0.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [top_right_arc_center[0], top_right_arc_center[1], 0.0],
                    color,
                    [1.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_right_arc_center[0], bottom_right_arc_center[1], 0.0],
                    color,
                    [1.0, 1.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_left_arc_center[0], bottom_left_arc_center[1], 0.0],
                    color,
                    [0.0, 1.0],
                ));

                vertices
            } else {
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
            }
        } else {
            warn!(
                "No clip bounds or cached vertices found for component id: {}, debug name: {}",
                self.id,
                self.debug_name.as_deref().unwrap_or("None")
            );
            Vec::new()
        }
    }

    pub fn get_indices(&mut self) -> Vec<u16> {
        if let Some(cached_indices) = &self.cached_indices {
            return cached_indices.clone();
        }

        let indices = if self.transform.border_radius > 0.0 {
            let mut indices = vec![];
            let segments = ROUNDED_CORNER_SEGMENT_COUNT as u16;

            // Helper function to create arc indices
            let create_arc_indices = |base_idx: u16| {
                let mut arc_indices = vec![];
                for i in 0..segments {
                    arc_indices.extend_from_slice(&[
                        base_idx,         // Center vertex
                        base_idx + 1 + i, // Current vertex
                        base_idx + 2 + i, // Next vertex
                    ]);
                }
                arc_indices
            };

            // Calculate starting indices for each component
            let top_left_start = 0;
            let top_edge_start = top_left_start + segments + 2; // +2 for center and last vertex
            let top_right_start = top_edge_start + 4;
            let right_edge_start = top_right_start + segments + 2;
            let bottom_right_start = right_edge_start + 4;
            let bottom_edge_start = bottom_right_start + segments + 2;
            let bottom_left_start = bottom_edge_start + 4;
            let left_edge_start = bottom_left_start + segments + 2;
            let center_start = left_edge_start + 4;

            // Add indices for each arc
            indices.extend(create_arc_indices(top_left_start));
            indices.extend(create_arc_indices(top_right_start));
            indices.extend(create_arc_indices(bottom_right_start));
            indices.extend(create_arc_indices(bottom_left_start));

            // Add indices for edges (rectangles)
            let edge_indices =
                |start: u16| vec![start, start + 1, start + 2, start + 1, start + 3, start + 2];

            indices.extend(edge_indices(top_edge_start));
            indices.extend(edge_indices(right_edge_start));
            indices.extend(edge_indices(bottom_edge_start));
            indices.extend(edge_indices(left_edge_start));

            // Add indices for center rectangle
            indices.extend_from_slice(&[
                center_start,
                center_start + 1,
                center_start + 2,
                center_start,
                center_start + 2,
                center_start + 3,
            ]);

            indices
        } else {
            vec![0, 1, 2, 0, 2, 3]
        };

        self.cached_indices = Some(indices.clone());
        indices
    }

    pub fn convert_to_ndc(&self, bounds: Bounds, screen_size: ComponentSize) -> Bounds {
        let clip_x = (2.0 * bounds.position.x / screen_size.width) - 1.0;
        let clip_y = 1.0 - (2.0 * bounds.position.y / screen_size.height);
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

    pub fn get_children_from_metadata(&self) -> Option<&Vec<Component>> {
        self.metadata.iter().find_map(|m| match m {
            ComponentMetaData::ChildComponents(children) => Some(children),
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
