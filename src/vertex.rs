#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub fn new(position: [f32; 3], color: [f32; 4], tex_coords: [f32; 2]) -> Self {
        Self {
            position,
            color,
            tex_coords,
        }
    }
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

pub const VERTEX_INDEX_LIST: &[u16] = &[
    0, 1, 2,  // First triangle (top-left, top-right, bottom-right)
    0, 2, 3,  // Second triangle (top-left, bottom-right, bottom-left)
];

pub fn create_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    use std::mem::size_of;
    wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: (size_of::<[f32; 3]>() + size_of::<[f32; 4]>()) as wgpu::BufferAddress,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x2,
            },
        ],
    }
}