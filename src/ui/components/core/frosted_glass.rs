use crate::{
    constants::UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
    ui::{
        Configurable, Positionable,
        component::{Component, ComponentConfig, ComponentMetaData},
        layout::Bounds,
    },
    wgpu_ctx::WgpuCtx,
};
use wgpu::util::DeviceExt;

pub struct FrostedGlassComponent;

impl Configurable for FrostedGlassComponent {
    fn configure(
        component: &mut Component,
        _config: ComponentConfig,
        wgpu_ctx: &mut WgpuCtx,
    ) -> Vec<ComponentMetaData> {
        let component_data = component.get_render_data(Bounds::default());
        // Create the buffer for component data
        let render_data_buffer =
            wgpu_ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(format!("{} Render Data Buffer", component.id).as_str()),
                    contents: bytemuck::cast_slice(&[component_data]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        // Create an enhanced sampler for the blur operations with anisotropic filtering
        // and mipmap support for higher quality results
        let sampler = wgpu_ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            anisotropy_clamp: 16, // Enable high-quality anisotropic filtering
            compare: None,
            ..Default::default()
        });

        // Create a placeholder texture view until we capture the actual frame
        let placeholder_texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let placeholder_texture_data: [u8; 4] = [255, 255, 255, 255];
        let placeholder_texture = wgpu_ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(format!("{} Placeholder Texture", component.id).as_str()),
            size: placeholder_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload placeholder texture data
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
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            placeholder_texture_size,
        );

        let texture_view = placeholder_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Using the standard unified bind group layout
        let bind_group_layout =
            wgpu_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
                    label: Some(
                        format!("{} Frosted Glass Bind Group Layout", component.id).as_str(),
                    ),
                });

        // Create unified bind group for frosted glass
        let bind_group = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    // Component uniform data (including frosted glass parameters)
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: render_data_buffer.as_entire_binding(),
                    },
                    // Texture view (will be replaced with captured frame)
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
                label: Some(format!("{} Frosted Glass Bind Group", component.id).as_str()),
            });

        vec![
            ComponentMetaData::BindGroup(bind_group),
            ComponentMetaData::RenderDataBuffer(render_data_buffer),
            ComponentMetaData::Sampler(sampler),
        ]
    }
}

impl Positionable for FrostedGlassComponent {
    fn set_position(component: &mut Component, wgpu_ctx: &mut WgpuCtx, bounds: Bounds) {
        if let Some(render_data_buffer) = component.get_render_data_buffer() {
            wgpu_ctx.queue.write_buffer(
                render_data_buffer,
                0,
                bytemuck::cast_slice(&[component.get_render_data(bounds)]),
            );
        }
    }
}
