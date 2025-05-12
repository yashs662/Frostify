use super::resources::{MouseResource, RenderGroupsResource};
use crate::{
    ui::{
        animation::{AnimationDirection, AnimationType, AnimationWhen},
        color::Color,
        ecs::{
            ComponentType, EcsSystem, EntityId, World,
            components::*,
            resources::{RenderOrderResource, WgpuQueueResource},
        },
        layout::{Bounds, ClipBounds, ComponentPosition},
    },
    utils::create_component_buffer_data,
};
use frostify_derive::time_system;

pub struct AnimationSystem {
    pub frame_time: f32,
}

// Animation update data to be collected in first pass
struct AnimationUpdateData {
    entity_id: EntityId,
    update_type: AnimationUpdateType,
    animation_index: usize,
    raw_progress: f32,
}

// Types of updates to perform in second pass
enum AnimationUpdateType {
    Scale { scale_factor: f32 },
    Color { color: Color },
    FrostedGlass { tint_color: Color },
}

impl EcsSystem for AnimationSystem {
    fn run(&mut self, world: &mut World) {
        // First pass: Collect animation updates without mutating components
        let mut updates = Vec::new();

        // Get relevant entities and build update list
        {
            let entities_with_animation =
                world.query_combined_2::<AnimationComponent, VisualComponent>();

            for (entity_id, anim_comp, visual_comp) in entities_with_animation {
                if !visual_comp.is_visible {
                    continue;
                }

                // Get interaction state if this component is interactive
                let (is_hovered, is_clicked) = world
                    .components
                    .get_component::<InteractionComponent>(entity_id)
                    .map(|interaction| (interaction.is_hovered, interaction.is_clicked))
                    .unwrap_or((false, false));

                // Process each animation
                for (index, animation) in anim_comp.animations.iter().enumerate() {
                    let should_animate_forward = match animation.config.when {
                        AnimationWhen::Hover => is_hovered,
                        AnimationWhen::OnClick => is_clicked,
                        AnimationWhen::Forever => true,
                    };

                    // Calculate delta based on frame time and duration
                    let delta = self.frame_time / animation.config.duration.as_secs_f32();

                    // Calculate the raw progress value based on animation direction
                    let raw_progress = match animation.config.direction {
                        AnimationDirection::Forward => {
                            if should_animate_forward {
                                (animation.progress + delta).min(1.0)
                            } else {
                                0.0 // Instant reset
                            }
                        }
                        AnimationDirection::Backward => {
                            if should_animate_forward {
                                1.0 // Instant full
                            } else {
                                (animation.progress - delta).max(0.0)
                            }
                        }
                        AnimationDirection::Alternate => {
                            if should_animate_forward {
                                (animation.progress + delta).min(1.0)
                            } else {
                                (animation.progress - delta).max(0.0)
                            }
                        }
                        AnimationDirection::AlternateReverse => {
                            if should_animate_forward {
                                (animation.progress - delta).max(0.0)
                            } else {
                                (animation.progress + delta).min(1.0)
                            }
                        }
                    };

                    // Apply easing to the calculated raw progress
                    let eased_progress = animation.config.easing.compute(raw_progress);
                    let should_process = should_animate_forward || animation.progress > 0.0;

                    if !should_process {
                        continue;
                    }

                    // Process different animation types and collect updates
                    match &animation.config.animation_type {
                        AnimationType::Scale { from, to, .. } => {
                            let scale = from + (to - from) * eased_progress;
                            let transform =
                                world.components.get_component::<TransformComponent>(entity_id).expect(
                                    "Expected TransformComponent to be present, while trying to update scale animation",
                                );
                            if (transform.scale_factor - scale).abs() > 0.001 {
                                updates.push(AnimationUpdateData {
                                    entity_id,
                                    update_type: AnimationUpdateType::Scale {
                                        scale_factor: scale,
                                    },
                                    animation_index: index,
                                    raw_progress,
                                });
                            }
                        }
                        AnimationType::Color { from, to } => {
                            let color = from.lerp(to, eased_progress);
                            updates.push(AnimationUpdateData {
                                entity_id,
                                update_type: AnimationUpdateType::Color { color },
                                animation_index: index,
                                raw_progress,
                            });
                        }
                        AnimationType::FrostedGlassTint { from, to } => {
                            let tint_color = from.lerp(to, eased_progress);
                            updates.push(AnimationUpdateData {
                                entity_id,
                                update_type: AnimationUpdateType::FrostedGlass { tint_color },
                                animation_index: index,
                                raw_progress,
                            });
                        }
                    }
                }
            }
        }

        let mut updated_render_datas = Vec::new();
        // Second pass: Apply all collected updates and calculate new render buffer data
        for update in updates {
            match update.update_type {
                AnimationUpdateType::Scale { scale_factor } => {
                    if let Some(transform) = world
                        .components
                        .get_component_mut::<TransformComponent>(update.entity_id)
                    {
                        transform.scale_factor = scale_factor;
                    }
                    if let Some(AnimationComponent { animations, .. }) =
                        world
                            .components
                            .get_component_mut::<AnimationComponent>(update.entity_id)
                    {
                        if update.animation_index < animations.len() {
                            // Update the raw progress value, not the eased one
                            animations[update.animation_index].progress = update.raw_progress;
                        }
                    }
                }
                AnimationUpdateType::Color { color } => {
                    if let Some(color_component) = world
                        .components
                        .get_component_mut::<ColorComponent>(update.entity_id)
                    {
                        color_component.color = color;
                    }
                    if let Some(AnimationComponent { animations, .. }) =
                        world
                            .components
                            .get_component_mut::<AnimationComponent>(update.entity_id)
                    {
                        if update.animation_index < animations.len() {
                            // Update the raw progress value, not the eased one
                            animations[update.animation_index].progress = update.raw_progress;
                        }
                    }
                }
                AnimationUpdateType::FrostedGlass { tint_color } => {
                    if let Some(frosted_glass) = world
                        .components
                        .get_component_mut::<FrostedGlassComponent>(update.entity_id)
                    {
                        frosted_glass.tint_color = tint_color;
                    }
                    if let Some(AnimationComponent { animations, .. }) =
                        world
                            .components
                            .get_component_mut::<AnimationComponent>(update.entity_id)
                    {
                        if update.animation_index < animations.len() {
                            // Update the raw progress value, not the eased one
                            animations[update.animation_index].progress = update.raw_progress;
                        }
                    }
                }
            }

            // Note: Special Case for TextComponent as it doesn't have a render
            // data component it is handled in the TextRenderer

            if world
                .components
                .get_component::<TextComponent>(update.entity_id)
                .is_some()
            {
                log::warn!(
                    "TextComponent does not have a render data component, skipping animation update"
                );
                continue;
            }

            updated_render_datas.push((
                update.entity_id,
                create_component_buffer_data(world, update.entity_id),
            ));
        }

        // Third pass: Update render data buffers
        let device_queue = world
            .resources
            .get_resource::<WgpuQueueResource>()
            .expect("Expected WgpuQueueResource to be present, while updating animation");

        for (entity_id, render_data) in updated_render_datas {
            let render_data_comp = world
                .components
                .get_component::<RenderDataComponent>(entity_id)
                .expect("Expected RenderDataComponent to be present for entity, while updating animation");

            device_queue.queue.write_buffer(
                render_data_comp
                    .render_data_buffer
                    .as_ref()
                    .expect("RenderDataComponent should have a valid render data buffer"),
                0,
                bytemuck::cast_slice(&[render_data]),
            );
        }
    }
}

// Define render group structure for ECS rendering
#[derive(Debug, Clone)]
pub struct RenderGroup {
    pub entity_ids: Vec<EntityId>,
    pub is_frosted_glass: bool,
    pub is_text: bool,
}

pub struct RenderPrepareSystem;

// TODO: Potential savings of 50-60 microseconds: avoid running this if nothing has
// changed in render order or hover states
impl EcsSystem for RenderPrepareSystem {
    fn run(&mut self, world: &mut World) {
        // Get the render order from the resource, if available
        let render_order_resource = world
            .resources
            .get_resource::<RenderOrderResource>()
            .expect("Expected RenderOrderResource to be present, while trying to get render order");
        let render_order = &render_order_resource.render_order;

        // Create render groups similar to the original approach
        let mut render_groups = Vec::new();
        let mut current_group = RenderGroup {
            entity_ids: Vec::new(),
            is_frosted_glass: false,
            is_text: false,
        };

        for component_id in render_order {
            // Get visual and identity components
            let identity_comp = world
                .components
                .get_component::<IdentityComponent>(*component_id)
                .unwrap_or_else(|| {
                    panic!(
                        "Failed to get IdentityComponent for entity: {}",
                        component_id
                    )
                });

            let visual_comp = world
                .components
                .get_component::<VisualComponent>(*component_id)
                .unwrap_or_else(|| {
                    panic!("Failed to get VisualComponent for entity: {}", component_id)
                });

            // Skip container components and inactive components
            if identity_comp.component_type == ComponentType::Container || !visual_comp.is_visible {
                continue;
            }

            let is_frosted_glass = identity_comp.component_type == ComponentType::FrostedGlass;
            let is_text = identity_comp.component_type == ComponentType::Text;

            // Check if we need to start a new group
            if (is_frosted_glass != current_group.is_frosted_glass
                || is_text != current_group.is_text)
                && !current_group.entity_ids.is_empty()
            {
                render_groups.push(current_group);
                current_group = RenderGroup {
                    entity_ids: Vec::new(),
                    is_frosted_glass,
                    is_text,
                };
            } else if current_group.entity_ids.is_empty() {
                current_group.is_frosted_glass = is_frosted_glass;
                current_group.is_text = is_text;
            }

            current_group.entity_ids.push(*component_id);
        }

        // Add the last group if not empty
        if !current_group.entity_ids.is_empty() {
            render_groups.push(current_group);
        }

        // Store render groups as a resource
        let render_groups_resource = world
            .resources
            .get_resource_mut::<RenderGroupsResource>()
            .expect("Expected RenderGroupsResource to be present");
        render_groups_resource.groups = render_groups;
    }
}

pub struct ComponentHoverSystem;

impl EcsSystem for ComponentHoverSystem {
    fn run(&mut self, world: &mut World) {
        // Get the mouse position from the resource
        let mouse_resource = world
            .resources
            .get_resource::<MouseResource>()
            .expect("Expected MouseResource to be present");

        let interactive_entities =
            world.query_combined_3::<BoundsComponent, InteractionComponent, VisualComponent>();

        let mut hovered_entities = Vec::new();
        let mut dragged_entities = Vec::new();

        for (entity_id, bounds_comp, interaction_comp, visual_comp) in interactive_entities {
            // Check if the entity is visible
            if visual_comp.is_visible
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
                "Sending drag event: {:?} for entity: {}",
                drag_event,
                entity_id
            );
            world.queue_event(*drag_event);
        }
    }
}

pub struct ComponentHoverResetSystem;

#[time_system]
impl EcsSystem for ComponentHoverResetSystem {
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

        let interactive_entities =
            world.query_combined_3::<BoundsComponent, InteractionComponent, VisualComponent>();

        let mut entities_interacted_with = Vec::new();

        for (entity_id, bounds_comp, interaction_comp, visual_comp) in interactive_entities {
            if visual_comp.is_visible
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
            log::debug!("Sending event: {:?} for entity: {}", app_event, entity_id);
            world.queue_event(*app_event);
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
