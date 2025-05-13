use crate::{
    app::AppEvent,
    ui::{
        ecs::builders::{EntityBuilder, container::ContainerBuilder},
        layout::*,
    },
    wgpu_ctx::WgpuCtx,
};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

fn get_event_sender() -> UnboundedSender<AppEvent> {
    let (event_tx, _) = unbounded_channel::<AppEvent>();
    event_tx
}

#[test]
fn test_basic_fixed_flex_row_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create second child with fixed size
    let child2_id = ContainerBuilder::new()
        .with_debug_name("Child2 Container")
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Set children on parent
    ctx.add_child_to_parent(parent_id, child1_id);
    ctx.add_child_to_parent(parent_id, child2_id);

    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_id_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 100.0);
    assert_eq!(child1_id_bounds.size.height, 100.0);
    assert_eq!(child2_id_bounds.size.width, 100.0);
    assert_eq!(child2_id_bounds.size.height, 100.0);

    // Test child positions - in row layout
    assert_eq!(child1_id_bounds.position.x, 0.0);
    assert_eq!(child2_id_bounds.position.x, 100.0);

    // Children should be at the same Y position
    assert_eq!(child1_id_bounds.position.y, child2_id_bounds.position.y);
}

#[test]
fn test_basic_fixed_flex_column_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Column)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create second child with fixed size
    let child2_id = ContainerBuilder::new()
        .with_debug_name("Child2 Container")
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Set children on parent
    ctx.add_child_to_parent(parent_id, child1_id);
    ctx.add_child_to_parent(parent_id, child2_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_id_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 100.0);
    assert_eq!(child1_id_bounds.size.height, 100.0);
    assert_eq!(child2_id_bounds.size.width, 100.0);
    assert_eq!(child2_id_bounds.size.height, 100.0);

    // Test child positions - in column layout
    assert_eq!(child1_id_bounds.position.y, 0.0);
    assert_eq!(child2_id_bounds.position.y, 100.0);

    // Children should be at the same X position
    assert_eq!(child1_id_bounds.position.x, child2_id_bounds.position.x);
}

#[test]
fn test_basic_fill_flex_row_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create second child with fill size
    let child2_id = ContainerBuilder::new()
        .with_debug_name("Child2 Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Set children on parent
    ctx.add_child_to_parent(parent_id, child1_id);
    ctx.add_child_to_parent(parent_id, child2_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_id_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 100.0);
    assert_eq!(child1_id_bounds.size.height, 100.0);
    assert_eq!(child2_id_bounds.size.width, 400.0);
    assert_eq!(child2_id_bounds.size.height, 300.0);

    // Test child positions - in row layout
    assert_eq!(child1_id_bounds.position.x, 0.0);
    assert_eq!(child2_id_bounds.position.x, 100.0);

    // Children should be at the same Y position
    assert_eq!(child1_id_bounds.position.y, child2_id_bounds.position.y);
}

#[test]
fn test_basic_fill_flex_column_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Column)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create second child with fill size
    let child2_id = ContainerBuilder::new()
        .with_debug_name("Child2 Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Set children on parent
    ctx.add_child_to_parent(parent_id, child1_id);
    ctx.add_child_to_parent(parent_id, child2_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_id_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 100.0);
    assert_eq!(child1_id_bounds.size.height, 100.0);
    assert_eq!(child2_id_bounds.size.width, 500.0);
    assert_eq!(child2_id_bounds.size.height, 200.0);

    // Test child positions - in column layout
    assert_eq!(child1_id_bounds.position.y, 0.0);
    assert_eq!(child2_id_bounds.position.y, 100.0);

    // Children should be at the same X position
    assert_eq!(child1_id_bounds.position.x, child2_id_bounds.position.x);
}

#[test]
fn test_basic_fill_flex_row_layout_with_padding() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .with_padding(Edges::horizontal(10.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create second child with fill size
    let child2_id = ContainerBuilder::new()
        .with_debug_name("Child2 Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Set children on parent
    ctx.add_child_to_parent(parent_id, child1_id);
    ctx.add_child_to_parent(parent_id, child2_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_id_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 100.0);
    assert_eq!(child1_id_bounds.size.height, 100.0);
    assert_eq!(child2_id_bounds.size.width, 380.0);
    assert_eq!(child2_id_bounds.size.height, 300.0);

    // Test child positions - in row layout
    assert_eq!(child1_id_bounds.position.x, 10.0);
    assert_eq!(child2_id_bounds.position.x, 110.0);

    // Children should be at the same Y position
    assert_eq!(child1_id_bounds.position.y, child2_id_bounds.position.y);
}

#[test]
fn test_basic_fill_flex_column_layout_with_padding() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Column)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .with_padding(Edges::vertical(10.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create second child with fill size
    let child2_id = ContainerBuilder::new()
        .with_debug_name("Child2 Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Set children on parent
    ctx.add_child_to_parent(parent_id, child1_id);
    ctx.add_child_to_parent(parent_id, child2_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_id_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 100.0);
    assert_eq!(child1_id_bounds.size.height, 100.0);
    assert_eq!(child2_id_bounds.size.width, 500.0);
    assert_eq!(child2_id_bounds.size.height, 180.0);

    // Test child positions - in column layout
    assert_eq!(child1_id_bounds.position.y, 10.0);
    assert_eq!(child2_id_bounds.position.y, 110.0);

    // Children should be at the same X position
    assert_eq!(child1_id_bounds.position.x, child2_id_bounds.position.x);
}

#[test]
fn test_nested_containers_with_flex_layout_fixed_nested_container() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create nested container with fixed size
    let nested_parent_id = ContainerBuilder::new()
        .with_debug_name("Nested Parent Container")
        .with_width(FlexValue::Fixed(200.0))
        .with_height(FlexValue::Fixed(200.0))
        .with_direction(FlexDirection::Row)
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create nested child with fixed size
    let nested_child_1_id = ContainerBuilder::new()
        .with_debug_name("Nested Child 1 Container")
        .with_width(FlexValue::Fixed(25.0))
        .with_height(FlexValue::Fixed(25.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create nested child with fill size
    let nested_child_2_id = ContainerBuilder::new()
        .with_debug_name("Nested Child 2 Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Set children on parents
    ctx.add_child_to_parent(nested_parent_id, nested_child_1_id);
    ctx.add_child_to_parent(nested_parent_id, nested_child_2_id);
    ctx.add_child_to_parent(parent_id, child1_id);
    ctx.add_child_to_parent(parent_id, nested_parent_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let nested_parent_bounds = computed_bounds.get(&nested_parent_id).unwrap();
    let nested_child_1_id_bounds = computed_bounds.get(&nested_child_1_id).unwrap();
    let nested_child_2_id_bounds = computed_bounds.get(&nested_child_2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 100.0);
    assert_eq!(child1_id_bounds.size.height, 100.0);
    assert_eq!(nested_parent_bounds.size.width, 200.0);
    assert_eq!(nested_parent_bounds.size.height, 200.0);
    assert_eq!(nested_child_1_id_bounds.size.width, 25.0);
    assert_eq!(nested_child_1_id_bounds.size.height, 25.0);
    assert_eq!(nested_child_2_id_bounds.size.width, 175.0);
    assert_eq!(nested_child_2_id_bounds.size.height, 200.0);

    // Test child positions - in row layout
    assert_eq!(child1_id_bounds.position.x, 0.0);
    assert_eq!(nested_parent_bounds.position.x, 100.0);
    assert_eq!(nested_child_1_id_bounds.position.x, 100.0);
    assert_eq!(nested_child_2_id_bounds.position.x, 125.0);

    // Children should be at the same Y position
    assert_eq!(child1_id_bounds.position.y, nested_parent_bounds.position.y);
    assert_eq!(
        nested_child_1_id_bounds.position.y,
        nested_child_2_id_bounds.position.y
    );
}

#[test]
fn test_nested_containers_with_flex_layout_fill_nested_container() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Column)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create nested container with fill size
    let nested_parent_id = ContainerBuilder::new()
        .with_debug_name("Nested Parent Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .with_direction(FlexDirection::Row)
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create nested child with fixed size
    let nested_child_1_id = ContainerBuilder::new()
        .with_debug_name("Nested Child 1 Container")
        .with_width(FlexValue::Fixed(25.0))
        .with_height(FlexValue::Fixed(25.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create nested child with fill size
    let nested_child_2_id = ContainerBuilder::new()
        .with_debug_name("Nested Child 2 Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Set children on parent
    ctx.add_child_to_parent(nested_parent_id, nested_child_1_id);
    ctx.add_child_to_parent(nested_parent_id, nested_child_2_id);
    ctx.add_child_to_parent(parent_id, child1_id);
    ctx.add_child_to_parent(parent_id, nested_parent_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let nested_parent_bounds = computed_bounds.get(&nested_parent_id).unwrap();
    let nested_child_1_id_bounds = computed_bounds.get(&nested_child_1_id).unwrap();
    let nested_child_2_id_bounds = computed_bounds.get(&nested_child_2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 500.0);
    assert_eq!(child1_id_bounds.size.height, 100.0);
    assert_eq!(nested_parent_bounds.size.width, 500.0);
    assert_eq!(nested_parent_bounds.size.height, 200.0);
    assert_eq!(nested_child_1_id_bounds.size.width, 25.0);
    assert_eq!(nested_child_1_id_bounds.size.height, 25.0);
    assert_eq!(nested_child_2_id_bounds.size.width, 475.0);
    assert_eq!(nested_child_2_id_bounds.size.height, 200.0);

    // Test child positions - in column layout
    assert_eq!(child1_id_bounds.position.y, 0.0);
    assert_eq!(nested_parent_bounds.position.y, 100.0);

    // Test nested children positions
    assert_eq!(nested_child_1_id_bounds.position.x, 0.0);
    assert_eq!(nested_child_2_id_bounds.position.x, 25.0);

    // Children should be at the same Y position
    assert_eq!(
        nested_child_1_id_bounds.position.y,
        nested_child_2_id_bounds.position.y
    );
}

#[test]
fn test_navbar_app_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Parent container
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .with_direction(FlexDirection::Column)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Parent container background
    let background_id = ContainerBuilder::new()
        .with_debug_name("Background Container")
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Nav bar container
    let nav_bar_id = ContainerBuilder::new()
        .with_debug_name("Nav Bar Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fixed(100.0))
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_padding(Edges::all(10.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Nav bar buttons with fixed size
    let button_size = 24.0;

    // Minimize button
    let minimize_icon_id = ContainerBuilder::new()
        .with_debug_name("Minimize Icon")
        .with_width(FlexValue::Fixed(button_size))
        .with_height(FlexValue::Fixed(button_size))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Expand button
    let expand_icon_id = ContainerBuilder::new()
        .with_debug_name("Expand Icon")
        .with_width(FlexValue::Fixed(button_size))
        .with_height(FlexValue::Fixed(button_size))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Close button
    let close_icon_id = ContainerBuilder::new()
        .with_debug_name("Close Icon")
        .with_width(FlexValue::Fixed(button_size))
        .with_height(FlexValue::Fixed(button_size))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Content container
    let content_container_id = ContainerBuilder::new()
        .with_debug_name("Content Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .with_direction(FlexDirection::Row)
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Text with fixed size
    let text_id = ContainerBuilder::new()
        .with_debug_name("Text Container")
        .with_width(FlexValue::Fixed(200.0))
        .with_height(FlexValue::Fixed(50.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Content image
    let image_id = ContainerBuilder::new()
        .with_debug_name("Image Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Add children to the content container
    ctx.add_child_to_parent(content_container_id, text_id);
    ctx.add_child_to_parent(content_container_id, image_id);

    // Add children to the nav bar container
    ctx.add_child_to_parent(nav_bar_id, minimize_icon_id);
    ctx.add_child_to_parent(nav_bar_id, expand_icon_id);
    ctx.add_child_to_parent(nav_bar_id, close_icon_id);

    // Add children to the main container
    ctx.add_child_to_parent(parent_id, background_id);
    ctx.add_child_to_parent(parent_id, nav_bar_id);
    ctx.add_child_to_parent(parent_id, content_container_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let background_bounds = computed_bounds.get(&background_id).unwrap();
    let nav_bar_bounds = computed_bounds.get(&nav_bar_id).unwrap();
    let minimize_icon_bounds = computed_bounds.get(&minimize_icon_id).unwrap();
    let expand_icon_bounds = computed_bounds.get(&expand_icon_id).unwrap();
    let close_icon_bounds = computed_bounds.get(&close_icon_id).unwrap();
    let content_container_bounds = computed_bounds.get(&content_container_id).unwrap();
    let text_bounds = computed_bounds.get(&text_id).unwrap();
    let image_bounds = computed_bounds.get(&image_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 1000.0);
    assert_eq!(parent_bounds.size.height, 800.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test background bounds
    assert_eq!(background_bounds.size.width, 1000.0);
    assert_eq!(background_bounds.size.height, 800.0);
    assert_eq!(background_bounds.position.x, 0.0);
    assert_eq!(background_bounds.position.y, 0.0);

    // Test nav bar bounds
    assert_eq!(nav_bar_bounds.size.width, 1000.0);
    assert_eq!(nav_bar_bounds.size.height, 100.0);
    assert_eq!(nav_bar_bounds.position.x, 0.0);
    assert_eq!(nav_bar_bounds.position.y, 0.0);

    // Test minimize icon bounds
    assert_eq!(minimize_icon_bounds.size.width, button_size);
    assert_eq!(minimize_icon_bounds.size.height, button_size);
    assert_eq!(minimize_icon_bounds.position.x, 10.0);
    assert_eq!(minimize_icon_bounds.position.y, 38.0);

    // Test expand icon bounds
    assert_eq!(expand_icon_bounds.size.width, button_size);
    assert_eq!(expand_icon_bounds.size.height, button_size);
    assert_eq!(expand_icon_bounds.position.x, 34.0);
    assert_eq!(expand_icon_bounds.position.y, 38.0);

    // Test close icon bounds
    assert_eq!(close_icon_bounds.size.width, button_size);
    assert_eq!(close_icon_bounds.size.height, button_size);
    assert_eq!(close_icon_bounds.position.x, 58.0);
    assert_eq!(close_icon_bounds.position.y, 38.0);

    // Test content container bounds - should take up remaining height
    assert_eq!(content_container_bounds.size.width, 1000.0);
    assert_eq!(content_container_bounds.size.height, 700.0); // 800 - 100 = 700
    assert_eq!(content_container_bounds.position.x, 0.0);
    assert_eq!(content_container_bounds.position.y, 100.0);

    // Test text bounds
    assert_eq!(text_bounds.size.width, 200.0);
    assert_eq!(text_bounds.size.height, 50.0);
    assert_eq!(text_bounds.position.x, 0.0);
    assert_eq!(text_bounds.position.y, 100.0);

    // Test image bounds - should fill the remaining width
    assert_eq!(image_bounds.size.width, 800.0); // 1000 - 200 = 800
    assert_eq!(image_bounds.size.height, 700.0); // Same as container
    assert_eq!(image_bounds.position.x, 200.0);
    assert_eq!(image_bounds.position.y, 100.0);
}

#[test]
fn test_margin_and_padding_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .with_padding(Edges::all(10.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .with_margin(Edges::all(10.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create second child with fill size
    let child2_id = ContainerBuilder::new()
        .with_debug_name("Child2 Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .with_margin(Edges::all(20.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Set children on parent
    ctx.add_child_to_parent(parent_id, child1_id);
    ctx.add_child_to_parent(parent_id, child2_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_id_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 100.0);
    assert_eq!(child1_id_bounds.size.height, 100.0);
    assert_eq!(child2_id_bounds.size.width, 320.0);
    assert_eq!(child2_id_bounds.size.height, 240.0);

    // Test child positions - in row layout
    assert_eq!(child1_id_bounds.position.x, 20.0);
    assert_eq!(child1_id_bounds.position.y, 20.0);
    assert_eq!(child2_id_bounds.position.x, 150.0);
    assert_eq!(child2_id_bounds.position.y, 30.0);
}

#[test]
fn test_fractional_sizing_in_a_container() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::Center))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fractional size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fraction(0.3))
        .with_height(FlexValue::Fraction(0.5))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create second child with fractional size
    let child2_id = ContainerBuilder::new()
        .with_debug_name("Child2 Container")
        .with_width(FlexValue::Fraction(0.7))
        .with_height(FlexValue::Fraction(0.5))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Set children on parent
    ctx.add_child_to_parent(parent_id, child1_id);
    ctx.add_child_to_parent(parent_id, child2_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_id_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 250.0);
    assert_eq!(parent_bounds.position.y, 250.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 150.0);
    assert_eq!(child1_id_bounds.size.height, 150.0);
    assert_eq!(child2_id_bounds.size.width, 350.0);
    assert_eq!(child2_id_bounds.size.height, 150.0);
    assert_eq!(child1_id_bounds.position.x, 250.0);
    assert_eq!(child1_id_bounds.position.y, 250.0);
    assert_eq!(child2_id_bounds.position.x, 400.0);
    assert_eq!(child2_id_bounds.position.y, 250.0);
}

#[test]
fn test_offset_in_nested_container() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::Center))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .with_offset(ComponentOffset {
            x: FlexValue::Fixed(20.0),
            y: FlexValue::Fixed(20.0),
        })
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Set children on parent
    ctx.add_child_to_parent(parent_id, child1_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();
    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 250.0);
    assert_eq!(parent_bounds.position.y, 250.0);
    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 100.0);
    assert_eq!(child1_id_bounds.size.height, 100.0);
    assert_eq!(child1_id_bounds.position.x, 270.0);
    assert_eq!(child1_id_bounds.position.y, 270.0);
}

#[test]
fn test_offset_in_nested_container_with_flex_value() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::Center))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .with_offset(ComponentOffset {
            x: FlexValue::Fraction(0.5),
            y: FlexValue::Fraction(0.5),
        })
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Set children on parent
    ctx.add_child_to_parent(parent_id, child1_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();
    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 250.0);
    assert_eq!(parent_bounds.position.y, 250.0);
    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 500.0);
    assert_eq!(child1_id_bounds.size.height, 300.0);
    assert_eq!(child1_id_bounds.position.x, 500.0);
    assert_eq!(child1_id_bounds.position.y, 400.0);
}

#[test]
fn test_multiple_fill_containers_with_fraction_width_container() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, child1_id);

    let child2_id = ContainerBuilder::new()
        .with_debug_name("Child2 Container")
        .with_width(FlexValue::Fraction(0.5))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, child2_id);

    let child3_id = ContainerBuilder::new()
        .with_debug_name("Child3 Container")
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, child3_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_id_bounds = computed_bounds.get(&child2_id).unwrap();
    let child3_id_bounds = computed_bounds.get(&child3_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 125.0);
    assert_eq!(child1_id_bounds.size.height, 300.0);

    assert_eq!(child2_id_bounds.size.width, 250.0);
    assert_eq!(child2_id_bounds.size.height, 300.0);

    assert_eq!(child3_id_bounds.size.width, 125.0);
    assert_eq!(child3_id_bounds.size.height, 300.0);

    // Test child positions - in row layout
    assert_eq!(child1_id_bounds.position.x, 0.0);
    assert_eq!(child1_id_bounds.position.y, 0.0);

    assert_eq!(child2_id_bounds.position.x, 125.0);
    assert_eq!(child2_id_bounds.position.y, 0.0);

    assert_eq!(child3_id_bounds.position.x, 375.0);
    assert_eq!(child3_id_bounds.position.y, 0.0);
}

#[test]
fn test_multiple_containers_with_different_directions_and_fractional_sizing() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, child1_id);

    let child2_id = ContainerBuilder::new()
        .with_debug_name("Child2 Container")
        .with_width(FlexValue::Fraction(0.5))
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, child2_id);

    let child2_sub_child1_id = ContainerBuilder::new()
        .with_debug_name("Child2 Sub Child 1 Container")
        .with_width(FlexValue::Fraction(0.5))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    let child2_sub_child2_id = ContainerBuilder::new()
        .with_debug_name("Child2 Sub Child 2 Container")
        .with_height(FlexValue::Fixed(20.0))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(child2_id, child2_sub_child1_id);
    ctx.add_child_to_parent(child2_id, child2_sub_child2_id);

    let child3_id = ContainerBuilder::new()
        .with_debug_name("Child3 Container")
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, child3_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_id_bounds = computed_bounds.get(&child2_id).unwrap();
    let child2_sub_child1_id_bounds = computed_bounds.get(&child2_sub_child1_id).unwrap();
    let child2_sub_child2_id_bounds = computed_bounds.get(&child2_sub_child2_id).unwrap();
    let child3_id_bounds = computed_bounds.get(&child3_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 125.0);
    assert_eq!(child1_id_bounds.size.height, 300.0);

    assert_eq!(child2_id_bounds.size.width, 250.0);
    assert_eq!(child2_id_bounds.size.height, 300.0);

    assert_eq!(child2_sub_child1_id_bounds.size.width, 125.0);
    assert_eq!(child2_sub_child1_id_bounds.size.height, 280.0);
    assert_eq!(child2_sub_child2_id_bounds.size.width, 250.0);
    assert_eq!(child2_sub_child2_id_bounds.size.height, 20.0);

    assert_eq!(child3_id_bounds.size.width, 125.0);
    assert_eq!(child3_id_bounds.size.height, 300.0);

    // Test child positions - in row layout
    assert_eq!(child1_id_bounds.position.x, 0.0);
    assert_eq!(child1_id_bounds.position.y, 0.0);

    assert_eq!(child2_id_bounds.position.x, 125.0);
    assert_eq!(child2_id_bounds.position.y, 0.0);

    assert_eq!(child2_sub_child1_id_bounds.position.x, 187.5);
    assert_eq!(child2_sub_child1_id_bounds.position.y, 0.0);
    assert_eq!(child2_sub_child2_id_bounds.position.x, 125.0);
    assert_eq!(child2_sub_child2_id_bounds.position.y, 280.0);

    assert_eq!(child3_id_bounds.position.x, 375.0);
    assert_eq!(child3_id_bounds.position.y, 0.0);
}

#[test]
fn test_multiple_containers_with_one_anchored() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000.0,
        height: 800.0,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    // Create parent container with fixed size
    let parent_id = ContainerBuilder::new()
        .with_debug_name("Parent Container")
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .with_justify_content(JustifyContent::SpaceBetween)
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    // Create first child with fixed size
    let child1_id = ContainerBuilder::new()
        .with_debug_name("Child1 Container")
        .with_width(FlexValue::Fraction(0.1))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, child1_id);

    let child2_id = ContainerBuilder::new()
        .with_debug_name("Child2 Container")
        .with_width(FlexValue::Fraction(0.5))
        .with_direction(FlexDirection::Column)
        .with_align_items(AlignItems::Center)
        .with_position(Position::Fixed(Anchor::Center))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, child2_id);

    let child3_id = ContainerBuilder::new()
        .with_debug_name("Child3 Container")
        .with_width(FlexValue::Fraction(0.2))
        .build(&mut ctx.world, &mut ctx.z_index_manager);

    ctx.add_child_to_parent(parent_id, child3_id);

    // Force layout computation
    ctx.find_root_component();
    ctx.compute_layout_and_sync(&mut wgpu_ctx);
    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_id_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_id_bounds = computed_bounds.get(&child2_id).unwrap();
    let child3_id_bounds = computed_bounds.get(&child3_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 0.0);
    assert_eq!(parent_bounds.position.y, 0.0);

    // Test child sizes
    assert_eq!(child1_id_bounds.size.width, 50.0);
    assert_eq!(child1_id_bounds.size.height, 300.0);

    assert_eq!(child2_id_bounds.size.width, 250.0);
    assert_eq!(child2_id_bounds.size.height, 300.0);

    assert_eq!(child3_id_bounds.size.width, 100.0);
    assert_eq!(child3_id_bounds.size.height, 300.0);

    // Test child positions - in row layout
    assert_eq!(child1_id_bounds.position.x, 0.0);
    assert_eq!(child1_id_bounds.position.y, 0.0);

    assert_eq!(child2_id_bounds.position.x, 125.0);
    assert_eq!(child2_id_bounds.position.y, 0.0);

    assert_eq!(child3_id_bounds.position.x, 400.0);
    assert_eq!(child3_id_bounds.position.y, 0.0);
}
