use crate::{
    test::test_utils::{get_event_sender, setup_asset_store_for_testing},
    ui::{
        ecs::builders::{
            EntityBuilder,
            container::ContainerBuilder,
            image::{ImageBuilder, ScaleMode},
        },
        layout::{Anchor, FlexDirection, FlexValue, LayoutContext, Position, Size},
    },
    wgpu_ctx::WgpuCtx,
};

#[test]
fn stretch_scaling_in_flex_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Initialize asset store
    setup_asset_store_for_testing();

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    let test_image = ImageBuilder::new("test.png")
        .with_debug_name("Test Image")
        .with_scale_mode(ScaleMode::Stretch)
        .build(&mut ctx.world, &mut wgpu_ctx, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, test_image);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let test_image_bounds = computed_bounds.get(&test_image).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test image bounds
    assert_eq!(test_image_bounds.size.width, 500.0);
    assert_eq!(test_image_bounds.size.height, 300.0);
    assert_eq!(test_image_bounds.position.x, 0.0);
    assert_eq!(test_image_bounds.position.y, 0.0);
}

#[test]
fn contain_scaling_in_flex_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Initialize asset store
    setup_asset_store_for_testing();

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    let test_image = ImageBuilder::new("test.png")
        .with_debug_name("Test Image")
        .with_scale_mode(ScaleMode::Contain)
        .build(&mut ctx.world, &mut wgpu_ctx, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, test_image);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let test_image_bounds = computed_bounds.get(&test_image).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test image bounds - Test image has an aspect ratio of 1:1 and size 512x512 px
    assert_eq!(test_image_bounds.size.width, 300.0);
    assert_eq!(test_image_bounds.size.height, 300.0);

    // Test image is centered in the parent container
    assert_eq!(test_image_bounds.position.x, 100.0);
    assert_eq!(test_image_bounds.position.y, 0.0);
}

#[test]
fn cover_scaling_in_flex_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Initialize asset store
    setup_asset_store_for_testing();

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    let test_image = ImageBuilder::new("test.png")
        .with_debug_name("Test Image")
        .with_scale_mode(ScaleMode::Cover)
        .build(&mut ctx.world, &mut wgpu_ctx, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, test_image);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let test_image_bounds = computed_bounds.get(&test_image).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test image bounds - Test image has an aspect ratio of 1:1 and size 512x512 px
    assert_eq!(test_image_bounds.size.width, 500.0);
    assert_eq!(test_image_bounds.size.height, 500.0);
    // Test image is centered in the parent container
    assert_eq!(test_image_bounds.position.x, 0.0);
    assert_eq!(test_image_bounds.position.y, -100.0);
}

#[test]
fn original_scaling_in_flex_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Initialize asset store
    setup_asset_store_for_testing();

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(600.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    let test_image = ImageBuilder::new("test.png")
        .with_debug_name("Test Image")
        .with_scale_mode(ScaleMode::Original)
        .build(&mut ctx.world, &mut wgpu_ctx, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, test_image);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let test_image_bounds = computed_bounds.get(&test_image).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 600.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test image bounds - Test image has an aspect ratio of 1:1 and size 512x512 px
    assert_eq!(test_image_bounds.size.width, 512.0);
    assert_eq!(test_image_bounds.size.height, 512.0);

    // Test image is centered in the parent container
    assert_eq!(test_image_bounds.position.x, 44.0);
    assert_eq!(test_image_bounds.position.y, -106.0);
}

#[test]
fn stretch_scaling_in_fixed_layout_center_anchor() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Initialize asset store
    setup_asset_store_for_testing();

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    let test_image = ImageBuilder::new("test.png")
        .with_debug_name("Test Image")
        .with_scale_mode(ScaleMode::Stretch)
        .with_fixed_position(Anchor::Center)
        .build(&mut ctx.world, &mut wgpu_ctx, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, test_image);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let test_image_bounds = computed_bounds.get(&test_image).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test image bounds
    assert_eq!(test_image_bounds.size.width, 500.0);
    assert_eq!(test_image_bounds.size.height, 300.0);
    assert_eq!(test_image_bounds.position.x, 0.0);
    assert_eq!(test_image_bounds.position.y, 0.0);
}

#[test]
fn contain_scaling_in_fixed_layout_center_anchor() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Initialize asset store
    setup_asset_store_for_testing();

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    let test_image = ImageBuilder::new("test.png")
        .with_debug_name("Test Image")
        .with_scale_mode(ScaleMode::Contain)
        .with_fixed_position(Anchor::Center)
        .build(&mut ctx.world, &mut wgpu_ctx, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, test_image);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let test_image_bounds = computed_bounds.get(&test_image).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test image bounds - Test image has an aspect ratio of 1:1 and size 512x512 px
    assert_eq!(test_image_bounds.size.width, 300.0);
    assert_eq!(test_image_bounds.size.height, 300.0);

    // Test image is centered in the parent container
    assert_eq!(test_image_bounds.position.x, 100.0);
    assert_eq!(test_image_bounds.position.y, 0.0);
}

#[test]
fn contain_scaling_in_fixed_layout_top_left_anchor() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Initialize asset store
    setup_asset_store_for_testing();

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    let test_image = ImageBuilder::new("test.png")
        .with_debug_name("Test Image")
        .with_scale_mode(ScaleMode::Contain)
        .with_fixed_position(Anchor::TopLeft)
        .build(&mut ctx.world, &mut wgpu_ctx, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, test_image);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let test_image_bounds = computed_bounds.get(&test_image).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test image bounds - Test image has an aspect ratio of 1:1 and size 512x512 px
    assert_eq!(test_image_bounds.size.width, 300.0);
    assert_eq!(test_image_bounds.size.height, 300.0);

    // Test image is centered in the parent container
    assert_eq!(test_image_bounds.position.x, 0.0);
    assert_eq!(test_image_bounds.position.y, 0.0);
}

#[test]
fn cover_scaling_in_fixed_layout_center_anchor() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Initialize asset store
    setup_asset_store_for_testing();

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    let test_image = ImageBuilder::new("test.png")
        .with_debug_name("Test Image")
        .with_scale_mode(ScaleMode::Cover)
        .with_fixed_position(Anchor::Center)
        .build(&mut ctx.world, &mut wgpu_ctx, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, test_image);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let test_image_bounds = computed_bounds.get(&test_image).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test image bounds - Test image has an aspect ratio of 1:1 and size 512x512 px
    assert_eq!(test_image_bounds.size.width, 500.0);
    assert_eq!(test_image_bounds.size.height, 500.0);
    // Test image is centered in the parent container
    assert_eq!(test_image_bounds.position.x, 0.0);
    assert_eq!(test_image_bounds.position.y, -100.0);
}

#[test]
fn cover_scaling_in_fixed_layout_top_left_anchor() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Initialize asset store
    setup_asset_store_for_testing();

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    let test_image = ImageBuilder::new("test.png")
        .with_debug_name("Test Image")
        .with_scale_mode(ScaleMode::Cover)
        .with_fixed_position(Anchor::TopLeft)
        .build(&mut ctx.world, &mut wgpu_ctx, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, test_image);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let test_image_bounds = computed_bounds.get(&test_image).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test image bounds - Test image has an aspect ratio of 1:1 and size 512x512 px
    assert_eq!(test_image_bounds.size.width, 500.0);
    assert_eq!(test_image_bounds.size.height, 500.0);
    // Test image is centered in the parent container
    assert_eq!(test_image_bounds.position.x, 0.0);
    assert_eq!(test_image_bounds.position.y, 0.0);
}

#[test]
fn original_scaling_in_fixed_layout_center_anchor() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Initialize asset store
    setup_asset_store_for_testing();

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(600.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    let test_image = ImageBuilder::new("test.png")
        .with_debug_name("Test Image")
        .with_scale_mode(ScaleMode::Original)
        .with_fixed_position(Anchor::Center)
        .build(&mut ctx.world, &mut wgpu_ctx, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, test_image);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let test_image_bounds = computed_bounds.get(&test_image).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 600.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test image bounds - Test image has an aspect ratio of 1:1 and size 512x512 px
    assert_eq!(test_image_bounds.size.width, 512.0);
    assert_eq!(test_image_bounds.size.height, 512.0);

    // Test image is centered in the parent container
    assert_eq!(test_image_bounds.position.x, 44.0);
    assert_eq!(test_image_bounds.position.y, -106.0);
}

#[test]
fn original_scaling_in_fixed_layout_top_left_anchor() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Initialize asset store
    setup_asset_store_for_testing();

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(600.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    let test_image = ImageBuilder::new("test.png")
        .with_debug_name("Test Image")
        .with_scale_mode(ScaleMode::Original)
        .with_fixed_position(Anchor::TopLeft)
        .build(&mut ctx.world, &mut wgpu_ctx, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, test_image);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let test_image_bounds = computed_bounds.get(&test_image).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 600.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test image bounds - Test image has an aspect ratio of 1:1 and size 512x512 px
    assert_eq!(test_image_bounds.size.width, 512.0);
    assert_eq!(test_image_bounds.size.height, 512.0);

    // Test image is centered in the parent container
    assert_eq!(test_image_bounds.position.x, 0.0);
    assert_eq!(test_image_bounds.position.y, 0.0);
}
