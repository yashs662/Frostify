use crate::{
    constants::UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
    ui::{
        ecs::{
            components::{BoundsComponent, RenderDataComponent, TextComponent},
            resources::{RenderOrderResource, ViewportResource, WgpuQueueResource},
            systems::create_component_buffer_data,
        },
        layout::LayoutContext,
        text_renderer::OptionalTextUpdateData,
    },
    wgpu_ctx::WgpuCtx,
};

use super::{EntityId, World, systems::AnimationSystem};

// Hybrid update method that runs ECS systems first, then updates traditional components
pub fn hybrid_update(layout_context: &mut LayoutContext, frame_time: f32) {
    // Run ECS systems directly on the world contained in layout_context
    {
        let world = &mut layout_context.world;
        world
            .run_system::<crate::ui::ecs::systems::AnimationSystem>(AnimationSystem { frame_time });
    }
}

// TODO: Reduce the number of syncs as much as possible
pub fn sync_computed_bounds_and_screen_size(
    layout_context: &mut LayoutContext,
    wgpu_ctx: &mut WgpuCtx,
) {
    layout_context
        .world
        .for_each_component_mut::<BoundsComponent, _>(|id, bounds_comp| {
            if let Some(computed_bounds) = layout_context.computed_bounds.get(&id) {
                bounds_comp.computed_bounds = *computed_bounds;
                bounds_comp.screen_size = layout_context.viewport_size;
            }
        });

    let device_queue = layout_context
        .world
        .resources
        .get_resource::<WgpuQueueResource>()
        .expect("expected WgpuQueueResource to exist")
        .clone();

    layout_context
        .world
        .for_each_component::<RenderDataComponent, _>(|id, render_data_comp| {
            device_queue.queue.write_buffer(
                render_data_comp
                    .render_data_buffer
                    .as_ref()
                    .expect("RenderDataComponent should have a valid render data buffer"),
                0,
                bytemuck::cast_slice(&[create_component_buffer_data(&layout_context.world, id)]),
            );
        });

    layout_context
        .world
        .for_each_component::<TextComponent, _>(|id, text_comp| {
            if let Some(text_bounds) = layout_context.computed_bounds.get(&id) {
                wgpu_ctx
                    .text_handler
                    .update((id, OptionalTextUpdateData::new().with_bounds(*text_bounds)));
            }
        });

    log::debug!("Syncing computed bounds and screen size");
}

// Update viewport resource in ECS when viewport changes
pub fn update_global_viewport_resource(layout_context: &mut LayoutContext) {
    let viewport_resource = layout_context
        .world
        .resources
        .get_resource_mut::<ViewportResource>()
        .expect("expected ViewportResource to exist");
    {
        viewport_resource.size = layout_context.viewport_size;
    }
}

pub fn update_frosted_glass_with_frame_texture(
    world: &mut World,
    entity_id: EntityId,
    frame_texture_view: &wgpu::TextureView,
    device: &wgpu::Device,
) {
    // let render_data_buffer = match component.get_render_data_buffer() {
    //     Some(buffer) => buffer,
    //     None => {
    //         error!("No render data buffer found for frosted glass component");
    //         return false;
    //     }
    // };

    let render_comp = world
        .components
        .get_component_mut::<RenderDataComponent>(entity_id)
        .expect("expected RenderDataComponent to exist to update frosted glass");

    let sampler = render_comp
        .sampler
        .as_ref()
        .expect("expected sampler to exist in render_comp for frosted glass");

    // Create unified bind group layout compatible with the shader
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: UNIFIED_BIND_GROUP_LAYOUT_ENTRIES,
        label: Some(format!("{} Unified Bind Group Layout", entity_id).as_str()),
    });

    // Create bind group with all required resources
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            // Component uniform data
            wgpu::BindGroupEntry {
                binding: 0,
                resource: render_comp.render_data_buffer.as_ref()
                    .expect("expected render data buffer to exist for updating frame texture to frosted glass")
                    .as_entire_binding(),
            },
            // Texture view
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(frame_texture_view),
            },
            // Sampler
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
        label: Some(format!("{} Unified Bind Group", entity_id).as_str()),
    });

    render_comp.bind_group = Some(bind_group);
}

// Sync render order from LayoutContext to ECS
pub fn sync_render_order(layout_context: &mut LayoutContext) {
    // Get the render order from the layout context
    let render_order = layout_context.get_render_order().clone();

    // Update or create the render order resource in the world
    let render_order_resource = layout_context
        .world
        .resources
        .get_resource_mut::<RenderOrderResource>()
        .expect("expected RenderOrderResource to exist");
    render_order_resource.render_order = render_order;
}

// Hybrid draw method that avoids multiple mutable borrows
pub fn hybrid_draw(layout_context: &mut LayoutContext, wgpu_ctx: &mut crate::wgpu_ctx::WgpuCtx) {
    // Sync the render order from layout context to ECS world
    sync_render_order(layout_context);

    // Draw using ECS system
    wgpu_ctx.draw(&mut layout_context.world);
}
