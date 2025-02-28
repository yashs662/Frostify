use crate::{
    app::AppEvent,
    color::Color,
    constants::TEXTURE_BIND_GROUP_LAYOUT_ENTIRES,
    img_utils::RgbaImg,
    text_renderer::OptionalTextUpdateData,
    ui::layout::{
        Bounds, ComponentOffset, ComponentPosition, ComponentSize, ComponentTransform, Layout,
        Position, Size,
    },
    vertex::Vertex,
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use log::{debug, error, warn};
use uuid::Uuid;
use wgpu::{util::DeviceExt, SamplerDescriptor};

#[derive(Debug, Clone)]
pub struct Component {
    pub id: Uuid,
    pub debug_name: String,
    pub component_type: ComponentType,
    pub transform: ComponentTransform,
    pub layout: Layout,
    children: Vec<Uuid>,
    parent: Option<Uuid>,
    pub metadata: Vec<ComponentMetaData>,
    pub config: Option<ComponentConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    Container,
    Label,
    Image,
    Button,
    Background,
}

#[derive(Debug, Clone)]
pub enum ComponentMetaData {
    ClickEvent(AppEvent),
    VertexBuffer(wgpu::Buffer),
    IndexBuffer(wgpu::Buffer),
    BindGroup(wgpu::BindGroup),
}

#[derive(Debug, Clone)]
pub enum ComponentConfig {
    BackgroundColor(BackgroundColorConfig),
    Label(LabelConfig),
    Image(ImageConfig),
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
pub struct LabelConfig {
    pub text: String,
    pub font_size: f32,
    pub line_height: f32,
    pub color: Color,
}

impl Component {
    pub fn new(id: Uuid, component_type: ComponentType) -> Self {
        Self {
            id,
            debug_name: format!("{:?}:{:?}", component_type, id),
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
        }
    }

    pub fn get_parent_id(&self) -> Option<Uuid> {
        self.parent
    }

    pub fn set_debug_name(&mut self, name: &str) {
        self.debug_name = name.to_string();
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
            ComponentType::Background => {
                let vertex_buffer = self.get_vertex_buffer();
                let index_buffer = self.get_index_buffer();
                let bind_group = self.get_bind_group();

                if vertex_buffer.is_none() || index_buffer.is_none() || bind_group.is_none() {
                    error!(
                        "Vertex buffer, index buffer, or bind group not found for component id: {}, unable to draw",
                        self.id
                    );
                    return;
                }

                let vertex_buffer = vertex_buffer.unwrap();
                let index_buffer = index_buffer.unwrap();
                let bind_group = bind_group.unwrap();

                let indices = self.get_indices();

                render_pass.set_pipeline(&app_pipelines.color_pipeline);
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }
            ComponentType::Label => {
                warn!("Text rendering is done in a separate pass");
            }
            ComponentType::Image => {
                let vertex_buffer = self.get_vertex_buffer();
                let index_buffer = self.get_index_buffer();
                let bind_group = self.get_bind_group();

                if vertex_buffer.is_none() || index_buffer.is_none() || bind_group.is_none() {
                    error!(
                        "Vertex buffer, index buffer, or bind group not found for component id: {}, unable to draw",
                        self.id
                    );
                    return;
                }

                let vertex_buffer = vertex_buffer.unwrap();
                let index_buffer = index_buffer.unwrap();
                let bind_group = bind_group.unwrap();

                let indices = self.get_indices();

                render_pass.set_pipeline(&app_pipelines.texture_pipeline);
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }
            ComponentType::Button | ComponentType::Container => {
                // Containers and buttons are not drawn directly
            }
        }
    }

    pub fn configure(&mut self, config: ComponentConfig, wgpu_ctx: &mut WgpuCtx) {
        self.config = Some(config.clone());

        match config {
            ComponentConfig::BackgroundColor(_) => {
                // Initial vertices with default bounds, will be recalculated on resize
                let vertices = self.calculate_vertices(Bounds::default());
                let indices = self.get_indices();

                // Create buffers
                let vertex_buffer =
                    wgpu_ctx
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(format!("{} Vertex Buffer", self.id).as_str()),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        });

                let index_buffer =
                    wgpu_ctx
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(format!("{} Index Buffer", self.id).as_str()),
                            contents: bytemuck::cast_slice(&indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                // Create an empty bind group for solid colors and gradients
                let bind_group_layout =
                    wgpu_ctx
                        .device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            entries: &[],
                            label: Some(format!("{} Bind Group Layout", self.id).as_str()),
                        });

                let bind_group = wgpu_ctx
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &bind_group_layout,
                        entries: &[],
                        label: Some(format!("{} Bind Group", self.id).as_str()),
                    });

                self.metadata
                    .push(ComponentMetaData::VertexBuffer(vertex_buffer));
                self.metadata
                    .push(ComponentMetaData::IndexBuffer(index_buffer));
                self.metadata.push(ComponentMetaData::BindGroup(bind_group));
            }
            ComponentConfig::Label(text_config) => {
                wgpu_ctx.text_handler.register_text(
                    self.id,
                    text_config.text,
                    text_config.font_size,
                    text_config.line_height,
                    Bounds::default(),
                    text_config.color,
                );
            }
            ComponentConfig::Image(image_config) => {
                let img = RgbaImg::new(&image_config.image_path).unwrap();
                let vertices = self.calculate_vertices(Bounds::default());
                let indices = self.get_indices();

                // Create texture and bind group
                let texture_size = wgpu::Extent3d {
                    width: img.width,
                    height: img.height,
                    depth_or_array_layers: 1,
                };
                let texture = wgpu_ctx.device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: texture_size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });

                // Write the image data to the texture
                wgpu_ctx.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &img.bytes,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * img.width),
                        rows_per_image: Some(img.height),
                    },
                    texture_size,
                );

                let sampler = wgpu_ctx.device.create_sampler(&SamplerDescriptor {
                    address_mode_u: wgpu::AddressMode::ClampToEdge,
                    address_mode_v: wgpu::AddressMode::ClampToEdge,
                    address_mode_w: wgpu::AddressMode::ClampToEdge,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    ..Default::default()
                });
                let bind_group_layout =
                    wgpu_ctx
                        .device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            entries: TEXTURE_BIND_GROUP_LAYOUT_ENTIRES,
                            label: None,
                        });
                let bind_group = wgpu_ctx
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(
                                    &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&sampler),
                            },
                        ],
                        label: None,
                    });
                let vertex_buffer =
                    wgpu_ctx
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        });

                let index_buffer =
                    wgpu_ctx
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: bytemuck::cast_slice(&indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                self.metadata
                    .push(ComponentMetaData::VertexBuffer(vertex_buffer));
                self.metadata
                    .push(ComponentMetaData::IndexBuffer(index_buffer));
                self.metadata.push(ComponentMetaData::BindGroup(bind_group));
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
                ComponentConfig::BackgroundColor(_) | ComponentConfig::Image(_) => {
                    // Convert to NDC space
                    let clip_bounds = self.convert_to_ndc(bounds, screen_size);
                    let vertices = self.calculate_vertices(clip_bounds);

                    // Update vertex buffer
                    if let Some(ComponentMetaData::VertexBuffer(vertex_buffer)) = self
                        .metadata
                        .iter()
                        .find(|m| matches!(m, ComponentMetaData::VertexBuffer(_)))
                    {
                        wgpu_ctx.queue.write_buffer(
                            vertex_buffer,
                            0,
                            bytemuck::cast_slice(&vertices),
                        );
                    }
                }
                ComponentConfig::Label(_) => {
                    let text_computed_bounds = wgpu_ctx.text_handler.measure_text(self.id);
                    let calc_bounds = if let Some(text_size) = text_computed_bounds {
                        if text_size.width == 0.0 || text_size.height == 0.0 {
                            // Initial Layout is not yet computed, wait for next set_position call
                            debug!(
                                "Text bounds not yet computed for component id: {}, waiting for next set_position call",
                                self.id
                            );
                            bounds
                        } else {
                            // center text use the x and y of bounds and the text size
                            let x = bounds.position.x + (bounds.size.width - text_size.width) / 2.0;
                            let y =
                                bounds.position.y + (bounds.size.height - text_size.height) / 2.0;
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
                        self.id,
                        OptionalTextUpdateData::new().with_bounds(calc_bounds),
                    ));
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

    fn get_indices(&self) -> Vec<u16> {
        vec![0, 1, 2, 0, 2, 3]
    }

    fn calculate_vertices(&self, clip_bounds: Bounds) -> Vec<Vertex> {
        let color = match &self.config {
            Some(ComponentConfig::BackgroundColor(bg_config)) => bg_config.color.value(),
            Some(ComponentConfig::Image(_)) => Color::White.value(),
            _ => return Vec::new(),
        };

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
    }

    fn convert_to_ndc(&self, bounds: Bounds, screen_size: ComponentSize) -> Bounds {
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

    fn get_vertex_buffer(&self) -> Option<&wgpu::Buffer> {
        self.get_metadata(|m| match m {
            ComponentMetaData::VertexBuffer(buf) => Some(buf),
            _ => None,
        })
    }

    fn get_index_buffer(&self) -> Option<&wgpu::Buffer> {
        self.get_metadata(|m| match m {
            ComponentMetaData::IndexBuffer(buf) => Some(buf),
            _ => None,
        })
    }

    fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.get_metadata(|m| match m {
            ComponentMetaData::BindGroup(group) => Some(group),
            _ => None,
        })
    }
}
