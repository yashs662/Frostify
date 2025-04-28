use crate::{
    constants::UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
    ui::{
        color::Color,
        ecs::{
            ComponentType, EntityId, GradientColorStop, GradientType, World,
            builders::{EntityBuilder, EntityBuilderProps},
            components::{
                ColorComponent, FrostedGlassComponent, GradientComponent, LayoutComponent,
                RenderDataComponent,
            },
            systems::create_component_buffer_data,
        },
        layout::Layout,
        z_index_manager::ZIndexManager,
    },
    wgpu_ctx::WgpuCtx,
};
use wgpu::util::DeviceExt;

use super::add_common_components;

pub struct BackgroundGradientConfig {
    pub color_stops: Vec<GradientColorStop>,
    pub gradient_type: GradientType,
    pub angle: f32,
    pub center: Option<(f32, f32)>,
    pub radius: Option<f32>,
}

pub struct BackgroundColorConfig {
    pub color: Color,
}

pub struct FrostedGlassConfig {
    pub tint_color: Color,
    pub blur_radius: f32,
    pub opacity: f32,
    pub tint_intensity: f32,
}

pub enum BackgroundType {
    Color(BackgroundColorConfig),
    Gradient(BackgroundGradientConfig),
    FrostedGlass(FrostedGlassConfig),
}

pub struct BackgroundBuilder {
    common: EntityBuilderProps,
    background_type: BackgroundType,
}

impl EntityBuilder for BackgroundBuilder {
    fn common_props(&mut self) -> &mut EntityBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl BackgroundBuilder {
    pub fn with_color(background_color_config: BackgroundColorConfig) -> Self {
        Self {
            common: EntityBuilderProps::default(),
            background_type: BackgroundType::Color(background_color_config),
        }
    }

    pub fn with_linear_gradient(background_gradient_config: BackgroundGradientConfig) -> Self {
        Self {
            common: EntityBuilderProps::default(),
            background_type: BackgroundType::Gradient(background_gradient_config),
        }
    }

    pub fn with_frosted_glass(frosted_glass_config: FrostedGlassConfig) -> Self {
        Self {
            common: EntityBuilderProps::default(),
            background_type: BackgroundType::FrostedGlass(frosted_glass_config),
        }
    }

    pub fn build(
        self,
        world: &mut World,
        wgpu_ctx: &mut WgpuCtx,
        z_index_manager: &mut ZIndexManager,
    ) -> EntityId {
        let component_type = match self.background_type {
            BackgroundType::Color(_) => ComponentType::BackgroundColor,
            BackgroundType::Gradient(_) => ComponentType::BackgroundGradient,
            BackgroundType::FrostedGlass(_) => ComponentType::FrostedGlass,
        };

        let entity_id = world.create_entity(
            self.common
                .debug_name.clone()
                .expect("Debug name is required for all components, tried to create a background color component without it."),
            component_type,
        );

        add_common_components(
            world,
            z_index_manager,
            entity_id,
            &self.common,
            component_type,
        );

        // Add layout component
        let mut layout = Layout::new();
        if let Some(margin) = self.common.margin {
            layout.margin = margin;
        }
        if let Some(padding) = self.common.padding {
            layout.padding = padding;
        }
        world.add_component(entity_id, LayoutComponent { layout });

        match &self.background_type {
            BackgroundType::Color(color) => {
                world.add_component(entity_id, ColorComponent { color: color.color });
            }
            BackgroundType::Gradient(gradient_config) => {
                world.add_component(
                    entity_id,
                    GradientComponent {
                        color_stops: gradient_config.color_stops.clone(),
                        gradient_type: gradient_config.gradient_type,
                        angle: gradient_config.angle,
                        center: gradient_config.center,
                        radius: gradient_config.radius,
                    },
                );
            }
            BackgroundType::FrostedGlass(frosted_glass_config) => {
                world.add_component(
                    entity_id,
                    FrostedGlassComponent {
                        tint_color: frosted_glass_config.tint_color,
                        blur_radius: frosted_glass_config.blur_radius,
                        opacity: frosted_glass_config.opacity,
                        tint_intensity: frosted_glass_config.tint_intensity,
                    },
                );
            }
        }

        // Configure
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

        let texture_view = match self.background_type {
            BackgroundType::Color { .. } | BackgroundType::FrostedGlass { .. } => {
                // Create an empty 1x1 white texture as placeholder for color-only components
                let placeholder_texture_size = wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                };
                let placeholder_texture_data: [u8; 4] = [255, 255, 255, 255]; // White pixel
                let placeholder_texture =
                    wgpu_ctx.device.create_texture(&wgpu::TextureDescriptor {
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

                placeholder_texture.create_view(&wgpu::TextureViewDescriptor::default())
            }
            BackgroundType::Gradient(gradient_config) => {
                let (_, gradient_texture_view) = Color::create_gradient_texture(
                    &wgpu_ctx.device,
                    &wgpu_ctx.queue,
                    gradient_config,
                    1024,
                    1024,
                );

                gradient_texture_view
            }
        };

        // Create Texture Sampler
        let sampler = wgpu_ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
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
