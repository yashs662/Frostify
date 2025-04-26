use crate::ui::{
    component::{Component, ComponentType},
    ecs::{
        EntityId, World,
        components::{
            AnimationComponent, BoundsComponent, HierarchyComponent, IdentityComponent,
            InteractionComponent, LayoutComponent, RenderDataComponent, SliderComponent,
            TransformComponent, VisualComponent,
        },
    },
};

pub fn convert_component_to_entity(component: &Component, world: &mut World) -> EntityId {
    let entity_id = component.id;

    // Create entity with the same UUID
    world.create_entity();

    // Add identity component
    world.add_component(
        entity_id,
        IdentityComponent {
            id: entity_id,
            debug_name: component.debug_name.clone(),
            component_type: component.component_type,
        },
    );

    // Add transform component
    world.add_component(
        entity_id,
        TransformComponent {
            size: component.transform.size.clone(),
            offset: component.transform.offset.clone(),
            position_type: component.transform.position_type,
            z_index: component.transform.z_index,
            border_radius: component.transform.border_radius,
            max_scale_factor: component.transform.max_scale_factor,
            min_scale_factor: component.transform.min_scale_factor,
            scale_factor: component.transform.scale_factor,
            scale_anchor: component.transform.scale_anchor,
        },
    );

    // Add layout component
    world.add_component(
        entity_id,
        LayoutComponent {
            layout: component.layout.clone(),
        },
    );

    // Add hierarchy component
    world.add_component(
        entity_id,
        HierarchyComponent {
            parent: component.get_parent_id(),
            children: component.get_all_children_ids(),
        },
    );

    // Add visual component
    world.add_component(
        entity_id,
        VisualComponent {
            component_type: component.component_type,
            border_width: component.border_width,
            border_color: component.border_color.clone(),
            border_position: component.border_position,
            shadow_color: component.shadow_color.clone(),
            shadow_offset: component.shadow_offset,
            shadow_blur: component.shadow_blur,
            shadow_opacity: component.shadow_opacity,
            is_visible: component.layout.visible,
        },
    );

    // Add bounds component
    world.add_component(
        entity_id,
        BoundsComponent {
            computed_bounds: component.computed_bounds,
            screen_size: component.screen_size,
            clip_bounds: component.clip_bounds,
            clip_self: component.clip_self,
        },
    );

    // Add interaction component
    world.add_component(
        entity_id,
        InteractionComponent {
            is_clickable: component.is_clickable(),
            is_draggable: component.is_draggable(),
            is_hoverable: component.is_hoverable(),
            is_hovered: component.is_hovered(),
            click_event: component.get_click_event().cloned(),
            drag_event: component.get_drag_event().cloned(),
        },
    );

    // Add animation component
    world.add_component(
        entity_id,
        AnimationComponent {
            animations: component.animations.clone(),
            needs_update: component.needs_update(),
        },
    );

    // Add render data component
    world.add_component(
        entity_id,
        RenderDataComponent {
            render_data_buffer: component.get_render_data_buffer().cloned(),
            bind_group: component.get_bind_group().cloned(),
            sampler: component.get_sampler().cloned(),
        },
    );

    entity_id
}

pub fn convert_layout_context_to_world(layout_context: &mut crate::ui::layout::LayoutContext) {
    // Add resources
    layout_context
        .world
        .add_resource(crate::ui::ecs::resources::ViewportResource {
            size: layout_context.viewport_size,
        });

    // Convert all components
    for (id, component) in layout_context.components.iter() {
        convert_component_to_entity(component, &mut layout_context.world);
    }
}

// Gradually migrate systems but keep current layout functionality
pub fn hybrid_update(
    layout_context: &mut crate::ui::layout::LayoutContext,
    world: &mut World,
    wgpu_ctx: &mut crate::wgpu_ctx::WgpuCtx,
    frame_time: f32,
) {
    // Run ECS systems first
    world.run_system(crate::ui::ecs::systems::AnimationSystem { frame_time });

    // Update traditional components using data from ECS
    for (entity_id, anim_comp) in world.query::<AnimationComponent>() {
        if anim_comp.needs_update {
            if let Some(component) = layout_context.get_component_mut(&entity_id) {
                // Sync data from ECS back to component
                if let Some(transform) = world.get_component::<TransformComponent>(entity_id) {
                    component.transform.scale_factor = transform.scale_factor;
                }

                // Call traditional update which will handle GPU updates
                component.update(wgpu_ctx, frame_time);
            }
        }
    }

    // Cleanup any removed entities
    world.cleanup();
}

pub fn sync_computed_bounds(layout_context: &mut crate::ui::layout::LayoutContext) {
    // Sync bounds back to ECS
    for (id, bounds) in layout_context.computed_bounds.iter() {
        if let Some(bounds_comp) = layout_context
            .world
            .get_component_mut::<BoundsComponent>(*id)
        {
            bounds_comp.computed_bounds = *bounds;
        }
    }
}

pub fn update_global_viewport_resource(layout_context: &mut crate::ui::layout::LayoutContext) {
    if let Some(viewport_resource) = layout_context
        .world
        .get_resource_mut::<crate::ui::ecs::resources::ViewportResource>()
    {
        viewport_resource.size = layout_context.viewport_size;
    }
}
