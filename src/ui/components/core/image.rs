use crate::{
    constants::UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
    ui::{
        Configurable, Positionable, Renderable,
        component::{Component, ComponentConfig, ComponentMetaData},
        components::image::ScaleMode,
        img_utils::RgbaImg,
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
        // Extract the image configuration
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

        // Store image dimensions for scaling calculations
        let metadata = ImageMetadata {
            original_width: img.width,
            original_height: img.height,
            scale_mode: image_config.scale_mode,
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

        // Create a sampler that respects the scaling mode
        let filter_mode = match image_config.scale_mode {
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

        let component_data = component.get_render_data(Bounds::default());

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
            ComponentMetaData::ImageMetadata(metadata),
        ]
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ImageMetadata {
    pub original_width: u32,
    pub original_height: u32,
    pub scale_mode: ScaleMode,
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

            // Get the image metadata if available
            let metadata = component.get_image_metadata();

            // Calculate proper positioning based on scale mode
            if let Some(metadata) = metadata {
                let original_width = metadata.original_width as f32;
                let original_height = metadata.original_height as f32;
                let container_width = bounds.size.width;
                let container_height = bounds.size.height;

                // Store the original bounds position for later restoration
                let original_position = [component_data.position[0], component_data.position[1]];

                // Apply scaling calculations based on scale_mode
                match metadata.scale_mode {
                    ScaleMode::Stretch => {
                        // STRETCH - default, use container dimensions directly
                    }
                    ScaleMode::Contain => {
                        // CONTAIN - scale to fit while preserving aspect ratio
                        let original_aspect = original_width / original_height;
                        let container_aspect = container_width / container_height;

                        if original_aspect > container_aspect {
                            // Image is wider than container (relative to height)
                            let new_height = container_width / original_aspect;
                            let y_offset = (container_height - new_height) / 2.0;
                            component_data.size[1] = new_height;
                            component_data.position[1] += y_offset;
                        } else {
                            // Image is taller than container (relative to width)
                            let new_width = container_height * original_aspect;
                            let x_offset = (container_width - new_width) / 2.0;
                            component_data.size[0] = new_width;
                            component_data.position[0] += x_offset;
                        }
                    }
                    ScaleMode::Cover => {
                        // COVER - scale to fill while preserving aspect ratio
                        // but keep original container bounds to ensure clipping
                        let original_aspect = original_width / original_height;
                        let container_aspect = container_width / container_height;

                        // Calculate scaled dimensions that fully cover the container
                        let (scaled_width, scaled_height): (f32, f32);
                        let (x_offset, y_offset): (f32, f32);

                        if original_aspect < container_aspect {
                            // Image is taller than container (relative to width)
                            scaled_width = container_width;
                            scaled_height = container_width / original_aspect;
                            x_offset = 0.0; // No horizontal offset
                            y_offset = (container_height - scaled_height) / 2.0;
                        } else {
                            // Image is wider than container (relative to height)
                            scaled_width = container_height * original_aspect;
                            scaled_height = container_height;
                            x_offset = (container_width - scaled_width) / 2.0;
                            y_offset = 0.0; // No vertical offset
                        }

                        // Store the scaled dimensions for the shader's texture calculations
                        component_data.size = [scaled_width, scaled_height];
                        component_data.position = [
                            original_position[0] + x_offset,
                            original_position[1] + y_offset,
                        ];

                        // Add special flag to indicate clipping should be enforced
                        // We'll send the actual container bounds through an additional structure
                        // that will be used by the shader to clip the content
                    }
                    ScaleMode::Original => {
                        // ORIGINAL - use original image dimensions
                        if original_width < container_width && original_height < container_height {
                            // Center the image in the container
                            let x_offset = (container_width - original_width) / 2.0;
                            let y_offset = (container_height - original_height) / 2.0;
                            component_data.size[0] = original_width;
                            component_data.size[1] = original_height;
                            component_data.position[0] += x_offset;
                            component_data.position[1] += y_offset;
                        } else {
                            // If the image is larger than the container, use contain logic
                            let original_aspect = original_width / original_height;
                            let container_aspect = container_width / container_height;

                            if original_aspect > container_aspect {
                                component_data.size[0] = container_width;
                                component_data.size[1] = container_width / original_aspect;
                                let y_offset = (container_height - component_data.size[1]) / 2.0;
                                component_data.position[1] += y_offset;
                            } else {
                                component_data.size[1] = container_height;
                                component_data.size[0] = container_height * original_aspect;
                                let x_offset = (container_width - component_data.size[0]) / 2.0;
                                component_data.position[0] += x_offset;
                            }
                        }
                    }
                }
            }

            wgpu_ctx.queue.write_buffer(
                render_data_buffer,
                0,
                bytemuck::cast_slice(&[component_data]),
            );
        }
    }
}
