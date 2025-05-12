use wgpu::{SamplerDescriptor, util::DeviceExt};

use crate::{
    constants::UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
    ui::{
        ecs::{
            ComponentType, EntityId, World,
            builders::{EntityBuilder, EntityBuilderProps},
            components::{ImageComponent, LayoutComponent, RenderDataComponent},
        },
        img_utils::RgbaImg,
        layout::Layout,
        z_index_manager::ZIndexManager,
    },
    utils::create_component_buffer_data,
    wgpu_ctx::WgpuCtx,
};

use super::add_common_components;

/// Defines how an image should be scaled to fit its container
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum ScaleMode {
    /// Stretch the image to fill the entire container (default)
    Stretch,
    /// Maintain aspect ratio, scale to fit while ensuring entire image is visible
    Contain,
    /// Maintain aspect ratio, scale to cover entire container (may crop)
    Cover,
    /// Don't scale the image (use original dimensions), FilterMode::Nearest
    /// is used for pixel-perfect scaling
    Original,
}

impl Default for ScaleMode {
    fn default() -> Self {
        Self::Stretch
    }
}

pub struct ImageBuilder {
    common: EntityBuilderProps,
    file_name: String,
    scale_mode: ScaleMode,
}

impl EntityBuilder for ImageBuilder {
    fn common_props(&mut self) -> &mut EntityBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl ImageBuilder {
    pub fn new(file_name: &str) -> Self {
        Self {
            common: EntityBuilderProps::default(),
            file_name: file_name.to_string(),
            scale_mode: ScaleMode::default(),
        }
    }

    pub fn with_scale_mode(mut self, scale_mode: ScaleMode) -> Self {
        self.scale_mode = scale_mode;
        self
    }

    // TODO: get fit to size working by creating a set_position trait which
    // calls the specific set_position function for the component type, maybe add it to the BoundsComponent

    pub fn build(
        self,
        world: &mut World,
        wgpu_ctx: &mut WgpuCtx,
        z_index_manager: &mut ZIndexManager,
    ) -> EntityId {
        let component_type = ComponentType::Image;

        let entity_id = world.create_entity(
                self.common
                .debug_name.clone()
                .expect("Debug name is required for all components, tried to create an image component without it."),
            component_type
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

        // Configure
        let img_loader = RgbaImg::new(self.file_name.as_str());
        let img = if let Err(img_load_err) = img_loader {
            panic!(
                "Failed to load image file: {}, error: {}",
                self.file_name, img_load_err
            );
        } else {
            img_loader.unwrap()
        };

        world.add_component(
            entity_id,
            ImageComponent {
                image_path: self.file_name.clone(),
                original_width: img.width,
                original_height: img.height,
                scale_mode: self.scale_mode,
            },
        );

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

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create a sampler that respects the scaling mode
        let filter_mode = match self.scale_mode {
            ScaleMode::Original => wgpu::FilterMode::Nearest, // Original - use Nearest for pixel-perfect scaling
            _ => wgpu::FilterMode::Linear, // Other modes - use Linear for smooth scaling
        };

        let sampler = wgpu_ctx.device.create_sampler(&SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter_mode,
            min_filter: filter_mode,
            mipmap_filter: filter_mode,
            ..Default::default()
        });

        let render_data_buffer =
            wgpu_ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(format!("{} Render Data Buffer", entity_id).as_str()),
                    contents: bytemuck::cast_slice(&[create_component_buffer_data(
                        world, entity_id,
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
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    // Sampler
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some(format!("{} Unified Bind Group", entity_id).as_str()),
            });

        // Add render data component with bind group
        world.add_component(
            entity_id,
            RenderDataComponent {
                render_data_buffer: Some(render_data_buffer),
                bind_group: Some(bind_group),
                sampler: Some(sampler),
            },
        );

        entity_id
    }
}
