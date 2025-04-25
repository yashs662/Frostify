use crate::{
    ui::{
        components::{component_builder::ComponentBuilder, container::FlexContainerBuilder},
        layout::*,
    },
    wgpu_ctx::WgpuCtx,
};

#[test]
fn test_basic_fixed_flex_row_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Create first child with fixed size
    let child1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut wgpu_ctx);
    let child1_id = child1.id;

    // Create second child with fixed size
    let child2 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut wgpu_ctx);
    let child2_id = child2.id;

    // Set children on parent
    parent.add_child(child1);
    parent.add_child(child2);

    // Add all components to context
    ctx.add_component(parent);

    // Force layout computation
    ctx.compute_layout();

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_bounds.size.width, 100.0);
    assert_eq!(child1_bounds.size.height, 100.0);
    assert_eq!(child2_bounds.size.width, 100.0);
    assert_eq!(child2_bounds.size.height, 100.0);

    // Test child positions - in row layout
    assert_eq!(child1_bounds.position.x, 0.0);
    assert_eq!(child2_bounds.position.x, 100.0);

    // Children should be at the same Y position
    assert_eq!(child1_bounds.position.y, child2_bounds.position.y);
}

#[test]
fn test_basic_fixed_flex_column_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Column)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Create first child with fixed size
    let child1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut wgpu_ctx);
    let child1_id = child1.id;

    // Create second child with fixed size
    let child2 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut wgpu_ctx);
    let child2_id = child2.id;

    // Set children on parent
    parent.add_child(child1);
    parent.add_child(child2);

    // Add all components to context
    ctx.add_component(parent);

    // Force layout computation
    ctx.compute_layout();

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_bounds.size.width, 100.0);
    assert_eq!(child1_bounds.size.height, 100.0);
    assert_eq!(child2_bounds.size.width, 100.0);
    assert_eq!(child2_bounds.size.height, 100.0);

    // Test child positions - in column layout
    assert_eq!(child1_bounds.position.y, 0.0);
    assert_eq!(child2_bounds.position.y, 100.0);

    // Children should be at the same X position
    assert_eq!(child1_bounds.position.x, child2_bounds.position.x);
}

#[test]
fn test_basic_fill_flex_row_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Create first child with fixed size
    let child1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut wgpu_ctx);
    let child1_id = child1.id;

    // Create second child with fill size
    let child2 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut wgpu_ctx);
    let child2_id = child2.id;

    // Set children on parent
    parent.add_child(child1);
    parent.add_child(child2);

    // Add all components to context
    ctx.add_component(parent);

    // Force layout computation
    ctx.compute_layout();

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_bounds.size.width, 100.0);
    assert_eq!(child1_bounds.size.height, 100.0);
    assert_eq!(child2_bounds.size.width, 400.0);
    assert_eq!(child2_bounds.size.height, 300.0);

    // Test child positions - in row layout
    assert_eq!(child1_bounds.position.x, 0.0);
    assert_eq!(child2_bounds.position.x, 100.0);

    // Children should be at the same Y position
    assert_eq!(child1_bounds.position.y, child2_bounds.position.y);
}

#[test]
fn test_basic_fill_flex_column_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Column)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Create first child with fixed size
    let child1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut wgpu_ctx);
    let child1_id = child1.id;

    // Create second child with fill size
    let child2 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut wgpu_ctx);
    let child2_id = child2.id;

    // Set children on parent
    parent.add_child(child1);
    parent.add_child(child2);

    // Add all components to context
    ctx.add_component(parent);

    // Force layout computation
    ctx.compute_layout();

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_bounds.size.width, 100.0);
    assert_eq!(child1_bounds.size.height, 100.0);
    assert_eq!(child2_bounds.size.width, 500.0);
    assert_eq!(child2_bounds.size.height, 200.0);

    // Test child positions - in column layout
    assert_eq!(child1_bounds.position.y, 0.0);
    assert_eq!(child2_bounds.position.y, 100.0);

    // Children should be at the same X position
    assert_eq!(child1_bounds.position.x, child2_bounds.position.x);
}

#[test]
fn test_basic_fill_flex_row_layout_with_padding() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .with_padding(Edges::horizontal(10.0))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Create first child with fixed size
    let child1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut wgpu_ctx);
    let child1_id = child1.id;

    // Create second child with fill size
    let child2 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut wgpu_ctx);
    let child2_id = child2.id;

    // Set children on parent
    parent.add_child(child1);
    parent.add_child(child2);

    // Add all components to context
    ctx.add_component(parent);

    // Force layout computation
    ctx.compute_layout();

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_bounds.size.width, 100.0);
    assert_eq!(child1_bounds.size.height, 100.0);
    assert_eq!(child2_bounds.size.width, 380.0);
    assert_eq!(child2_bounds.size.height, 300.0);

    // Test child positions - in row layout
    assert_eq!(child1_bounds.position.x, 10.0);
    assert_eq!(child2_bounds.position.x, 110.0);

    // Children should be at the same Y position
    assert_eq!(child1_bounds.position.y, child2_bounds.position.y);
}

#[test]
fn test_basic_fill_flex_column_layout_with_padding() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Column)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .with_padding(Edges::vertical(10.0))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Create first child with fixed size
    let child1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut wgpu_ctx);
    let child1_id = child1.id;

    // Create second child with fill size
    let child2 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut wgpu_ctx);
    let child2_id = child2.id;

    // Set children on parent
    parent.add_child(child1);
    parent.add_child(child2);

    // Add all components to context
    ctx.add_component(parent);

    // Force layout computation
    ctx.compute_layout();

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_bounds.size.width, 100.0);
    assert_eq!(child1_bounds.size.height, 100.0);
    assert_eq!(child2_bounds.size.width, 500.0);
    assert_eq!(child2_bounds.size.height, 180.0);

    // Test child positions - in column layout
    assert_eq!(child1_bounds.position.y, 10.0);
    assert_eq!(child2_bounds.position.y, 110.0);

    // Children should be at the same X position
    assert_eq!(child1_bounds.position.x, child2_bounds.position.x);
}

#[test]
fn test_nested_containers_with_flex_layout_fixed_nested_container() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Create first child with fixed size
    let child1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut wgpu_ctx);
    let child1_id = child1.id;

    // Create nested container with fixed size
    let mut nested_parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(200.0))
        .with_height(FlexValue::Fixed(200.0))
        .with_direction(FlexDirection::Row)
        .build(&mut wgpu_ctx);
    let nested_parent_id = nested_parent.id;

    // Create nested child with fixed size
    let nested_child_1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(25.0))
        .with_height(FlexValue::Fixed(25.0))
        .build(&mut wgpu_ctx);
    let nested_child_1_id = nested_child_1.id;

    // Create nested child with fill size
    let nested_child_2 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut wgpu_ctx);
    let nested_child_2_id = nested_child_2.id;

    // Set children on parents
    nested_parent.add_child(nested_child_1);
    nested_parent.add_child(nested_child_2);
    parent.add_child(child1);
    parent.add_child(nested_parent);

    // Add all components to context
    ctx.add_component(parent);

    // Force layout computation
    ctx.compute_layout();

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_bounds = computed_bounds.get(&child1_id).unwrap();
    let nested_parent_bounds = computed_bounds.get(&nested_parent_id).unwrap();
    let nested_child_1_bounds = computed_bounds.get(&nested_child_1_id).unwrap();
    let nested_child_2_bounds = computed_bounds.get(&nested_child_2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_bounds.size.width, 100.0);
    assert_eq!(child1_bounds.size.height, 100.0);
    assert_eq!(nested_parent_bounds.size.width, 200.0);
    assert_eq!(nested_parent_bounds.size.height, 200.0);
    assert_eq!(nested_child_1_bounds.size.width, 25.0);
    assert_eq!(nested_child_1_bounds.size.height, 25.0);
    assert_eq!(nested_child_2_bounds.size.width, 175.0);
    assert_eq!(nested_child_2_bounds.size.height, 200.0);

    // Test child positions - in row layout
    assert_eq!(child1_bounds.position.x, 0.0);
    assert_eq!(nested_parent_bounds.position.x, 100.0);
    assert_eq!(nested_child_1_bounds.position.x, 100.0);
    assert_eq!(nested_child_2_bounds.position.x, 125.0);

    // Children should be at the same Y position
    assert_eq!(child1_bounds.position.y, nested_parent_bounds.position.y);
    assert_eq!(
        nested_child_1_bounds.position.y,
        nested_child_2_bounds.position.y
    );
}

#[test]
fn test_nested_containers_with_flex_layout_fill_nested_container() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Column)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Create first child with fixed size
    let child1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fixed(100.0))
        .build(&mut wgpu_ctx);
    let child1_id = child1.id;

    // Create nested container with fill size
    let mut nested_parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .with_direction(FlexDirection::Row)
        .build(&mut wgpu_ctx);
    let nested_parent_id = nested_parent.id;

    // Create nested child with fixed size
    let nested_child_1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(25.0))
        .with_height(FlexValue::Fixed(25.0))
        .build(&mut wgpu_ctx);
    let nested_child_1_id = nested_child_1.id;

    // Create nested child with fill size
    let nested_child_2 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut wgpu_ctx);
    let nested_child_2_id = nested_child_2.id;

    // Set children on parent
    nested_parent.add_child(nested_child_1);
    nested_parent.add_child(nested_child_2);
    parent.add_child(child1);
    parent.add_child(nested_parent);

    ctx.add_component(parent);

    // Force layout computation
    ctx.compute_layout();

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_bounds = computed_bounds.get(&child1_id).unwrap();
    let nested_parent_bounds = computed_bounds.get(&nested_parent_id).unwrap();
    let nested_child_1_bounds = computed_bounds.get(&nested_child_1_id).unwrap();
    let nested_child_2_bounds = computed_bounds.get(&nested_child_2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_bounds.size.width, 500.0);
    assert_eq!(child1_bounds.size.height, 100.0);
    assert_eq!(nested_parent_bounds.size.width, 500.0);
    assert_eq!(nested_parent_bounds.size.height, 200.0);
    assert_eq!(nested_child_1_bounds.size.width, 25.0);
    assert_eq!(nested_child_1_bounds.size.height, 25.0);
    assert_eq!(nested_child_2_bounds.size.width, 475.0);
    assert_eq!(nested_child_2_bounds.size.height, 200.0);

    // Test child positions - in column layout
    assert_eq!(child1_bounds.position.y, 0.0);
    assert_eq!(nested_parent_bounds.position.y, 100.0);

    // Test nested children positions
    assert_eq!(nested_child_1_bounds.position.x, 0.0);
    assert_eq!(nested_child_2_bounds.position.x, 25.0);

    // Children should be at the same Y position
    assert_eq!(
        nested_child_1_bounds.position.y,
        nested_child_2_bounds.position.y
    );
}

#[test]
fn test_navbar_app_layout() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    ctx.initialize(1000.0, 800.0);

    // Parent container
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .with_direction(FlexDirection::Column)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Parent container background
    let background = FlexContainerBuilder::new()
        .with_position(Position::Absolute(Anchor::TopLeft))
        .build(&mut wgpu_ctx);
    let background_id = background.id;

    // Nav bar container
    let mut nav_bar = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fixed(100.0))
        .with_direction(FlexDirection::Row)
        .with_align_items(AlignItems::Center)
        .with_padding(Edges::all(10.0))
        .build(&mut wgpu_ctx);
    let nav_bar_id = nav_bar.id;

    // Nav bar buttons with fixed size
    let button_size = 24.0;

    // Minimize button
    let minimize_icon = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(button_size))
        .with_height(FlexValue::Fixed(button_size))
        .build(&mut wgpu_ctx);
    let minimize_icon_id = minimize_icon.id;

    // Expand button
    let expand_icon = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(button_size))
        .with_height(FlexValue::Fixed(button_size))
        .build(&mut wgpu_ctx);
    let expand_icon_id = expand_icon.id;

    // Close button
    let close_icon = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(button_size))
        .with_height(FlexValue::Fixed(button_size))
        .build(&mut wgpu_ctx);
    let close_icon_id = close_icon.id;

    // Content container
    let mut content_container = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .with_direction(FlexDirection::Row)
        .build(&mut wgpu_ctx);
    let content_container_id = content_container.id;

    // Text with fixed size
    let text = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(200.0))
        .with_height(FlexValue::Fixed(50.0))
        .build(&mut wgpu_ctx);
    let text_id = text.id;

    // Content image
    let image = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .build(&mut wgpu_ctx);
    let image_id = image.id;

    // Add children to the content container
    content_container.add_child(text);
    content_container.add_child(image);

    // Add children to the nav bar container
    nav_bar.add_child(minimize_icon);
    nav_bar.add_child(expand_icon);
    nav_bar.add_child(close_icon);

    // Add children to the main container
    parent.add_child(background);
    parent.add_child(nav_bar);
    parent.add_child(content_container);

    // Add components in the correct order
    ctx.add_component(parent);

    // Force layout computation
    ctx.compute_layout();

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
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::TopLeft))
        .with_padding(Edges::all(10.0))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Create first child with fixed size
    let child1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .with_margin(Edges::all(10.0))
        .build(&mut wgpu_ctx);
    let child1_id = child1.id;

    // Create second child with fill size
    let child2 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .with_margin(Edges::all(20.0))
        .build(&mut wgpu_ctx);
    let child2_id = child2.id;

    // Set children on parent
    parent.add_child(child1);
    parent.add_child(child2);

    // Add all components to context
    ctx.add_component(parent);

    // Force layout computation
    ctx.compute_layout();

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);

    // Test child sizes
    assert_eq!(child1_bounds.size.width, 100.0);
    assert_eq!(child1_bounds.size.height, 100.0);
    assert_eq!(child2_bounds.size.width, 320.0);
    assert_eq!(child2_bounds.size.height, 240.0);

    // Test child positions - in row layout
    assert_eq!(child1_bounds.position.x, 20.0);
    assert_eq!(child1_bounds.position.y, 20.0);
    assert_eq!(child2_bounds.position.x, 150.0);
    assert_eq!(child2_bounds.position.y, 30.0);
}

#[test]
fn test_fractional_sizing_in_a_container() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::Center))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Create first child with fractional size
    let child1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fraction(0.3))
        .with_height(FlexValue::Fraction(0.5))
        .build(&mut wgpu_ctx);
    let child1_id = child1.id;

    // Create second child with fractional size
    let child2 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fraction(0.7))
        .with_height(FlexValue::Fraction(0.5))
        .build(&mut wgpu_ctx);
    let child2_id = child2.id;

    // Set children on parent
    parent.add_child(child1);
    parent.add_child(child2);

    // Add all components to context
    ctx.add_component(parent);

    // Force layout computation
    ctx.compute_layout();

    let computed_bounds = ctx.get_computed_bounds();

    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_bounds = computed_bounds.get(&child1_id).unwrap();
    let child2_bounds = computed_bounds.get(&child2_id).unwrap();

    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 250.0);
    assert_eq!(parent_bounds.position.y, 250.0);

    // Test child sizes
    assert_eq!(child1_bounds.size.width, 150.0);
    assert_eq!(child1_bounds.size.height, 150.0);
    assert_eq!(child2_bounds.size.width, 350.0);
    assert_eq!(child2_bounds.size.height, 150.0);
    assert_eq!(child1_bounds.position.x, 250.0);
    assert_eq!(child1_bounds.position.y, 250.0);
    assert_eq!(child2_bounds.position.x, 400.0);
    assert_eq!(child2_bounds.position.y, 250.0);
}

#[test]
fn test_offset_in_nested_container() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::Center))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Create first child with fixed size
    let child1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(100.0))
        .with_height(FlexValue::Fixed(100.0))
        .with_offset(ComponentOffset {
            x: FlexValue::Fixed(20.0),
            y: FlexValue::Fixed(20.0),
        })
        .build(&mut wgpu_ctx);
    let child1_id = child1.id;

    // Set children on parent
    parent.add_child(child1);
    // Add all components to context
    ctx.add_component(parent);
    // Force layout computation
    ctx.compute_layout();
    let computed_bounds = ctx.get_computed_bounds();
    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_bounds = computed_bounds.get(&child1_id).unwrap();
    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 250.0);
    assert_eq!(parent_bounds.position.y, 250.0);
    // Test child sizes
    assert_eq!(child1_bounds.size.width, 100.0);
    assert_eq!(child1_bounds.size.height, 100.0);
    assert_eq!(child1_bounds.position.x, 270.0);
    assert_eq!(child1_bounds.position.y, 270.0);
}

#[test]
fn test_offset_in_nested_container_with_flex_value() {
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let mut parent = FlexContainerBuilder::new()
        .with_width(FlexValue::Fixed(500.0))
        .with_height(FlexValue::Fixed(300.0))
        .with_direction(FlexDirection::Row)
        .with_position(Position::Absolute(Anchor::Center))
        .build(&mut wgpu_ctx);
    let parent_id = parent.id;

    // Create first child with fixed size
    let child1 = FlexContainerBuilder::new()
        .with_width(FlexValue::Fill)
        .with_height(FlexValue::Fill)
        .with_offset(ComponentOffset {
            x: FlexValue::Fraction(0.5),
            y: FlexValue::Fraction(0.5),
        })
        .build(&mut wgpu_ctx);
    let child1_id = child1.id;

    // Set children on parent
    parent.add_child(child1);
    // Add all components to context
    ctx.add_component(parent);
    // Force layout computation
    ctx.compute_layout();
    let computed_bounds = ctx.get_computed_bounds();
    // Get computed bounds for all components
    let parent_bounds = computed_bounds.get(&parent_id).unwrap();
    let child1_bounds = computed_bounds.get(&child1_id).unwrap();
    // Test parent bounds
    assert_eq!(parent_bounds.size.width, 500.0);
    assert_eq!(parent_bounds.size.height, 300.0);
    assert_eq!(parent_bounds.position.x, 250.0);
    assert_eq!(parent_bounds.position.y, 250.0);
    // Test child sizes
    assert_eq!(child1_bounds.size.width, 500.0);
    assert_eq!(child1_bounds.size.height, 300.0);
    assert_eq!(child1_bounds.position.x, 500.0);
    assert_eq!(child1_bounds.position.y, 400.0);
}
