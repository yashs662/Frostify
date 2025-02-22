use super::{Bounds, ComponentPosition};
use crate::{
    color::Color,
    vertex::Vertex,
    wgpu_ctx::{AppPipelines, PipelinePreference},
};
use wgpu::util::DeviceExt;

pub struct BackgroundComponent {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub pipeline_preference: PipelinePreference,
}

impl BackgroundComponent {
    pub fn new(
        device: &wgpu::Device,
        bounds: Bounds,
        start_color: Color,
        end_color: Color,
        angle: f32,
        pipeline_preference: PipelinePreference,
    ) -> Self {
        // Calculate gradient direction vector based on angle
        let (sin, cos) = angle.sin_cos();
        let direction = [cos, sin];
        let colors = Color::gradient(start_color, end_color, direction);

        // Initial vertices in non-NDC space (will be converted during resize)
        let vertices = vec![
            Vertex::new(
                [bounds.position.x, bounds.position.y, 0.0],
                colors[0],
                [0.0, 0.0],
            ),
            Vertex::new(
                [
                    bounds.position.x + bounds.size.width,
                    bounds.position.y,
                    0.0,
                ],
                colors[1],
                [1.0, 0.0],
            ),
            Vertex::new(
                [
                    bounds.position.x + bounds.size.width,
                    bounds.position.y + bounds.size.height,
                    0.0,
                ],
                colors[2],
                [1.0, 1.0],
            ),
            Vertex::new(
                [
                    bounds.position.x,
                    bounds.position.y + bounds.size.height,
                    0.0,
                ],
                colors[3],
                [0.0, 1.0],
            ),
        ];

        let indices = vec![0, 1, 2, 0, 2, 3];

        // Create buffers
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Background Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Background Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create an empty bind group for solid colors and gradients
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[],
            label: Some("Background Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[],
            label: Some("Background Bind Group"),
        });

        Self {
            vertex_buffer,
            index_buffer,
            bind_group,
            vertices,
            indices,
            pipeline_preference,
        }
    }

    pub fn draw<'a, 'b: 'a>(
        &'b self,
        render_pass: &mut wgpu::RenderPass<'a>,
        app_pipelines: &mut AppPipelines,
    ) {
        match self.pipeline_preference {
            PipelinePreference::Color => {
                render_pass.set_pipeline(&app_pipelines.color_pipeline);
            }
            PipelinePreference::Texture => {
                render_pass.set_pipeline(&app_pipelines.texture_pipeline);
            }
        }
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32, bounds: Bounds) {
        // Convert pixel coordinates to NDC coordinates
        let ndc_x = (bounds.position.x / width as f32) * 2.0 - 1.0;
        let ndc_y = 1.0 - (bounds.position.y / height as f32) * 2.0;
        let ndc_width = (bounds.size.width / width as f32) * 2.0;
        let ndc_height = (bounds.size.height / height as f32) * 2.0;

        self.vertices = vec![
            Vertex::new([ndc_x, ndc_y, 0.0], self.vertices[0].color, [0.0, 0.0]),
            Vertex::new(
                [ndc_x + ndc_width, ndc_y, 0.0],
                self.vertices[1].color,
                [1.0, 0.0],
            ),
            Vertex::new(
                [ndc_x + ndc_width, ndc_y - ndc_height, 0.0],
                self.vertices[2].color,
                [1.0, 1.0],
            ),
            Vertex::new(
                [ndc_x, ndc_y - ndc_height, 0.0],
                self.vertices[3].color,
                [0.0, 1.0],
            ),
        ];
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
    }

    pub fn set_position(
        &mut self,
        queue: &wgpu::Queue,
        position: ComponentPosition,
        bounds: Bounds,
    ) {
        self.vertices = vec![
            Vertex::new(
                [position.x, position.y, 0.0],
                self.vertices[0].color,
                [0.0, 0.0],
            ),
            Vertex::new(
                [position.x + bounds.size.width, position.y, 0.0],
                self.vertices[1].color,
                [1.0, 0.0],
            ),
            Vertex::new(
                [
                    position.x + bounds.size.width,
                    position.y + bounds.size.height,
                    0.0,
                ],
                self.vertices[2].color,
                [1.0, 1.0],
            ),
            Vertex::new(
                [position.x, position.y + bounds.size.height, 0.0],
                self.vertices[3].color,
                [0.0, 1.0],
            ),
        ];

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
    }
}
