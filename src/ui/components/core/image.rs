use crate::{
    constants::TEXTURE_BIND_GROUP_LAYOUT_ENTIRES,
    img_utils::RgbaImg,
    ui::{
        components::core::{Configurable, component::ComponentMetaData},
        layout::Bounds,
    },
};
use log::error;
use wgpu::{SamplerDescriptor, util::DeviceExt};

use super::{Positionable, Renderable};

pub struct ImageComponent {}

impl Configurable for ImageComponent {
    fn configure(
        component: &mut super::component::Component,
        config: super::component::ComponentConfig,
        wgpu_ctx: &mut crate::wgpu_ctx::WgpuCtx,
    ) -> Vec<super::component::ComponentMetaData> {
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
        let screen_size = wgpu_ctx.get_screen_size();
        let vertices = component.calculate_vertices(Some(Bounds::default()), None, screen_size);
        let indices = component.get_indices();

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
        let vertex_buffer = wgpu_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let index_buffer = wgpu_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        vec![
            ComponentMetaData::VertexBuffer(vertex_buffer),
            ComponentMetaData::IndexBuffer(index_buffer),
            ComponentMetaData::BindGroup(bind_group),
        ]
    }
}

impl Renderable for ImageComponent {
    fn draw(
        component: &mut super::component::Component,
        render_pass: &mut wgpu::RenderPass,
        app_pipelines: &mut crate::wgpu_ctx::AppPipelines,
    ) {
        let indices = component.get_indices();
        let vertex_buffer = component.get_vertex_buffer();
        let index_buffer = component.get_index_buffer();
        let bind_group = component.get_bind_group();

        if vertex_buffer.is_none() || index_buffer.is_none() || bind_group.is_none() {
            error!(
                "Vertex buffer, index buffer, or bind group not found for component id: {}, unable to draw",
                component.id
            );
            return;
        }

        let vertex_buffer = vertex_buffer.unwrap();
        let index_buffer = index_buffer.unwrap();
        let bind_group = bind_group.unwrap();

        render_pass.set_pipeline(&app_pipelines.texture_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }
}

impl Positionable for ImageComponent {
    fn set_position(
        component: &mut super::component::Component,
        wgpu_ctx: &mut crate::wgpu_ctx::WgpuCtx,
        bounds: Bounds,
    ) {
        // Convert to NDC space
        let screen_size = wgpu_ctx.get_screen_size();
        let clip_bounds = component.convert_to_ndc(bounds, screen_size);
        let vertices = component.calculate_vertices(Some(clip_bounds), None, screen_size);

        // Update vertex buffer
        if let Some(vertex_buffer) = component.get_vertex_buffer() {
            wgpu_ctx
                .queue
                .write_buffer(vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        }
    }
}
