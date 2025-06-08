use crate::{
    constants::SCROLL_MULTIPLIER,
    ui::{
        animation::{AnimationDirection, AnimationType, AnimationWhen},
        color::Color,
        ecs::{
            ComponentType, EcsSystem, EntityId, World,
            components::*,
            resources::{
                MouseResource, RenderGroupsResource, RenderOrderResource, RequestReLayoutResource,
                WgpuQueueResource,
            },
        },
        layout::{Bounds, ClipBounds, ComponentPosition},
    },
    utils::create_entity_buffer_data,
};
use frostify_derive::{time_function, time_system};

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
    Opacity { opacity: f32 },
}

impl EcsSystem for AnimationSystem {
    fn run(&mut self, world: &mut World) {
        // First pass: Collect animation updates without mutating components
        let mut updates = Vec::new();
        let mut animations_to_be_flipped = Vec::new();
        let mut entities_with_exit_anim_completed = Vec::new();
        let mut entities_with_entry_anim_completed = Vec::new();
        let mut any_update_requires_relayout = false;

        // Get relevant entities and build update list
        {
            let entities_with_animation =
                world.query_combined_2::<AnimationComponent, InteractionComponent>();

            for (entity_id, anim_comp, interaction_comp) in entities_with_animation {
                if !interaction_comp.is_active {
                    continue;
                }

                // Get interaction state if this component is interactive
                let is_hovered = world
                    .components
                    .get_component::<InteractionComponent>(entity_id)
                    .map(|interaction| interaction.is_hovered)
                    .expect("Expected InteractionComponent to be present, while trying to get hover state");

                // Process each animation
                for (index, animation) in anim_comp.animations.iter().enumerate() {
                    let should_animate_forward = match animation.config.when {
                        AnimationWhen::Hover => is_hovered,
                        AnimationWhen::Forever => animation.is_forever_going_forward,
                        AnimationWhen::Entry => interaction_comp.is_just_activated,
                        AnimationWhen::Exit => interaction_comp.is_just_deactivated,
                    };

                    // Reset animation progress when starting a new entry/exit animation
                    let mut current_progress = animation.progress;
                    if matches!(
                        animation.config.when,
                        AnimationWhen::Entry | AnimationWhen::Exit
                    ) && should_animate_forward
                        && animation.progress >= 1.0
                    {
                        current_progress = 0.0;
                    }

                    // Calculate delta based on frame time and duration
                    let delta = self.frame_time / animation.config.duration.as_secs_f32();

                    // Calculate the raw progress value based on animation direction
                    let raw_progress = match animation.config.direction {
                        AnimationDirection::Forward => {
                            if should_animate_forward {
                                (current_progress + delta).min(1.0)
                            } else {
                                // Don't reset entry/exit animations when not animating
                                match animation.config.when {
                                    AnimationWhen::Entry | AnimationWhen::Exit => current_progress,
                                    _ => 0.0, // Instant reset for other animation types
                                }
                            }
                        }
                        AnimationDirection::Backward => {
                            if should_animate_forward {
                                1.0 // Instant full
                            } else {
                                // Don't reset entry/exit animations when not animating
                                match animation.config.when {
                                    AnimationWhen::Entry | AnimationWhen::Exit => current_progress,
                                    _ => (current_progress - delta).max(0.0),
                                }
                            }
                        }
                        AnimationDirection::Alternate => {
                            if should_animate_forward {
                                (current_progress + delta).min(1.0)
                            } else {
                                // Don't reset entry/exit animations when not animating
                                match animation.config.when {
                                    AnimationWhen::Entry | AnimationWhen::Exit => current_progress,
                                    _ => (current_progress - delta).max(0.0),
                                }
                            }
                        }
                        AnimationDirection::AlternateReverse => {
                            if should_animate_forward {
                                (current_progress - delta).max(0.0)
                            } else {
                                // Don't reset entry/exit animations when not animating
                                match animation.config.when {
                                    AnimationWhen::Entry | AnimationWhen::Exit => current_progress,
                                    _ => (current_progress + delta).min(1.0),
                                }
                            }
                        }
                    };

                    // Apply easing to the calculated raw progress
                    let eased_progress = animation.config.easing.compute(raw_progress);
                    let should_process = should_animate_forward
                        || current_progress > 0.0
                        || animation.config.when == AnimationWhen::Forever;

                    // Don't process completed entry/exit animations that aren't actively animating
                    if matches!(
                        animation.config.when,
                        AnimationWhen::Entry | AnimationWhen::Exit
                    ) && !should_animate_forward
                        && raw_progress >= 1.0
                    {
                        continue;
                    }

                    if !should_process {
                        continue;
                    }

                    // Process different animation types and collect updates
                    match &animation.config.animation_type {
                        AnimationType::Scale { range, .. } => {
                            let scale = range.from + (range.to - range.from) * eased_progress;
                            let transform =
                                world.components.get_component::<TransformComponent>(entity_id).expect(
                                    "Expected TransformComponent to be present, while trying to update scale animation",
                                );
                            if (transform.scale_factor - scale).abs() > 0.001 {
                                any_update_requires_relayout = true;
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
                        AnimationType::Color { range } => {
                            let color = range.from.lerp(&range.to, eased_progress);
                            updates.push(AnimationUpdateData {
                                entity_id,
                                update_type: AnimationUpdateType::Color { color },
                                animation_index: index,
                                raw_progress,
                            });
                        }
                        AnimationType::FrostedGlassTint { range } => {
                            let tint_color = range.from.lerp(&range.to, eased_progress);
                            updates.push(AnimationUpdateData {
                                entity_id,
                                update_type: AnimationUpdateType::FrostedGlass { tint_color },
                                animation_index: index,
                                raw_progress,
                            });
                        }
                        AnimationType::Opacity { range } => {
                            let opacity = range.from + (range.to - range.from) * eased_progress;
                            updates.push(AnimationUpdateData {
                                entity_id,
                                update_type: AnimationUpdateType::Opacity { opacity },
                                animation_index: index,
                                raw_progress,
                            });
                        }
                    }

                    // check if we need to flip the animation direction for forever animations
                    if animation.config.when == AnimationWhen::Forever
                        && animation.config.direction.allows_reverse_transition()
                        && (raw_progress >= 1.0 || raw_progress <= 0.0)
                    {
                        animations_to_be_flipped.push((entity_id, index));
                    }

                    // Check if we need to stop playing entry and exit animations
                    if animation.config.when == AnimationWhen::Entry
                        && raw_progress >= 1.0
                        && interaction_comp.is_just_activated
                    {
                        entities_with_entry_anim_completed.push(entity_id);
                    }

                    if animation.config.when == AnimationWhen::Exit
                        && raw_progress >= 1.0
                        && interaction_comp.is_just_deactivated
                    {
                        entities_with_exit_anim_completed.push(entity_id);
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
                }
                AnimationUpdateType::Color { color } => {
                    if let Some(color_component) = world
                        .components
                        .get_component_mut::<ColorComponent>(update.entity_id)
                    {
                        color_component.color = color;
                    }
                }
                AnimationUpdateType::FrostedGlass { tint_color } => {
                    if let Some(frosted_glass) = world
                        .components
                        .get_component_mut::<FrostedGlassComponent>(update.entity_id)
                    {
                        frosted_glass.tint_color = tint_color;
                    }
                }
                AnimationUpdateType::Opacity { opacity } => {
                    if let Some(visual_comp) = world
                        .components
                        .get_component_mut::<VisualComponent>(update.entity_id)
                    {
                        visual_comp.opacity = opacity;
                    }
                }
            }

            // Update the animation progress in the AnimationComponent
            if let Some(AnimationComponent { animations, .. }) = world
                .components
                .get_component_mut::<AnimationComponent>(update.entity_id)
            {
                if update.animation_index < animations.len() {
                    // Update the raw progress value, not the eased one
                    animations[update.animation_index].progress = update.raw_progress;
                }
            }

            updated_render_datas.push((
                update.entity_id,
                create_entity_buffer_data(&world.components, update.entity_id),
            ));
        }

        // Handle completed entry and exit animations
        for entity_id in entities_with_entry_anim_completed {
            if let Some(interaction_comp) = world
                .components
                .get_component_mut::<InteractionComponent>(entity_id)
            {
                log::debug!("Entry animation completed for entity: {}", entity_id);
                interaction_comp.is_just_activated = false;
            }
        }

        for entity_id in entities_with_exit_anim_completed {
            if let Some(interaction_comp) = world
                .components
                .get_component_mut::<InteractionComponent>(entity_id)
            {
                log::debug!("Exit animation completed for entity: {}", entity_id);
                interaction_comp.is_just_deactivated = false;
                interaction_comp.is_active = false;
            }
        }

        // Flip the animation direction for forever animations
        for (entity_id, animation_index) in animations_to_be_flipped {
            if let Some(anim_comp) = world
                .components
                .get_component_mut::<AnimationComponent>(entity_id)
            {
                if animation_index < anim_comp.animations.len() {
                    anim_comp.animations[animation_index].is_forever_going_forward =
                        !anim_comp.animations[animation_index].is_forever_going_forward;
                }
            }
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
                &render_data_comp.render_data_buffer,
                0,
                bytemuck::cast_slice(&[render_data]),
            );
        }

        if any_update_requires_relayout {
            // Request a relayout if any animations were updated
            let request_relayout_resource = world
                .resources
                .get_resource_mut::<RequestReLayoutResource>()
                .expect("Expected RequestReLayoutResource to be present");
            request_relayout_resource.request_relayout = true;
        }

        // TODO: Handle special case where scale animation causes the parent's
        // max_scroll to change if so check if we need to scroll the parent to avoid over scrolling
    }
}

// Define render group structure for ECS rendering
#[derive(Debug, Clone)]
pub struct RenderGroup {
    pub entity_ids: Vec<EntityId>,
    pub is_frosted_glass: bool,
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

            let interaction_comp = world
                .components
                .get_component::<InteractionComponent>(*component_id)
                .unwrap_or_else(|| {
                    panic!(
                        "Failed to get InteractionComponent for entity: {}",
                        component_id
                    )
                });

            // Skip container components and inactive components
            if identity_comp.component_type == ComponentType::Container
                || !interaction_comp.is_active
            {
                continue;
            }

            let is_frosted_glass = identity_comp.component_type == ComponentType::FrostedGlass;

            // Check if we need to start a new group
            if (is_frosted_glass != current_group.is_frosted_glass)
                && !current_group.entity_ids.is_empty()
            {
                render_groups.push(current_group);
                current_group = RenderGroup {
                    entity_ids: Vec::new(),
                    is_frosted_glass,
                };
            } else if current_group.entity_ids.is_empty() {
                current_group.is_frosted_glass = is_frosted_glass;
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
            world.query_combined_2::<BoundsComponent, InteractionComponent>();

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
            world.query_combined_2::<BoundsComponent, InteractionComponent>();

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
            log::debug!("Sending event: {:?} from entity: {}", app_event, entity_id);
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

        let scrollable_entities =
            world.query_combined_3::<LayoutComponent, InteractionComponent, BoundsComponent>();

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

pub struct ComponentActivationSystem {
    pub activate: bool,
    pub entity_id: EntityId,
    pub affect_children: bool,
}

#[time_system]
impl EcsSystem for ComponentActivationSystem {
    fn run(&mut self, world: &mut World) {
        // Parent
        let (parent_has_entry_anim, parent_has_exit_anim) = if let Some(parent_anim_comp) = world
            .components
            .get_component::<AnimationComponent>(self.entity_id)
        {
            let parent_has_entry_anim = parent_anim_comp
                .animations
                .iter()
                .any(|anim| anim.config.when == AnimationWhen::Entry);
            let parent_has_exit_anim = parent_anim_comp
                .animations
                .iter()
                .any(|anim| anim.config.when == AnimationWhen::Exit);
            (parent_has_entry_anim, parent_has_exit_anim)
        } else {
            (false, false)
        };

        let parent_interaction_comp = world
            .components
            .get_component_mut::<InteractionComponent>(self.entity_id)
            .expect("Expected InteractionComponent to be present for parent entity");

        // reset both just activated and deactivated flags
        parent_interaction_comp.is_just_activated = false;
        parent_interaction_comp.is_just_deactivated = false;

        if self.activate {
            if parent_interaction_comp.is_active {
                log::warn!(
                    "Trying to activate an already active component: {}",
                    self.entity_id
                );
                return; // Already active, no need to activate again
            }

            parent_interaction_comp.is_active = true;

            if parent_has_entry_anim {
                parent_interaction_comp.is_just_activated = true;
            }
        } else {
            if !parent_interaction_comp.is_active {
                log::warn!(
                    "Trying to deactivate an already inactive component: {}",
                    self.entity_id
                );
                return; // Already inactive, no need to deactivate again
            }

            // Instant deactivation by default
            parent_interaction_comp.is_active = false;

            // The animation system will handle the deactivation - This is to handle exit animations
            if parent_has_exit_anim {
                parent_interaction_comp.is_active = true;
                parent_interaction_comp.is_just_deactivated = true;
            }
        }

        if !self.affect_children {
            return;
        }

        let children = gather_all_children(world, self.entity_id);
        for child_id in children {
            let (child_has_entry_anim, child_has_exit_anim) = if let Some(child_anim_comp) = world
                .components
                .get_component::<AnimationComponent>(child_id)
            {
                let child_has_entry_anim = child_anim_comp
                    .animations
                    .iter()
                    .any(|anim| anim.config.when == AnimationWhen::Entry);
                let child_has_exit_anim = child_anim_comp
                    .animations
                    .iter()
                    .any(|anim| anim.config.when == AnimationWhen::Exit);
                (child_has_entry_anim, child_has_exit_anim)
            } else {
                (false, false)
            };

            if let Some(interaction_comp) = world
                .components
                .get_component_mut::<InteractionComponent>(child_id)
            {
                // Reset both just activated and deactivated flags
                interaction_comp.is_just_activated = false;
                interaction_comp.is_just_deactivated = false;

                if self.activate {
                    interaction_comp.is_active = true;
                    if child_has_entry_anim {
                        interaction_comp.is_just_activated = true;
                    }
                } else {
                    interaction_comp.is_active = false;

                    // The animation system will handle the deactivation - This is to handle exit animations
                    if child_has_exit_anim {
                        interaction_comp.is_active = true;
                        interaction_comp.is_just_deactivated = true;
                    }
                }
            }
        }

        // Request a relayout after activating/deactivating components
        let request_relayout_resource = world
            .resources
            .get_resource_mut::<RequestReLayoutResource>()
            .expect("Expected RequestReLayoutResource to be present");
        request_relayout_resource.request_relayout = true;
    }
}

/// function to iteratively collect all children entities
#[time_function]
fn gather_all_children(world: &World, root_entity_id: EntityId) -> Vec<EntityId> {
    let mut all_children = Vec::new();
    let mut to_process = vec![root_entity_id];

    while let Some(entity_id) = to_process.pop() {
        if let Some(hierarchy_comp) = world
            .components
            .get_component::<HierarchyComponent>(entity_id)
        {
            for &child_id in &hierarchy_comp.children {
                all_children.push(child_id);
                to_process.push(child_id);
            }
        }
    }

    all_children
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
