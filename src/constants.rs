pub const TEXTURE_BIND_GROUP_LAYOUT_ENTIRES: &[wgpu::BindGroupLayoutEntry] = &[
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
];

pub const WINDOW_RESIZE_BORDER_WIDTH: f64 = 2.0;
pub const ROUNDED_CORNER_SEGMENT_COUNT: u32 = 16;
pub const WINDOW_CONTROL_BUTTON_SIZE: f32 = 24.0;
