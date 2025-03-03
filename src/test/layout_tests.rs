use crate::ui::{
    components::core::component::{Component, ComponentType},
    layout::*,
};
use uuid::Uuid;

#[test]
fn test_basic_fixed_flex_row_layout() {
    let mut ctx = LayoutContext::default();
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let parent_id = Uuid::new_v4();
    let mut parent = Component::new(parent_id, ComponentType::Container);
    parent.transform.position_type = Position::Absolute(Anchor::TopLeft);
    parent.transform.size = Size::fixed(500.0, 300.0);
    parent.layout = Layout::flex_row();

    // Create first child with fixed size
    let child1_id = Uuid::new_v4();
    let mut child1 = Component::new(child1_id, ComponentType::Container);
    child1.transform.size = Size::fixed(100.0, 100.0);
    child1.set_parent(parent_id);

    // Create second child with fixed size
    let child2_id = Uuid::new_v4();
    let mut child2 = Component::new(child2_id, ComponentType::Container);
    child2.transform.size = Size::fixed(100.0, 100.0);
    child2.set_parent(parent_id);

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
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let parent_id = Uuid::new_v4();
    let mut parent = Component::new(parent_id, ComponentType::Container);
    parent.transform.position_type = Position::Absolute(Anchor::TopLeft);
    parent.transform.size = Size::fixed(500.0, 300.0);
    parent.layout = Layout::flex_column();

    // Create first child with fixed size
    let child1_id = Uuid::new_v4();
    let mut child1 = Component::new(child1_id, ComponentType::Container);
    child1.transform.size = Size::fixed(100.0, 100.0);
    child1.set_parent(parent_id);

    // Create second child with fixed size
    let child2_id = Uuid::new_v4();
    let mut child2 = Component::new(child2_id, ComponentType::Container);
    child2.transform.size = Size::fixed(100.0, 100.0);
    child2.set_parent(parent_id);

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
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let parent_id = Uuid::new_v4();
    let mut parent = Component::new(parent_id, ComponentType::Container);
    parent.transform.position_type = Position::Absolute(Anchor::TopLeft);
    parent.transform.size = Size::fixed(500.0, 300.0);
    parent.layout = Layout::flex_row();

    // Create first child with fixed size
    let child1_id = Uuid::new_v4();
    let mut child1 = Component::new(child1_id, ComponentType::Container);
    child1.transform.size = Size::fixed(100.0, 100.0);
    child1.set_parent(parent_id);

    // Create second child with fill size
    let child2_id = Uuid::new_v4();
    let mut child2 = Component::new(child2_id, ComponentType::Container);
    child2.transform.size = Size::fill();
    child2.set_parent(parent_id);

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
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let parent_id = Uuid::new_v4();
    let mut parent = Component::new(parent_id, ComponentType::Container);
    parent.transform.position_type = Position::Absolute(Anchor::TopLeft);
    parent.transform.size = Size::fixed(500.0, 300.0);
    parent.layout = Layout::flex_column();

    // Create first child with fixed size
    let child1_id = Uuid::new_v4();
    let mut child1 = Component::new(child1_id, ComponentType::Container);
    child1.transform.size = Size::fixed(100.0, 100.0);
    child1.set_parent(parent_id);

    // Create second child with fill size
    let child2_id = Uuid::new_v4();
    let mut child2 = Component::new(child2_id, ComponentType::Container);
    child2.transform.size = Size::fill();
    child2.set_parent(parent_id);

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
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let parent_id = Uuid::new_v4();
    let mut parent = Component::new(parent_id, ComponentType::Container);
    parent.transform.position_type = Position::Absolute(Anchor::TopLeft);
    parent.transform.size = Size::fixed(500.0, 300.0);
    parent.layout = Layout::flex_row();
    parent.layout.with_padding(Edges::horizontal(10.0));

    // Create first child with fixed size
    let child1_id = Uuid::new_v4();
    let mut child1 = Component::new(child1_id, ComponentType::Container);
    child1.transform.size = Size::fixed(100.0, 100.0);
    child1.set_parent(parent_id);

    // Create second child with fill size
    let child2_id = Uuid::new_v4();
    let mut child2 = Component::new(child2_id, ComponentType::Container);
    child2.transform.size = Size::fill();
    child2.set_parent(parent_id);

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
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let parent_id = Uuid::new_v4();
    let mut parent = Component::new(parent_id, ComponentType::Container);
    parent.transform.position_type = Position::Absolute(Anchor::TopLeft);
    parent.transform.size = Size::fixed(500.0, 300.0);
    parent.layout = Layout::flex_column();
    parent.layout.with_padding(Edges::vertical(10.0));

    // Create first child with fixed size
    let child1_id = Uuid::new_v4();
    let mut child1 = Component::new(child1_id, ComponentType::Container);
    child1.transform.size = Size::fixed(100.0, 100.0);
    child1.set_parent(parent_id);

    // Create second child with fill size
    let child2_id = Uuid::new_v4();
    let mut child2 = Component::new(child2_id, ComponentType::Container);
    child2.transform.size = Size::fill();
    child2.set_parent(parent_id);

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
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let parent_id = Uuid::new_v4();
    let mut parent = Component::new(parent_id, ComponentType::Container);
    parent.transform.position_type = Position::Absolute(Anchor::TopLeft);
    parent.transform.size = Size::fixed(500.0, 300.0);
    parent.layout = Layout::flex_row();

    // Create first child with fixed size
    let child1_id = Uuid::new_v4();
    let mut child1 = Component::new(child1_id, ComponentType::Container);
    child1.transform.size = Size::fixed(100.0, 100.0);
    child1.set_parent(parent_id);

    // Create nested container with fixed size
    let nested_parent_id = Uuid::new_v4();
    let mut nested_parent = Component::new(nested_parent_id, ComponentType::Container);
    nested_parent.transform.size = Size::fixed(200.0, 200.0);
    nested_parent.layout = Layout::flex_row();
    nested_parent.set_parent(parent_id);

    // Create nested child with fixed size
    let nested_child_1_id = Uuid::new_v4();
    let mut nested_child_1 = Component::new(nested_child_1_id, ComponentType::Container);
    nested_child_1.transform.size = Size::fixed(25.0, 25.0);
    nested_child_1.set_parent(nested_parent_id);

    // Create nested child with fill size
    let nested_child_2_id = Uuid::new_v4();
    let mut nested_child_2 = Component::new(nested_child_2_id, ComponentType::Container);
    nested_child_2.transform.size = Size::fill();
    nested_child_2.set_parent(nested_parent_id);

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
    ctx.initialize(1000.0, 800.0);

    // Create parent container with fixed size
    let parent_id = Uuid::new_v4();
    let mut parent = Component::new(parent_id, ComponentType::Container);
    parent.transform.position_type = Position::Absolute(Anchor::TopLeft);
    parent.transform.size = Size::fixed(500.0, 300.0);
    parent.layout = Layout::flex_column();

    // Create first child with fixed size
    let child1_id = Uuid::new_v4();
    let mut child1 = Component::new(child1_id, ComponentType::Container);
    child1.transform.size.width = FlexValue::Fill;
    child1.transform.size.height = FlexValue::Fixed(100.0);
    child1.set_parent(parent_id);

    // Create nested container with fill size
    let nested_parent_id = Uuid::new_v4();
    let mut nested_parent = Component::new(nested_parent_id, ComponentType::Container);
    nested_parent.transform.size = Size::fill();
    nested_parent.layout = Layout::flex_row();
    nested_parent.set_parent(parent_id);

    // Create nested child with fixed size
    let nested_child_1_id = Uuid::new_v4();
    let mut nested_child_1 = Component::new(nested_child_1_id, ComponentType::Container);
    nested_child_1.transform.size = Size::fixed(25.0, 25.0);
    nested_child_1.set_parent(nested_parent_id);

    // Create nested child with fill size
    let nested_child_2_id = Uuid::new_v4();
    let mut nested_child_2 = Component::new(nested_child_2_id, ComponentType::Container);
    nested_child_2.transform.size = Size::fill();
    nested_child_2.set_parent(nested_parent_id);

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
    ctx.initialize(1000.0, 800.0);

    // parent container
    let parent_id = uuid::Uuid::new_v4();
    let mut parent = Component::new(parent_id, ComponentType::Container);
    parent.transform.position_type = Position::Absolute(Anchor::TopLeft);
    parent.layout = Layout::flex_column();

    // parent container background
    let background_id = uuid::Uuid::new_v4();
    let mut background = Component::new(background_id, ComponentType::BackgroundColor);
    background.transform.position_type = Position::Absolute(Anchor::TopLeft);
    background.set_parent(parent_id);

    // Nav bar container
    let nav_bar_id = uuid::Uuid::new_v4();
    let mut nav_bar = Component::new(nav_bar_id, ComponentType::Container);
    nav_bar.transform.size.height = FlexValue::Fixed(100.0);
    nav_bar.layout = Layout::flex_row();
    nav_bar.layout.align_items = AlignItems::Center;
    nav_bar.layout.padding = Edges::all(10.0);
    nav_bar.set_parent(parent_id);

    // Nav bar buttons with fixed size
    let button_size = 24.0;

    // Minimize button
    let minimize_icon_id = uuid::Uuid::new_v4();
    let mut minimize_icon = Component::new(minimize_icon_id, ComponentType::Image);
    minimize_icon.transform.size.width = FlexValue::Fixed(button_size);
    minimize_icon.transform.size.height = FlexValue::Fixed(button_size);
    minimize_icon.set_parent(nav_bar_id);

    // Expand button
    let expand_icon_id = uuid::Uuid::new_v4();
    let mut expand_icon = Component::new(expand_icon_id, ComponentType::Image);
    expand_icon.transform.size.width = FlexValue::Fixed(button_size);
    expand_icon.transform.size.height = FlexValue::Fixed(button_size);
    expand_icon.set_parent(nav_bar_id);

    // Close button
    let close_icon_id = uuid::Uuid::new_v4();
    let mut close_icon = Component::new(close_icon_id, ComponentType::Image);
    close_icon.transform.size.width = FlexValue::Fixed(button_size);
    close_icon.transform.size.height = FlexValue::Fixed(button_size);
    close_icon.set_parent(nav_bar_id);

    // Content container
    let content_container_id = uuid::Uuid::new_v4();
    let mut content_container = Component::new(content_container_id, ComponentType::Container);
    content_container.layout = Layout::flex_row();
    content_container.set_parent(parent_id);

    // text with fixed size
    let text_id = uuid::Uuid::new_v4();
    let mut text = Component::new(text_id, ComponentType::Text);
    text.transform.size.width = FlexValue::Fixed(200.0); // Fixed width
    text.transform.size.height = FlexValue::Fixed(50.0); // Fixed height
    text.set_parent(content_container_id);

    // Content image
    let image_id = uuid::Uuid::new_v4();
    let mut image = Component::new(image_id, ComponentType::Image);
    image.set_parent(content_container_id);

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
