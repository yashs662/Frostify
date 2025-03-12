use crate::{
    color::Color,
    constants::UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
    ui::{
        Configurable, Positionable, Renderable,
        component::{Component, ComponentConfig, ComponentMetaData},
        layout::Bounds,
    },
    wgpu_ctx::{AppPipelines, WgpuCtx},
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
        let mut component_data = component.get_render_data(Bounds::default());
        component_data.use_texture = 1; // Enable texture mode for the shader

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
            gradient_config.color_stops,
            gradient_config.gradient_type,
            gradient_config.angle,
            gradient_config.center,
            gradient_config.radius,
            512, // Width of texture
            512, // Height of texture
        );

        // Create a sampler for the gradient texture
        let gradient_sampler = wgpu_ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
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

impl Renderable for BackgroundGradientComponent {
    fn draw(
        component: &mut Component,
        render_pass: &mut wgpu::RenderPass,
        app_pipelines: &mut AppPipelines,
    ) {
        let bind_group = component.get_bind_group();

        if bind_group.is_none() {
            error!(
                "Required resources not found for gradient component id: {}, unable to draw",
                component.id
            );
            return;
        }

        let bind_group = bind_group.unwrap();

        render_pass.set_pipeline(&app_pipelines.unified_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        // Draw a single triangle that covers the whole screen
        render_pass.draw(0..3, 0..1);
    }
}

impl Positionable for BackgroundGradientComponent {
    fn set_position(component: &mut Component, wgpu_ctx: &mut WgpuCtx, bounds: Bounds) {
        let mut component_data = component.get_render_data(bounds);

        // Ensure texture mode is enabled
        component_data.use_texture = 1;

        if let Some(render_data_buffer) = component.get_render_data_buffer() {
            wgpu_ctx.queue.write_buffer(
                render_data_buffer,
                0,
                bytemuck::cast_slice(&[component_data]),
            );
        }
    }
}
