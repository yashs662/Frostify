use super::{Bounds, Component, ComponentPosition, ComponentSize, DrawableComponent};
use crate::img_utils::RgbaImg;
use crate::vertex::Vertex;
use wgpu::util::DeviceExt;
use wgpu::SamplerDescriptor;

pub struct ImageComponent {
    drawable: DrawableComponent,
    size: ComponentSize,
    position: ComponentPosition,
    children: Vec<Box<dyn Component>>,
}

impl ImageComponent {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_path: &str,
        size: ComponentSize,
        position: ComponentPosition,
    ) -> Self {
        let img = RgbaImg::new(texture_path).unwrap();
        let vertices = create_vertices(position.x, position.y, size.width, size.height);

        // Create texture and bind group
        let texture_size = wgpu::Extent3d {
            width: img.width,
            height: img.height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
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
        queue.write_texture(
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

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        let indices = vec![0, 1, 2, 0, 2, 3];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            drawable: DrawableComponent {
                vertex_buffer,
                index_buffer,
                bind_group,
                vertices,
                indices,
            },
            size,
            position,
            children: Vec::new(),
        }
    }

    pub fn get_position(&self) -> ComponentPosition {
        self.position
    }

    pub fn get_size(&self) -> ComponentSize {
        self.size
    }
}

impl Component for ImageComponent {
    fn update(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.drawable.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.drawable.vertices),
        );
    }

    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_bind_group(0, &self.drawable.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.drawable.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.drawable.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..self.drawable.indices.len() as u32, 0, 0..1);

        // Draw all children
        for child in &self.children {
            child.draw(render_pass);
        }
    }

    fn resize(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, width: u32, height: u32) {
        // Convert pixel coordinates to NDC coordinates using top-left as reference
        let ndc_x = (self.position.x / width as f32) * 2.0 - 1.0;
        let ndc_y = 1.0 - (self.position.y / height as f32) * 2.0;
        let ndc_width = (self.size.width / width as f32) * 2.0;
        let ndc_height = (self.size.height / height as f32) * 2.0;

        // Create vertices with top-left positioning
        self.drawable.vertices = create_vertices(ndc_x, ndc_y, ndc_width, ndc_height);
        self.update(queue);

        // Resize children
        for child in &mut self.children {
            child.resize(queue, device, width, height);
        }
    }

    fn set_position(
        &mut self,
        queue: &wgpu::Queue,
        _device: &wgpu::Device,
        position: ComponentPosition,
    ) {
        self.position = position;
        self.drawable.vertices =
            create_vertices(position.x, position.y, self.size.width, self.size.height);
        self.update(queue);
    }

    fn handle_mouse_click(&mut self, x: f32, y: f32) -> bool {
        // First check if any children handle the click
        for child in &mut self.children {
            if child.handle_mouse_click(x, y) {
                return true;
            }
        }
        false
    }

    // Implement new Component trait methods
    fn add_child(&mut self, child: Box<dyn Component>) {
        self.children.push(child);
    }

    fn remove_child(&mut self, index: usize) -> Option<Box<dyn Component>> {
        if index < self.children.len() {
            Some(self.children.remove(index))
        } else {
            None
        }
    }

    fn get_children(&self) -> &Vec<Box<dyn Component>> {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn Component>> {
        &mut self.children
    }

    fn get_bounds(&self) -> Bounds {
        Bounds::new(self.position, self.size)
    }
}

fn create_vertices(x: f32, y: f32, width: f32, height: f32) -> Vec<Vertex> {
    vec![
        // Top-left
        Vertex::new([x, y, 0.0], [1.0, 1.0, 1.0, 1.0], [0.0, 0.0]),
        // Top-right
        Vertex::new([x + width, y, 0.0], [1.0, 1.0, 1.0, 1.0], [1.0, 0.0]),
        // Bottom-right
        Vertex::new(
            [x + width, y - height, 0.0],
            [1.0, 1.0, 1.0, 1.0],
            [1.0, 1.0],
        ),
        // Bottom-left
        Vertex::new([x, y - height, 0.0], [1.0, 1.0, 1.0, 1.0], [0.0, 1.0]),
    ]
}
