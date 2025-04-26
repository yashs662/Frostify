use crate::{
    constants::UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
    ui::{
        Configurable, Positionable,
        color::Color,
        component::{Component, ComponentConfig, ComponentMetaData},
        layout::Bounds,
    },
    wgpu_ctx::WgpuCtx,
};
use log::error;
use wgpu::util::DeviceExt;

pub struct BackgroundGradientComponent;

impl Configurable for BackgroundGradientComponent {
    fn configure(
        component: &mut Component,
        config: ComponentConfig,
        wgpu_ctx: &mut WgpuCtx,
    ) -> Vec<ComponentMetaData> {
        // Extract the gradient configuration
        let gradient_config = match config.clone().get_gradient_config() {
            Some(config) => config,
            None => {
                error!("Expected gradient config for BackgroundGradientComponent");
                return Vec::new();
            }
        };

        // Create component uniform data with texture mode enabled
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

        // Generate the gradient texture
        let (_, gradient_texture_view) = Color::create_gradient_texture(
            &wgpu_ctx.device,
            &wgpu_ctx.queue,
            gradient_config,
            1024,
            1024,
        );

        // Create a sampler for the gradient texture
        let gradient_sampler = wgpu_ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            anisotropy_clamp: 16, // Enable anisotropic filtering for sharper edges
            ..Default::default()
        });

        // Create bind group with texture
        let bind_group_layout =
            wgpu_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
                    label: Some(
                        format!("{} Gradient Unified Bind Group Layout", component.id).as_str(),
                    ),
                });

        let bind_group = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    // Component uniform with gradient data
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: render_data_buffer.as_entire_binding(),
                    },
                    // Gradient texture view
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&gradient_texture_view),
                    },
                    // Gradient sampler
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&gradient_sampler),
                    },
                ],
                label: Some(format!("{} Gradient Unified Bind Group", component.id).as_str()),
            });

        // Store texture for potential reuse or recreation
        vec![
            ComponentMetaData::BindGroup(bind_group),
            ComponentMetaData::RenderDataBuffer(render_data_buffer),
        ]
    }
}

impl Positionable for BackgroundGradientComponent {
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
