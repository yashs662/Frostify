use wgpu::util::DeviceExt;

use crate::{
    constants::UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
    ui::{
        color::Color,
        ecs::{
            ComponentType, EntityId, World,
            builders::{EntityBuilder, EntityBuilderProps, add_common_components},
            components::{LayoutComponent, RenderDataComponent, TextComponent},
        },
        layout::{Bounds, Layout},
        z_index_manager::ZIndexManager,
    },
    utils::create_entity_buffer_data,
    wgpu_ctx::WgpuCtx,
};

pub struct TextConfig {
    pub text: String,
    pub font_size: f32,
    pub line_height_multiplier: f32,
    pub color: Color,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_size: 16.0,
            line_height_multiplier: 1.5,
            color: Color::Black,
        }
    }
}

pub struct TextBuilder {
    common: EntityBuilderProps,
    config: TextConfig,
}

impl EntityBuilder for TextBuilder {
    fn common_props(&mut self) -> &mut EntityBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl TextBuilder {
    pub fn new() -> Self {
        Self {
            common: EntityBuilderProps::default(),
            config: TextConfig::default(),
        }
    }

    pub fn with_text<S: Into<String>>(mut self, text: S) -> Self {
        self.config.text = text.into();
        self
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.config.font_size = font_size;
        self
    }

    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.config.line_height_multiplier = line_height;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.config.color = color;
        self
    }

    pub fn build(
        self,
        world: &mut World,
        wgpu_ctx: &mut WgpuCtx,
        z_index_manager: &mut ZIndexManager,
    ) -> EntityId {
        let component_type = ComponentType::Text;

        let entity_id = world.create_entity(
            self.common
                .debug_name.clone()
                .expect("Debug name is required for all components, tried to create a text component without it."),
            component_type,
        );

        add_common_components(world, z_index_manager, entity_id, &self.common);

        // Add layout component
        let mut layout = Layout::new();
        if let Some(margin) = self.common.margin {
            layout.margin = margin;
        }
        if let Some(padding) = self.common.padding {
            layout.padding = padding;
        }
        world.add_component(entity_id, LayoutComponent { layout });

        // Add text component
        // TODO: Make Fit to size work again
        world.add_component(
            entity_id,
            TextComponent {
                text: self.config.text.clone(),
                font_size: self.config.font_size,
                line_height_multiplier: self.config.line_height_multiplier,
                color: self.config.color,
            },
        );

        // Configure text handler
        wgpu_ctx.text_handler.register_text(
            entity_id,
            self.config.text.clone(),
            self.config.font_size,
            self.config.line_height_multiplier,
            Bounds::default(),
            self.config.color,
        );

        // Create texture and bind group
        let placeholder_texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let placeholder_texture_data: [u8; 4] = [0, 0, 0, 0]; // Transparent pixel
        let placeholder_texture = wgpu_ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(format!("{} Placeholder Texture", entity_id).as_str()),
            size: placeholder_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload white pixel to placeholder texture
        wgpu_ctx.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &placeholder_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &placeholder_texture_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4), // 4 bytes per pixel
                rows_per_image: Some(1),
            },
            placeholder_texture_size,
        );

        let placeholder_texture_view =
            placeholder_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let render_data_buffer =
            wgpu_ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(format!("{} Render Data Buffer", entity_id).as_str()),
                    contents: bytemuck::cast_slice(&[create_entity_buffer_data(
                        &world.components,
                        entity_id,
                    )]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        // Create unified bind group layout compatible with the shader
        let bind_group_layout =
            wgpu_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
                    label: Some(format!("{} Unified Bind Group Layout", entity_id).as_str()),
                });

        // Create bind group with all required resources
        let bind_group = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    // Component uniform data
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: render_data_buffer.as_entire_binding(),
                    },
                    // Texture view
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&placeholder_texture_view),
                    },
                    // Sampler
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(
                            wgpu_ctx.text_handler.get_sampler(),
                        ),
                    },
                ],
                label: Some(format!("{} Unified Bind Group", entity_id).as_str()),
            });

        // Create initial RenderDataComponent - will be updated after layout is computed
        // For now, create a placeholder that will be properly set up during layout sync
        world.add_component(
            entity_id,
            RenderDataComponent {
                render_data_buffer: Some(render_data_buffer),
                bind_group: Some(bind_group),
                sampler: Some(wgpu_ctx.text_handler.get_sampler().clone()),
                vertex_buffer: None, // Will be created during layout sync
                index_buffer: None,  // Will be created during layout sync
            },
        );

        entity_id
    }
}
