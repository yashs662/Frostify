use crate::ui::{
    ecs::{
        BorderPosition, ComponentType, EcsComponents, EntityId, RenderBufferData, World,
        components::{
            BoundsComponent, ColorComponent, FrostedGlassComponent, HierarchyComponent,
            IdentityComponent, InteractionComponent, VisualComponent,
        },
    },
    geometry::QuadVertex,
};

pub fn create_unified_pipeline(
    device: &wgpu::Device,
    swap_chain_format: wgpu::TextureFormat,
    unified_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    // Create unified shader for both color and texture
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Unified Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../assets/shaders/color.wgsl").into()),
    });

    // Pipeline layout that works for both standard components and frosted glass
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Unified Pipeline Layout"),
        bind_group_layouts: &[unified_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Unified Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[QuadVertex::desc()], // Use vertex buffers for quad geometry
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

pub fn create_entity_buffer_data(
    components: &EcsComponents,
    entity_id: EntityId,
) -> RenderBufferData {
    // Necessary components for rendering
    let bounds_comp = components
        .get_component::<BoundsComponent>(entity_id)
        .expect("Expected BoundsComponent to exist while preparing render data");
    let visual_comp = components
        .get_component::<VisualComponent>(entity_id)
        .expect("Expected VisualComponent to exist while preparing render data");
    let identity_comp = components
        .get_component::<IdentityComponent>(entity_id)
        .expect("Expected IdentityComponent to exist while preparing render data");

    let default_color = [1.0, 0.0, 1.0, 1.0];
    let position = [
        bounds_comp.computed_bounds.position.x,
        bounds_comp.computed_bounds.position.y,
    ];
    let size = [
        bounds_comp.computed_bounds.size.width,
        bounds_comp.computed_bounds.size.height,
    ];

    // Get color and frosted glass parameters if available
    let (color, blur_radius, tint_intensity) = match identity_comp.component_type {
        ComponentType::BackgroundColor => {
            let color_comp = components
                .get_component::<ColorComponent>(entity_id)
                .expect("BackgroundColor Type Component should have ColorComponent");
            (color_comp.color.value(), 0.0, 0.0)
        }
        ComponentType::FrostedGlass => {
            let frosted_glass_comp = components
                .get_component::<FrostedGlassComponent>(entity_id)
                .expect("FrostedGlass Type Component should have FrostedGlassComponent");
            (
                frosted_glass_comp.tint_color.value(),
                frosted_glass_comp.blur_radius,
                frosted_glass_comp.tint_intensity,
            )
        }
        _ => (default_color, 0.0, 0.0),
    };

    let use_texture = match identity_comp.component_type {
        ComponentType::BackgroundGradient | ComponentType::Image | ComponentType::Text => 1,
        ComponentType::FrostedGlass => 2,
        _ => 0,
    };

    // Convert border position enum to u32 for shader
    let border_position_value = match visual_comp.border_position {
        BorderPosition::Inside => 0u32,
        BorderPosition::Center => 1u32,
        BorderPosition::Outside => 2u32,
    };

    // Pre-compute corner properties
    let content_min = vec![
        bounds_comp.computed_bounds.position.x,
        bounds_comp.computed_bounds.position.y,
    ];
    let content_max = vec![
        bounds_comp.computed_bounds.position.x + bounds_comp.computed_bounds.size.width,
        bounds_comp.computed_bounds.position.y + bounds_comp.computed_bounds.size.height,
    ];

    // Calculate max radius to prevent overlap
    let max_radius_x = bounds_comp.computed_bounds.size.width * 0.5;
    let max_radius_y = bounds_comp.computed_bounds.size.height * 0.5;
    let max_radius = max_radius_x.min(max_radius_y);

    // Clamp all radii to max
    let tl_radius = visual_comp.border_radius.top_left.min(max_radius);
    let tr_radius = visual_comp.border_radius.top_right.min(max_radius);
    let bl_radius = visual_comp.border_radius.bottom_left.min(max_radius);
    let br_radius = visual_comp.border_radius.bottom_right.min(max_radius);

    // Calculate outer radii based on border position
    let (
        outer_tl_radius,
        outer_tr_radius,
        outer_bl_radius,
        outer_br_radius,
        inner_tl_radius,
        inner_tr_radius,
        inner_bl_radius,
        inner_br_radius,
    ) = if visual_comp.border_width > 0.0 {
        match visual_comp.border_position {
            BorderPosition::Inside => (
                tl_radius,
                tr_radius,
                bl_radius,
                br_radius,
                (tl_radius - visual_comp.border_width).max(0.0),
                (tr_radius - visual_comp.border_width).max(0.0),
                (bl_radius - visual_comp.border_width).max(0.0),
                (br_radius - visual_comp.border_width).max(0.0),
            ),
            BorderPosition::Center => {
                let half_border = visual_comp.border_width * 0.5;
                (
                    tl_radius + half_border,
                    tr_radius + half_border,
                    bl_radius + half_border,
                    br_radius + half_border,
                    (tl_radius - half_border).max(0.0),
                    (tr_radius - half_border).max(0.0),
                    (bl_radius - half_border).max(0.0),
                    (br_radius - half_border).max(0.0),
                )
            }
            BorderPosition::Outside => (
                tl_radius + visual_comp.border_width,
                tr_radius + visual_comp.border_width,
                bl_radius + visual_comp.border_width,
                br_radius + visual_comp.border_width,
                tl_radius,
                tr_radius,
                bl_radius,
                br_radius,
            ),
        }
    } else {
        (
            tl_radius, tr_radius, bl_radius, br_radius, tl_radius, tr_radius, bl_radius, br_radius,
        )
    };

    // Calculate corner centers
    let tl_center = [content_min[0] + tl_radius, content_min[1] + tl_radius];
    let tr_center = [content_max[0] - tr_radius, content_min[1] + tr_radius];
    let bl_center = [content_min[0] + bl_radius, content_max[1] - bl_radius];
    let br_center = [content_max[0] - br_radius, content_max[1] - br_radius];

    // Calculate inner and outer bounds
    let (inner_min, inner_max, outer_min, outer_max) = if visual_comp.border_width > 0.0 {
        match visual_comp.border_position {
            BorderPosition::Inside => (
                vec![
                    content_min[0] + visual_comp.border_width,
                    content_min[1] + visual_comp.border_width,
                ],
                vec![
                    content_max[0] - visual_comp.border_width,
                    content_max[1] - visual_comp.border_width,
                ],
                content_min,
                content_max,
            ),
            BorderPosition::Center => {
                let half_border = visual_comp.border_width * 0.5;
                (
                    vec![content_min[0] + half_border, content_min[1] + half_border],
                    vec![content_max[0] - half_border, content_max[1] - half_border],
                    vec![content_min[0] - half_border, content_min[1] - half_border],
                    vec![content_max[0] + half_border, content_max[1] + half_border],
                )
            }
            BorderPosition::Outside => (
                content_min.clone(),
                content_max.clone(),
                vec![
                    content_min[0] - visual_comp.border_width,
                    content_min[1] - visual_comp.border_width,
                ],
                vec![
                    content_max[0] + visual_comp.border_width,
                    content_max[1] + visual_comp.border_width,
                ],
            ),
        }
    } else {
        (
            content_min.clone(),
            content_max.clone(),
            content_min,
            content_max,
        )
    };

    let (clip_bounds, clip_border_radius, clip_enabled) =
        if let Some(clip_bounds) = &bounds_comp.clip_bounds {
            (
                [
                    clip_bounds.bounds.position.x,
                    clip_bounds.bounds.position.y,
                    clip_bounds.bounds.position.x + clip_bounds.bounds.size.width,
                    clip_bounds.bounds.position.y + clip_bounds.bounds.size.height,
                ],
                [
                    clip_bounds.border_radius.top_left,
                    clip_bounds.border_radius.top_right,
                    clip_bounds.border_radius.bottom_left,
                    clip_bounds.border_radius.bottom_right,
                ],
                [
                    if clip_bounds.clip_x { 1.0 } else { 0.0 },
                    if clip_bounds.clip_y { 1.0 } else { 0.0 },
                ],
            )
        } else {
            // Default to full screen with no clipping
            (
                [
                    0.0,
                    0.0,
                    bounds_comp.screen_size.width as f32,
                    bounds_comp.screen_size.height as f32,
                ],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0],
            )
        };

    RenderBufferData {
        color,
        position,
        size,
        border_radius: visual_comp.border_radius.values(),
        screen_size: [
            bounds_comp.screen_size.width as f32,
            bounds_comp.screen_size.height as f32,
        ],
        use_texture,
        blur_radius,
        opacity: visual_comp.opacity,
        tint_intensity,
        border_width: visual_comp.border_width,
        border_position: border_position_value,
        border_color: visual_comp.border_color.value(),
        inner_bounds: [inner_min[0], inner_min[1], inner_max[0], inner_max[1]],
        outer_bounds: [outer_min[0], outer_min[1], outer_max[0], outer_max[1]],
        corner_centers: [tl_center[0], tl_center[1], tr_center[0], tr_center[1]],
        corner_centers2: [bl_center[0], bl_center[1], br_center[0], br_center[1]],
        corner_radii: [
            inner_tl_radius,
            inner_tr_radius,
            inner_bl_radius,
            inner_br_radius,
        ],
        corner_radii2: [
            outer_tl_radius,
            outer_tr_radius,
            outer_bl_radius,
            outer_br_radius,
        ],
        shadow_color: visual_comp.shadow_color.value(),
        shadow_offset: [visual_comp.shadow_offset.0, visual_comp.shadow_offset.1],
        shadow_blur: visual_comp.shadow_blur,
        shadow_opacity: visual_comp.shadow_opacity,
        clip_bounds,
        clip_border_radius,
        clip_enabled,
        _padding3: [0.0; 8],
    }
}

/// function to iteratively collect all children entities
pub fn gather_all_children(world: &World, root_entity_id: EntityId) -> Vec<EntityId> {
    let mut all_children = Vec::new();
    let mut to_process = vec![root_entity_id];

    while let Some(entity_id) = to_process.pop() {
        let hierarchy_comp = world
            .components
            .get_component::<HierarchyComponent>(entity_id)
            .expect("Expected HierarchyComponent to be present");

        for &child_id in &hierarchy_comp.children {
            all_children.push(child_id);
            to_process.push(child_id);
        }
    }

    all_children
}

/// function to deactivate a component and all its children
pub fn deactivate_component_and_children(world: &mut World, entity_id: EntityId) {
    // Deactivate the modal parent
    let interaction_comp = world
        .components
        .get_component_mut::<InteractionComponent>(entity_id)
        .expect("Expected InteractionComponent to be present for modal parent entity");

    interaction_comp.is_active = false;
    interaction_comp.is_just_activated = false;
    interaction_comp.is_just_deactivated = false;

    // Deactivate all children
    for child_id in gather_all_children(world, entity_id) {
        let interaction_comp = world
            .components
            .get_component_mut::<InteractionComponent>(child_id)
            .expect("Expected InteractionComponent to be present for modal child entity");

        interaction_comp.is_active = false;
        interaction_comp.is_just_activated = false;
        interaction_comp.is_just_deactivated = false;
    }
}

/// function to iteratively collect all children entities with their component types
pub fn gather_all_children_with_types(
    world: &World,
    root_entity_id: EntityId,
) -> Vec<(EntityId, ComponentType)> {
    let mut all_children = Vec::new();
    let mut to_process = vec![root_entity_id];

    while let Some(entity_id) = to_process.pop() {
        let hierarchy_comp = world
            .components
            .get_component::<HierarchyComponent>(entity_id)
            .expect("Expected HierarchyComponent to be present");

        for &child_id in &hierarchy_comp.children {
            // Get the component type of the child entity
            let component_type = world
                .components
                .get_component::<IdentityComponent>(child_id)
                .expect("Expected IdentityComponent to be present")
                .component_type;

            all_children.push((child_id, component_type));
            to_process.push(child_id);
        }
    }

    all_children
}

#[derive(Default)]
pub enum AppFonts {
    #[default]
    CenturyGothic,
    CenturyGothicBold,
}

impl AppFonts {
    pub const fn as_str(&self) -> &'static str {
        match self {
            AppFonts::CenturyGothic => "CenturyGothic",
            AppFonts::CenturyGothicBold => "CenturyGothicBold",
        }
    }

    pub const fn as_family_name(&self) -> &'static str {
        match self {
            AppFonts::CenturyGothic => "Century Gothic",
            AppFonts::CenturyGothicBold => "Century Gothic",
        }
    }
}
