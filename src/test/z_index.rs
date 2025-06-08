use crate::{
    test::test_utils::get_event_sender,
    ui::{
        ecs::resources::NamedRefsResource,
        layout::{LayoutContext, Size},
        z_index_manager::ZIndexManager,
    },
    wgpu_ctx::WgpuCtx,
};
use std::cmp::Ordering;
use uuid::Uuid;

#[test]
fn child_ordering() {
    let mut z_index_manager = ZIndexManager::new();
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    let parent_id = Uuid::new_v4();

    z_index_manager.set_root_id(parent_id);
    z_index_manager.register_component(parent_id, None);

    let mut child_ids = Vec::new();
    for _ in 0..10 {
        let child_id = Uuid::new_v4();
        child_ids.push(child_id);
        z_index_manager.register_component(child_id, Some(parent_id));
    }

    let named_refs_resource = ctx
        .world
        .resources
        .get_resource::<NamedRefsResource>()
        .unwrap();
    let render_order = z_index_manager.generate_render_order(named_refs_resource);
    assert_eq!(render_order.len(), 11);
    assert_eq!(render_order[0], parent_id);
    for (i, child_id) in child_ids.iter().enumerate() {
        assert_eq!(render_order[i + 1], *child_id);
    }
}

#[test]
fn custom_z_index() {
    let mut z_index_manager = ZIndexManager::new();
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    let parent_id = Uuid::new_v4();

    z_index_manager.set_root_id(parent_id);
    z_index_manager.register_component(parent_id, None);

    let mut child_ids = Vec::new();
    for i in 0..10 {
        let child_id = Uuid::new_v4();
        child_ids.push(child_id);
        z_index_manager.register_component(child_id, Some(parent_id));
        if i == 5 {
            z_index_manager.set_adjustment(child_id, 100);
        }
    }

    let named_refs_resource = ctx
        .world
        .resources
        .get_resource::<NamedRefsResource>()
        .unwrap();
    let render_order = z_index_manager.generate_render_order(named_refs_resource);
    assert_eq!(render_order.len(), 11);
    assert_eq!(render_order[0], parent_id);

    // All children should be in the order they were added, except for the one with the adjustment which should be at the end
    for (i, child_id) in child_ids.iter().enumerate() {
        match i.cmp(&5) {
            Ordering::Less => assert_eq!(render_order[i + 1], *child_id),
            Ordering::Equal => assert_eq!(render_order.last().unwrap(), child_id),
            Ordering::Greater => assert_eq!(render_order[i], *child_id),
        }
    }
}

#[test]
fn hierarchical_z_index_ordering() {
    let mut z_index_manager = ZIndexManager::new();
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    let parent_id = Uuid::new_v4();

    // Set up root parent
    z_index_manager.set_root_id(parent_id);
    z_index_manager.register_component(parent_id, None);

    // Create two child components
    let child1_id = Uuid::new_v4();
    let child2_id = Uuid::new_v4();

    // Register children in order: child1 first, then child2
    z_index_manager.register_component(child1_id, Some(parent_id));
    z_index_manager.register_component(child2_id, Some(parent_id));

    // Create 5 sub-children for child1
    let mut child1_subchildren = Vec::new();
    for _ in 0..5 {
        let sub_child_id = Uuid::new_v4();
        child1_subchildren.push(sub_child_id);
        z_index_manager.register_component(sub_child_id, Some(child1_id));
    }

    // Create 5 sub-children for child2
    let mut child2_subchildren = Vec::new();
    for _ in 0..5 {
        let sub_child_id = Uuid::new_v4();
        child2_subchildren.push(sub_child_id);
        z_index_manager.register_component(sub_child_id, Some(child2_id));
    }

    // Get render order
    let named_refs_resource = ctx
        .world
        .resources
        .get_resource::<NamedRefsResource>()
        .unwrap();
    let render_order = z_index_manager.generate_render_order(named_refs_resource);
    assert_eq!(render_order.len(), 13);
    assert_eq!(render_order[0], parent_id);
    assert_eq!(render_order[1], child1_id);
    for (i, sub_child_id) in child1_subchildren.iter().enumerate() {
        assert_eq!(render_order[i + 2], *sub_child_id);
    }
    assert_eq!(render_order[7], child2_id);
    for (i, sub_child_id) in child2_subchildren.iter().enumerate() {
        assert_eq!(render_order[i + 8], *sub_child_id);
    }
}

#[test]
fn inverted_hierarchical_z_index_ordering() {
    let mut z_index_manager = ZIndexManager::new();
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    let parent_id = Uuid::new_v4();

    // Set up root parent
    z_index_manager.set_root_id(parent_id);
    z_index_manager.register_component(parent_id, None);

    // Create two child components
    let child1_id = Uuid::new_v4();
    let child2_id = Uuid::new_v4();

    // Register children in order: child1 first, then child2
    z_index_manager.register_component(child1_id, Some(parent_id));
    z_index_manager.register_component(child2_id, Some(parent_id));

    // Create 5 sub-children for child1
    let mut child1_subchildren = Vec::new();
    for _ in 0..5 {
        let sub_child_id = Uuid::new_v4();
        child1_subchildren.push(sub_child_id);
        z_index_manager.register_component(sub_child_id, Some(child1_id));
    }

    // Create 5 sub-children for child2
    let mut child2_subchildren = Vec::new();
    for _ in 0..5 {
        let sub_child_id = Uuid::new_v4();
        child2_subchildren.push(sub_child_id);
        z_index_manager.register_component(sub_child_id, Some(child2_id));
    }

    // Apply negative adjustment to child2, this should make it render before child1 even though it was registered after
    // This is a negative adjustment, so child2 and its sub-children should render before child1 and its sub-children
    z_index_manager.set_adjustment(child2_id, -1);

    // Get render order
    let named_refs_resource = ctx
        .world
        .resources
        .get_resource::<NamedRefsResource>()
        .unwrap();
    let render_order = z_index_manager.generate_render_order(named_refs_resource);
    assert_eq!(render_order.len(), 13);
    assert_eq!(render_order[0], parent_id);
    assert_eq!(render_order[1], child2_id);
    for (i, sub_child_id) in child2_subchildren.iter().enumerate() {
        assert_eq!(render_order[i + 2], *sub_child_id);
    }
    assert_eq!(render_order[7], child1_id);
    for (i, sub_child_id) in child1_subchildren.iter().enumerate() {
        assert_eq!(render_order[i + 8], *sub_child_id);
    }
}

#[test]
fn multiple_adjustments() {
    let mut z_index_manager = ZIndexManager::new();
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    let parent_id = Uuid::new_v4();

    // Set up root parent
    z_index_manager.set_root_id(parent_id);
    z_index_manager.register_component(parent_id, None);

    // Create child components
    let child1_id = Uuid::new_v4();
    let child2_id = Uuid::new_v4();
    let child3_id = Uuid::new_v4();
    let child4_id = Uuid::new_v4();
    let child5_id = Uuid::new_v4();
    let child6_id = Uuid::new_v4();

    // Register children in order: child1 first, then child2
    z_index_manager.register_component(child1_id, Some(parent_id));
    z_index_manager.register_component(child2_id, Some(parent_id));
    z_index_manager.register_component(child3_id, Some(parent_id));
    z_index_manager.register_component(child4_id, Some(parent_id));
    z_index_manager.register_component(child5_id, Some(parent_id));
    z_index_manager.register_component(child6_id, Some(parent_id));

    // Apply adjustments
    z_index_manager.set_adjustment(child1_id, 1);
    z_index_manager.set_adjustment(child2_id, 2);
    z_index_manager.set_adjustment(child3_id, 3);
    z_index_manager.set_adjustment(child4_id, -1);
    z_index_manager.set_adjustment(child5_id, 4);
    z_index_manager.set_adjustment(child6_id, -2);

    // Get render order
    let named_refs_resource = ctx
        .world
        .resources
        .get_resource::<NamedRefsResource>()
        .unwrap();
    let render_order = z_index_manager.generate_render_order(named_refs_resource);
    assert_eq!(render_order.len(), 7);
    assert_eq!(render_order[0], parent_id);
    assert_eq!(render_order[1], child6_id);
    assert_eq!(render_order[2], child4_id);
    assert_eq!(render_order[3], child1_id);
    assert_eq!(render_order[4], child2_id);
    assert_eq!(render_order[5], child3_id);
    assert_eq!(render_order[6], child5_id);
}

#[test]
fn multiple_adjustments_in_hierarchy() {
    let mut z_index_manager = ZIndexManager::new();
    let mut ctx = LayoutContext::default();
    let mut wgpu_ctx = pollster::block_on(WgpuCtx::new_noop());
    let viewport_size = Size {
        width: 1000,
        height: 800,
    };
    ctx.initialize(viewport_size, &mut wgpu_ctx, &get_event_sender());

    let parent_id = Uuid::new_v4();

    // Set up root parent
    z_index_manager.set_root_id(parent_id);
    z_index_manager.register_component(parent_id, None);

    // Create child components
    let child1_id = Uuid::new_v4();
    let child2_id = Uuid::new_v4();

    // Register children in order: child1 first, then child2
    z_index_manager.register_component(child1_id, Some(parent_id));
    z_index_manager.register_component(child2_id, Some(parent_id));

    // Apply adjustments
    z_index_manager.set_adjustment(child1_id, 1);
    z_index_manager.set_adjustment(child2_id, -1);

    // Create 5 sub-children for child1
    let child1_subchild_1 = Uuid::new_v4();
    let child1_subchild_2 = Uuid::new_v4();
    let child1_subchild_3 = Uuid::new_v4();
    let child1_subchild_4 = Uuid::new_v4();
    let child1_subchild_5 = Uuid::new_v4();

    // Register child1's sub-children
    z_index_manager.register_component(child1_subchild_1, Some(child1_id));
    z_index_manager.register_component(child1_subchild_2, Some(child1_id));
    z_index_manager.register_component(child1_subchild_3, Some(child1_id));
    z_index_manager.register_component(child1_subchild_4, Some(child1_id));
    z_index_manager.register_component(child1_subchild_5, Some(child1_id));

    // Apply adjustments to child1's sub-children
    z_index_manager.set_adjustment(child1_subchild_1, 1);
    z_index_manager.set_adjustment(child1_subchild_2, 2);
    z_index_manager.set_adjustment(child1_subchild_3, 3);
    z_index_manager.set_adjustment(child1_subchild_4, -1);
    z_index_manager.set_adjustment(child1_subchild_5, 4);

    // Create 5 sub-children for child2
    let child2_subchild_1 = Uuid::new_v4();
    let child2_subchild_2 = Uuid::new_v4();
    let child2_subchild_3 = Uuid::new_v4();
    let child2_subchild_4 = Uuid::new_v4();
    let child2_subchild_5 = Uuid::new_v4();

    // Register child2's sub-children
    z_index_manager.register_component(child2_subchild_1, Some(child2_id));
    z_index_manager.register_component(child2_subchild_2, Some(child2_id));
    z_index_manager.register_component(child2_subchild_3, Some(child2_id));
    z_index_manager.register_component(child2_subchild_4, Some(child2_id));
    z_index_manager.register_component(child2_subchild_5, Some(child2_id));

    // Apply adjustments to child2's sub-children
    z_index_manager.set_adjustment(child2_subchild_1, 0);
    z_index_manager.set_adjustment(child2_subchild_2, -4);
    z_index_manager.set_adjustment(child2_subchild_3, 5);
    z_index_manager.set_adjustment(child2_subchild_4, 2);
    z_index_manager.set_adjustment(child2_subchild_5, -1);

    // Get render order
    let named_refs_resource = ctx
        .world
        .resources
        .get_resource::<NamedRefsResource>()
        .unwrap();
    let render_order = z_index_manager.generate_render_order(named_refs_resource);
    assert_eq!(render_order.len(), 13);
    assert_eq!(render_order[0], parent_id);
    assert_eq!(render_order[1], child2_id);
    assert_eq!(render_order[2], child2_subchild_2);
    assert_eq!(render_order[3], child2_subchild_5);
    assert_eq!(render_order[4], child2_subchild_1);
    assert_eq!(render_order[5], child2_subchild_4);
    assert_eq!(render_order[6], child2_subchild_3);
    assert_eq!(render_order[7], child1_id);
    assert_eq!(render_order[8], child1_subchild_4);
    assert_eq!(render_order[9], child1_subchild_1);
    assert_eq!(render_order[10], child1_subchild_2);
    assert_eq!(render_order[11], child1_subchild_3);
    assert_eq!(render_order[12], child1_subchild_5);
}
