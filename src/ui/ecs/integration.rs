use crate::ui::{
    component::{Component, ComponentConfig, ComponentType},
    ecs::{
        EntityId, World,
        components::{
            AnimationComponent, BoundsComponent, FrostedGlassComponent, HierarchyComponent,
            IdentityComponent, InteractionComponent, LayoutComponent, RenderDataComponent,
            SliderComponent, TransformComponent, VisualComponent,
        },
        resources::{RenderOrderResource, ViewportResource},
    },
    layout::LayoutContext,
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
            border_color: component.border_color,
            border_position: component.border_position,
            shadow_color: component.shadow_color,
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

    // Add render data component with GPU resources
    world.add_component(
        entity_id,
        RenderDataComponent {
            render_data_buffer: component.get_render_data_buffer().cloned(),
            bind_group: component.get_bind_group().cloned(),
            sampler: component.get_sampler().cloned(),
        },
    );

    // If it's a frosted glass component, add specialized FrostedGlassComponent
    if component.component_type == ComponentType::FrostedGlass {
        if let Some(ComponentConfig::FrostedGlass(config)) = &component.config {
            world.add_component(
                entity_id,
                FrostedGlassComponent {
                    tint_color: config.tint_color,
                    blur_radius: config.blur_radius,
                    opacity: config.opacity,
                    tint_intensity: config.tint_intensity,
                    needs_frame_update: true,
                },
            );
        }
    }

    // If it's a slider component, add SliderComponent data
    if component.is_a_slider() {
        if let Some(slider_data) = component.get_slider_data() {
            world.add_component(
                entity_id,
                SliderComponent {
                    value: slider_data.value,
                    min: slider_data.min,
                    max: slider_data.max,
                    step: slider_data.step,
                    thumb_id: slider_data.thumb_id,
                    track_fill_id: slider_data.track_fill_id,
                    track_bounds: component.get_track_bounds(),
                    needs_update: component.needs_update(),
                    is_dragging: slider_data.is_dragging,
                },
            );
        }
    }

    entity_id
}

pub fn convert_layout_context_to_world(layout_context: &mut LayoutContext) {
    // Add resources
    layout_context.world.add_resource(ViewportResource {
        size: layout_context.viewport_size,
    });

    // Convert all components
    for (id, component) in layout_context.components.iter() {
        convert_component_to_entity(component, &mut layout_context.world);
    }
}

// Hybrid update method that runs ECS systems first, then updates traditional components
pub fn hybrid_update(
    layout_context: &mut LayoutContext,
    wgpu_ctx: &mut crate::wgpu_ctx::WgpuCtx,
    frame_time: f32,
) {
    // Run ECS systems directly on the world contained in layout_context
    {
        let world = &mut layout_context.world;
        world.run_system(crate::ui::ecs::systems::AnimationSystem { frame_time });
    }

    // First collect animation data from ECS in a separate step
    let mut component_updates = Vec::new();
    {
        let world = &layout_context.world;
        for (entity_id, anim_comp) in world.query::<AnimationComponent>() {
            if anim_comp.needs_update {
                if let Some(transform) = world.get_component::<TransformComponent>(entity_id) {
                    component_updates.push((entity_id, transform.scale_factor));
                }
            }
        }
    }

    // Then apply the updates to components in a separate step
    for (entity_id, scale_factor) in component_updates {
        if let Some(component) = layout_context.get_component_mut(&entity_id) {
            component.transform.scale_factor = scale_factor;
            component.update(wgpu_ctx, frame_time);
        }
    }

    // Cleanup any removed entities
    layout_context.world.cleanup();
}

// Sync computed bounds from LayoutContext to ECS
pub fn sync_computed_bounds(layout_context: &mut LayoutContext) {
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

// Update viewport resource in ECS when viewport changes
pub fn update_global_viewport_resource(layout_context: &mut LayoutContext) {
    if let Some(viewport_resource) = layout_context.world.get_resource_mut::<ViewportResource>() {
        viewport_resource.size = layout_context.viewport_size;
    }
}

// Sync render order from LayoutContext to ECS
pub fn sync_render_order(layout_context: &mut LayoutContext) {
    // Get the render order from the layout context
    let render_order = layout_context.get_render_order().clone();

    // Update or create the render order resource in the world
    layout_context
        .world
        .add_resource(RenderOrderResource { render_order });
}

// Hybrid draw method that avoids multiple mutable borrows
pub fn hybrid_draw(layout_context: &mut LayoutContext, wgpu_ctx: &mut crate::wgpu_ctx::WgpuCtx) {
    // First ensure all render data is synced within layout_context
    {
        for (id, component) in &layout_context.components {
            if let Some(render_data_comp) = layout_context
                .world
                .get_component_mut::<RenderDataComponent>(*id)
            {
                // Update render data if component has GPU resources
                render_data_comp.render_data_buffer = component.get_render_data_buffer().cloned();
                render_data_comp.bind_group = component.get_bind_group().cloned();
                render_data_comp.sampler = component.get_sampler().cloned();
            }
        }
    }

    // Sync the render order from layout context to ECS world
    sync_render_order(layout_context);

    // Draw using ECS system
    wgpu_ctx.draw(&mut layout_context.world);
}
