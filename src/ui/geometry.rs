use crate::ui::ecs::RenderBufferData;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 2], // Position in clip space (-1 to 1)
    pub uv: [f32; 2],       // UV coordinates (0 to 1)
}

impl QuadVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct QuadGeometry {
    pub vertices: [QuadVertex; 4],
    pub indices: [u16; 6],
}

impl QuadGeometry {
    /// Create a quad that covers the component bounds including shadow expansion
    pub fn from_component_bounds(
        render_data: &RenderBufferData,
        screen_width: f32,
        screen_height: f32,
    ) -> Self {
        // Calculate expanded bounds that include shadow
        let shadow_expansion_x = render_data.shadow_offset[0].abs() + render_data.shadow_blur * 2.0;
        let shadow_expansion_y = render_data.shadow_offset[1].abs() + render_data.shadow_blur * 2.0;

        let expanded_min_x = render_data.outer_bounds[0] - shadow_expansion_x;
        let expanded_min_y = render_data.outer_bounds[1] - shadow_expansion_y;
        let expanded_max_x = render_data.outer_bounds[2] + shadow_expansion_x;
        let expanded_max_y = render_data.outer_bounds[3] + shadow_expansion_y;

        // Convert to clip space coordinates (-1 to 1)
        let clip_min_x = (expanded_min_x / screen_width) * 2.0 - 1.0;
        let clip_max_x = (expanded_max_x / screen_width) * 2.0 - 1.0;
        let clip_min_y = -((expanded_min_y / screen_height) * 2.0 - 1.0); // Flip Y
        let clip_max_y = -((expanded_max_y / screen_height) * 2.0 - 1.0); // Flip Y
        // Create quad vertices (counter-clockwise)
        // Note: Since we flip the Y-axis for clip space, we need to adjust UV coordinates accordingly
        let vertices = [
            // Top-left (in screen space, but bottom-left in clip space due to Y flip)
            QuadVertex {
                position: [clip_min_x, clip_max_y],
                uv: [0.0, 1.0], // Flipped V coordinate
            },
            // Top-right (in screen space, but bottom-right in clip space due to Y flip)
            QuadVertex {
                position: [clip_max_x, clip_max_y],
                uv: [1.0, 1.0], // Flipped V coordinate
            },
            // Bottom-left (in screen space, but top-left in clip space due to Y flip)
            QuadVertex {
                position: [clip_min_x, clip_min_y],
                uv: [0.0, 0.0], // Flipped V coordinate
            },
            // Bottom-right (in screen space, but top-right in clip space due to Y flip)
            QuadVertex {
                position: [clip_max_x, clip_min_y],
                uv: [1.0, 0.0], // Flipped V coordinate
            },
        ];

        // Create indices for two triangles forming a quad
        let indices = [
            0, 1, 2, // First triangle: top-left, top-right, bottom-left
            2, 1, 3, // Second triangle: bottom-left, top-right, bottom-right
        ];

        Self { vertices, indices }
    }

    /// Create vertex and index buffers for this quad
    pub fn create_buffers(&self, device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }
}
