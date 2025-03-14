use crate::constants::UNIFIED_BIND_GROUP_LAYOUT_ENTRIES;

pub fn create_unified_pipeline(
    device: &wgpu::Device,
    swap_chain_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    // Create unified shader for both color and texture
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Unified Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/color.wgsl").into()),
    });

    // Create unified bind group layout that supports both color-only and texture rendering
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
        label: Some("Unified Bind Group Layout"),
    });

    // Pipeline layout that works for both standard components and frosted glass
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Unified Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Unified Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[], // No vertex buffers needed for full-screen triangle approach
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: swap_chain_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}
