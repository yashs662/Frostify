use crate::{
    constants::SCROLL_MULTIPLIER,
    ui::{
        ecs::{
            EcsSystem, World,
            components::*,
            resources::{MouseResource, RenderOrderResource, RequestReLayoutResource},
        },
        layout::{Bounds, ClipBounds, ComponentPosition},
    },
};
use frostify_derive::time_system;

pub struct MouseHoverSystem;

impl EcsSystem for MouseHoverSystem {
    fn run(&mut self, world: &mut World) {
        // Get the mouse position from the resource
        let mouse_resource = world
            .resources
            .get_resource::<MouseResource>()
            .expect("Expected MouseResource to be present");

        let interactive_entities = world
            .components
            .query_combined_2::<BoundsComponent, InteractionComponent>();

        let mut hovered_entities = Vec::new();
        let mut dragged_entities = Vec::new();

        for (entity_id, bounds_comp, interaction_comp) in interactive_entities {
            // Check if the entity is active
            if interaction_comp.is_active
                && is_hit(
                    bounds_comp.computed_bounds,
                    bounds_comp.clip_bounds,
                    mouse_resource.position,
                )
            {
                if interaction_comp.is_hoverable {
                    hovered_entities.push(entity_id);
                }

                if mouse_resource.is_dragging && interaction_comp.is_draggable {
                    dragged_entities.push((
                        entity_id,
                        interaction_comp
                            .drag_event
                            .expect("expected draggable entity to have drag event"),
                    ));
                }
            }
        }

        world.for_each_component_mut::<InteractionComponent, _>(|id, interaction_comp| {
            interaction_comp.is_hovered = hovered_entities.contains(&id);
        });

        if !dragged_entities.is_empty() {
            let render_order_resource = world
                .resources
                .get_resource::<RenderOrderResource>()
                .expect(
                    "Expected RenderOrderResource to be present, while trying to get render order",
                );

            // Get the entity with the highest z index
            let (entity_id, drag_event) = dragged_entities
                .iter()
                .max_by_key(|(id, _)| {
                    render_order_resource
                        .render_order
                        .iter()
                        .position(|e| e == id)
                        .unwrap_or(usize::MAX)
                })
                .expect("Expected at least one entity to be dragged");

            // Send the drag event
            log::debug!(
                "Sending drag event: {drag_event:?} for entity: {entity_id}"
            );
            world.queue_event(*drag_event);
        }
    }
}

pub struct HoverStateResetSystem;

#[time_system]
impl EcsSystem for HoverStateResetSystem {
    fn run(&mut self, world: &mut World) {
        world.for_each_component_mut::<InteractionComponent, _>(|_, interaction_comp| {
            interaction_comp.is_hovered = false;
        });
    }
}

pub struct MouseInputSystem;

#[time_system]
impl EcsSystem for MouseInputSystem {
    fn run(&mut self, world: &mut World) {
        // Get the mouse position from the resource
        let mouse_resource = world
            .resources
            .get_resource::<MouseResource>()
            .expect("Expected MouseResource to be present");

        let render_order_resource = world
            .resources
            .get_resource::<RenderOrderResource>()
            .expect("Expected RenderOrderResource to be present, while trying to get render order");

        let interactive_entities = world
            .components
            .query_combined_2::<BoundsComponent, InteractionComponent>();

        let mut entities_interacted_with = Vec::new();

        for (entity_id, bounds_comp, interaction_comp) in interactive_entities {
            if interaction_comp.is_active
                && is_hit(
                    bounds_comp.computed_bounds,
                    bounds_comp.clip_bounds,
                    mouse_resource.position,
                )
            {
                // get index of the entity in the render order
                let index = render_order_resource
                    .render_order
                    .iter()
                    .position(|id| *id == entity_id)
                    .expect("Expected clicked entity to be in the render order");

                // Handle click events
                if interaction_comp.is_clickable && mouse_resource.is_released {
                    entities_interacted_with.push((
                        entity_id,
                        interaction_comp
                            .click_event
                            .expect("expected clickable entity to have click event"),
                        index,
                    ));
                }
            }
        }

        // send the event for the entity that has the highest z index
        if let Some((entity_id, app_event, _)) = entities_interacted_with
            .iter()
            .max_by_key(|(_, _, index)| *index)
        {
            log::debug!("Sending event: {app_event:?} from entity: {entity_id}");
            world.queue_event(*app_event);
        }
    }
}

pub struct MouseScrollSystem;

#[time_system]
impl EcsSystem for MouseScrollSystem {
    fn run(&mut self, world: &mut World) {
        // Get the mouse position from the resource
        let mouse_resource = world
            .resources
            .get_resource::<MouseResource>()
            .expect("Expected MouseResource to be present");
        let render_order_resource = world
            .resources
            .get_resource::<RenderOrderResource>()
            .expect("Expected RenderOrderResource to be present, while trying to get render order");

        let scrollable_entities = world
            .components
            .query_combined_3::<LayoutComponent, InteractionComponent, BoundsComponent>();

        let mut entities_scrolled = Vec::new();

        for (entity_id, layout_comp, interaction_comp, bounds_comp) in scrollable_entities {
            if layout_comp.layout.is_scrollable
                && interaction_comp.is_active
                && is_hit(
                    bounds_comp.computed_bounds,
                    bounds_comp.clip_bounds,
                    mouse_resource.position,
                )
            {
                // get index of the entity in the render order
                let index = render_order_resource
                    .render_order
                    .iter()
                    .position(|id| *id == entity_id)
                    .expect("Expected scrolled entity to be in the render order");

                entities_scrolled.push((entity_id, index));
            }
        }

        // Apply scroll delta to the entity with the highest z index
        if let Some((entity_id, _)) = entities_scrolled.iter().max_by_key(|(_, index)| *index) {
            if let Some(layout_comp) = world
                .components
                .get_component_mut::<LayoutComponent>(*entity_id)
            {
                layout_comp.layout.scroll_position +=
                    mouse_resource.scroll_delta * SCROLL_MULTIPLIER;
                layout_comp.layout.scroll_position = layout_comp
                    .layout
                    .scroll_position
                    .clamp(0.0, layout_comp.layout.max_scroll);
                let request_relayout_resource = world
                    .resources
                    .get_resource_mut::<RequestReLayoutResource>()
                    .expect("Expected RequestReLayoutResource to be present");
                request_relayout_resource.request_relayout = true;
            }
        }
    }
}

fn is_hit(
    computed_bounds: Bounds,
    clip_bounds: Option<ClipBounds>,
    mouse_position: ComponentPosition,
) -> bool {
    let x = mouse_position.x;
    let y = mouse_position.y;

    // Check if clipped
    if let Some(clip_bounds) = &clip_bounds {
        if clip_bounds.clip_x
            && (x < clip_bounds.bounds.position.x
                || x > clip_bounds.bounds.position.x + clip_bounds.bounds.size.width)
        {
            return false;
        }
        if clip_bounds.clip_y
            && (y < clip_bounds.bounds.position.y
                || y > clip_bounds.bounds.position.y + clip_bounds.bounds.size.height)
        {
            return false;
        }
    }

    x >= computed_bounds.position.x
        && x <= computed_bounds.position.x + computed_bounds.size.width
        && y >= computed_bounds.position.y
        && y <= computed_bounds.position.y + computed_bounds.size.height
}
