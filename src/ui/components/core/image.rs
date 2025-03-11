use crate::{
    constants::UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
    img_utils::RgbaImg,
    ui::{
        Configurable, Positionable, Renderable,
        component::{Component, ComponentBufferData, ComponentConfig, ComponentMetaData},
        layout::Bounds,
    },
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use log::error;
use wgpu::{SamplerDescriptor, util::DeviceExt};

pub struct ImageComponent;

impl Configurable for ImageComponent {
    fn configure(
        component: &mut Component,
        config: ComponentConfig,
        wgpu_ctx: &mut WgpuCtx,
    ) -> Vec<ComponentMetaData> {
        // we know config is of type ComponentConfig::Image
        let image_config = config.get_image_config().unwrap();
        let img_loader = RgbaImg::new(&image_config.file_name);
        let img = if let Err(img_load_err) = img_loader {
            error!(
                "Failed to load image file: {}, error: {}",
                image_config.file_name, img_load_err
            );
            return vec![];
        } else {
            img_loader.unwrap()
        };

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

        // Create the render data buffer with component uniform data
        let component_data = ComponentBufferData {
            color: [1.0, 1.0, 1.0, 1.0], // White color to preserve original texture colors
            location: [0.0, 0.0],        // Will be updated in set_position
            size: [0.0, 0.0],            // Will be updated in set_position
            border_radius: component.transform.border_radius.values(),
            screen_size: [
                wgpu_ctx.surface_config.width as f32,
                wgpu_ctx.surface_config.height as f32,
            ],
            use_texture: 1, // Enable texture sampling
            _padding: [0.0],
        };

        let render_data_buffer =
            wgpu_ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(format!("{} Render Data Buffer", component.id).as_str()),
                    contents: bytemuck::cast_slice(&[component_data]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        // Create unified bind group with render data buffer, texture, and sampler
        let bind_group_layout =
            wgpu_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
                    label: None,
                });

        let bind_group = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: render_data_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(
                            &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: None,
            });

        vec![
            ComponentMetaData::RenderDataBuffer(render_data_buffer),
            ComponentMetaData::BindGroup(bind_group),
        ]
    }
}

impl Renderable for ImageComponent {
    fn draw(
        component: &mut Component,
        render_pass: &mut wgpu::RenderPass,
        app_pipelines: &mut AppPipelines,
    ) {
        let bind_group = component.get_bind_group();

        if bind_group.is_none() {
            error!(
                "Bind group not found for image component id: {}, unable to draw",
                component.id
            );
            return;
        }

        // Use the color pipeline with our bind group
        render_pass.set_pipeline(&app_pipelines.unified_pipeline);
        render_pass.set_bind_group(0, bind_group.unwrap(), &[]);

        // Draw full-screen triangle with the shader handling clipping
        render_pass.draw(0..3, 0..1);
    }
}

impl Positionable for ImageComponent {
    fn set_position(component: &mut Component, wgpu_ctx: &mut WgpuCtx, bounds: Bounds) {
        if let Some(render_data_buffer) = component.get_render_data_buffer() {
            let mut component_data = component.get_render_data(bounds);

            // Make sure texture flag is set
            component_data.use_texture = 1;

            wgpu_ctx.queue.write_buffer(
                render_data_buffer,
                0,
                bytemuck::cast_slice(&[component_data]),
            );
        }
    }
}
