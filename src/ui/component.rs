use crate::{
    app::AppEvent,
    color::Color,
    constants::ROUNDED_CORNER_SEGMENT_COUNT,
    ui::{
        Configurable, Positionable, Renderable,
        components::core::{
            background_color::BackgroundColorComponent,
            background_gradient::BackgroundGradientComponent, image::ImageComponent,
            text::TextComponent,
        },
        layout::{
            Bounds, ComponentOffset, ComponentPosition, ComponentSize, ComponentTransform, Layout,
            Position, Size,
        },
    },
    vertex::Vertex,
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use log::warn;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use super::layout::BorderRadius;

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
    screen_size: ComponentSize,
    requires_children_extraction: bool,
    is_clickable: bool,
    is_draggable: bool,
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
    RenderDataBuffer(wgpu::Buffer),
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
    pub color_stops: Vec<GradientColorStop>,
    pub angle: f32, // Angle in degrees
}

#[derive(Debug, Clone)]
pub struct GradientColorStop {
    pub color: Color,
    pub position: f32, // 0.0 to 1.0 representing the position along the gradient line
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ComponentBufferData {
    pub color: [f32; 4],
    pub location: [f32; 2],
    pub size: [f32; 2],
    pub border_radius: [f32; 4],
    pub screen_size: [f32; 2],
    pub _padding: [f32; 2],
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
                border_radius: BorderRadius::zero(),
            },
            layout: Layout::new(),
            children_ids: Vec::new(),
            parent_id: None,
            metadata: Vec::new(),
            config: None,
            cached_indices: None,
            screen_size: ComponentSize::default(),
            requires_children_extraction: false,
            is_clickable: false,
            is_draggable: false,
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

    pub fn set_border_radius(&mut self, radius: BorderRadius) {
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

    pub fn set_position(
        &mut self,
        wgpu_ctx: &mut WgpuCtx,
        bounds: Bounds,
        screen_size: ComponentSize,
    ) {
        self.screen_size = screen_size;
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
        screen_size: ComponentSize,
    ) -> Vec<Vertex> {
        if let Some(clip_bounds) = clip_bounds {
            let top = clip_bounds.position.y;
            let bottom = top - clip_bounds.size.height;
            let left = clip_bounds.position.x;
            let right = left + clip_bounds.size.width;

            if self.transform.border_radius.has_any_radius() {
                // Calculate the physical pixel radius
                let physical_radius = self.transform.border_radius;

                // Calculate the aspect ratio of the component
                let width_ratio = clip_bounds.size.width / screen_size.width;
                let height_ratio = clip_bounds.size.height / screen_size.height;

                // Calculate max allowed radius (half of smallest dimension)
                let max_radius_w = clip_bounds.size.width / 2.0;
                let max_radius_h = clip_bounds.size.height / 2.0;
                let max_radius = max_radius_w.min(max_radius_h);

                // Determine appropriate radius in NDC space, preserving aspect ratio
                let adjusted_radius = |radius: f32| {
                    (radius * width_ratio)
                        .min(radius * height_ratio)
                        .min(max_radius)
                };

                let tl_radius = adjusted_radius(physical_radius.top_left);
                let tr_radius = adjusted_radius(physical_radius.top_right);
                let br_radius = adjusted_radius(physical_radius.bottom_right);
                let bl_radius = adjusted_radius(physical_radius.bottom_left);

                let top_left_arc_center = [left + tl_radius, top - tl_radius];
                let top_right_arc_center = [right - tr_radius, top - tr_radius];
                let bottom_right_arc_center = [right - br_radius, bottom + br_radius];
                let bottom_left_arc_center = [left + bl_radius, bottom + bl_radius];

                let top_left_arc_start = [left, top - tl_radius];
                let top_right_arc_start = [right - tr_radius, top];
                let bottom_right_arc_start = [right, bottom - br_radius];
                let bottom_left_arc_start = [left + bl_radius, bottom];

                let top_left_arc_end = [left + tl_radius, top];
                let top_right_arc_end = [right, top - tr_radius];
                let bottom_right_arc_end = [right - br_radius, bottom];
                let bottom_left_arc_end = [left, bottom - bl_radius];

                // all arcs are constructed going clockwise
                let mut vertices = vec![];

                // Top-left arc
                // Center vertex first
                vertices.push(Vertex::new(
                    [top_left_arc_center[0], top_left_arc_center[1], 0.0],
                    [0.0, 0.0],
                ));

                for i in 0..=ROUNDED_CORNER_SEGMENT_COUNT {
                    let t = i as f32 / ROUNDED_CORNER_SEGMENT_COUNT as f32;
                    let angle = std::f32::consts::PI / 2.0 + (t * std::f32::consts::PI / 2.0);
                    let x = top_left_arc_center[0] + tl_radius * angle.cos();
                    let y = top_left_arc_center[1] + tl_radius * angle.sin();
                    vertices.push(Vertex::new([x, y, 0.0], [0.0, 0.0]));
                }

                // Top edge is a rectangle made up of the points top left arc end, top right arc start,
                // top left arc center, top right arc center
                vertices.push(Vertex::new(
                    [top_left_arc_end[0], top_left_arc_end[1], 0.0],
                    [0.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [top_right_arc_start[0], top_right_arc_start[1], 0.0],
                    [1.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [top_left_arc_center[0], top_left_arc_center[1], 0.0],
                    [0.0, 1.0],
                ));
                vertices.push(Vertex::new(
                    [top_right_arc_center[0], top_right_arc_center[1], 0.0],
                    [1.0, 1.0],
                ));

                // Top-right arc
                // Center vertex first
                vertices.push(Vertex::new(
                    [top_right_arc_center[0], top_right_arc_center[1], 0.0],
                    [0.0, 0.0],
                ));

                for i in 0..=ROUNDED_CORNER_SEGMENT_COUNT {
                    let t = i as f32 / ROUNDED_CORNER_SEGMENT_COUNT as f32;
                    let angle = t * std::f32::consts::PI / 2.0;
                    let x = top_right_arc_center[0] + tr_radius * angle.cos();
                    let y = top_right_arc_center[1] + tr_radius * angle.sin();
                    vertices.push(Vertex::new([x, y, 0.0], [0.0, 0.0]));
                }

                // Right edge is a rectangle made up of the points top right arc end, bottom right arc start,
                // top right arc center, bottom right arc center
                vertices.push(Vertex::new(
                    [top_right_arc_center[0], top_right_arc_center[1], 0.0],
                    [0.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [top_right_arc_end[0], top_right_arc_end[1], 0.0],
                    [1.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_right_arc_center[0], bottom_right_arc_center[1], 0.0],
                    [0.0, 1.0],
                ));
                vertices.push(Vertex::new(
                    [
                        bottom_right_arc_start[0],
                        bottom_right_arc_start[1] + br_radius * 2.0,
                        0.0,
                    ],
                    [1.0, 1.0],
                ));

                // Bottom-right arc
                // Center vertex first
                vertices.push(Vertex::new(
                    [bottom_right_arc_center[0], bottom_right_arc_center[1], 0.0],
                    [0.0, 0.0],
                ));

                for i in 0..=ROUNDED_CORNER_SEGMENT_COUNT {
                    let t = i as f32 / ROUNDED_CORNER_SEGMENT_COUNT as f32;
                    let angle = -std::f32::consts::PI / 2.0 + (t * std::f32::consts::PI / 2.0);
                    let x = bottom_right_arc_center[0] + br_radius * angle.cos();
                    let y = bottom_right_arc_center[1] + br_radius * angle.sin();
                    vertices.push(Vertex::new([x, y, 0.0], [0.0, 0.0]));
                }

                // Bottom edge is a rectangle made up of the points bottom right arc end, bottom left arc start,
                // bottom right arc center, bottom left arc center
                vertices.push(Vertex::new(
                    [bottom_right_arc_end[0], bottom_right_arc_end[1], 0.0],
                    [0.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_left_arc_start[0], bottom_left_arc_start[1], 0.0],
                    [1.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_right_arc_center[0], bottom_right_arc_center[1], 0.0],
                    [0.0, 1.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_left_arc_center[0], bottom_left_arc_center[1], 0.0],
                    [1.0, 1.0],
                ));

                // Bottom-left arc
                // Center vertex first
                vertices.push(Vertex::new(
                    [bottom_left_arc_center[0], bottom_left_arc_center[1], 0.0],
                    [0.0, 0.0],
                ));

                for i in 0..=ROUNDED_CORNER_SEGMENT_COUNT {
                    let t = i as f32 / ROUNDED_CORNER_SEGMENT_COUNT as f32;
                    let angle = (t * std::f32::consts::PI / 2.0) + std::f32::consts::PI;
                    let x = bottom_left_arc_center[0] + bl_radius * angle.cos();
                    let y = bottom_left_arc_center[1] + bl_radius * angle.sin();
                    vertices.push(Vertex::new([x, y, 0.0], [0.0, 0.0]));
                }

                // Left edge is a rectangle made up of the points bottom left arc end, top left arc start,
                // bottom left arc center, top left arc center
                vertices.push(Vertex::new(
                    [top_left_arc_start[0], top_left_arc_start[1], 0.0],
                    [0.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [top_left_arc_center[0], top_left_arc_center[1], 0.0],
                    [1.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [
                        bottom_left_arc_end[0],
                        bottom_left_arc_end[1] + bl_radius * 2.0,
                        0.0,
                    ],
                    [0.0, 1.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_left_arc_center[0], bottom_left_arc_center[1], 0.0],
                    [1.0, 1.0],
                ));

                // Center rectangle
                vertices.push(Vertex::new(
                    [top_left_arc_center[0], top_left_arc_center[1], 0.0],
                    [0.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [top_right_arc_center[0], top_right_arc_center[1], 0.0],
                    [1.0, 0.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_right_arc_center[0], bottom_right_arc_center[1], 0.0],
                    [1.0, 1.0],
                ));
                vertices.push(Vertex::new(
                    [bottom_left_arc_center[0], bottom_left_arc_center[1], 0.0],
                    [0.0, 1.0],
                ));

                vertices
            } else {
                vec![
                    // Top-left
                    Vertex::new([left, top, 0.0], [0.0, 0.0]),
                    // Top-right
                    Vertex::new([right, top, 0.0], [1.0, 0.0]),
                    // Bottom-right
                    Vertex::new([right, bottom, 0.0], [1.0, 1.0]),
                    // Bottom-left
                    Vertex::new([left, bottom, 0.0], [0.0, 1.0]),
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

        let indices = if self.transform.border_radius.has_any_radius() {
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

    pub fn convert_to_ndc(bounds: Bounds, screen_size: ComponentSize) -> Bounds {
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

    pub fn get_render_data_buffer(&self) -> Option<&wgpu::Buffer> {
        self.get_metadata(|m| match m {
            ComponentMetaData::RenderDataBuffer(buf) => Some(buf),
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

    pub fn set_click_event(&mut self, event: AppEvent) {
        self.metadata.push(ComponentMetaData::ClickEvent(event));
        if self.get_event_sender().is_some() {
            self.is_clickable = true;
        }
    }

    pub fn is_clickable(&self) -> bool {
        self.is_clickable
    }

    pub fn set_drag_event(&mut self, event: AppEvent) {
        self.metadata.push(ComponentMetaData::DragEvent(event));
        if self.get_event_sender().is_some() {
            self.is_draggable = true;
        }
    }

    pub fn set_event_sender(&mut self, sender: UnboundedSender<AppEvent>) {
        self.metadata.push(ComponentMetaData::EventSender(sender));
        if self.get_click_event().is_some() {
            self.is_clickable = true;
        }
        if self.get_drag_event().is_some() {
            self.is_draggable = true;
        }
    }

    pub fn get_drag_event(&self) -> Option<&AppEvent> {
        self.metadata.iter().find_map(|m| match m {
            ComponentMetaData::DragEvent(event) => Some(event),
            _ => None,
        })
    }

    pub fn is_draggable(&self) -> bool {
        self.is_draggable
    }

    pub fn get_render_data(&self, bounds: Bounds) -> ComponentBufferData {
        let default_color = [1.0, 0.0, 1.0, 1.0];
        let location = [bounds.position.x, bounds.position.y];
        let size = [bounds.size.width, bounds.size.height];
        let color = match &self.config {
            Some(ComponentConfig::BackgroundColor(BackgroundColorConfig { color })) => {
                color.value()
            }
            _ => default_color,
        };
        let border_radius = self.transform.border_radius.values();

        ComponentBufferData {
            color,
            location,
            size,
            border_radius,
            screen_size: [self.screen_size.width, self.screen_size.height],
            _padding: [0.0, 0.0],
        }
    }
}
