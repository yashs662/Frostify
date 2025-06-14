use crate::{
    ui::{
        animation::{AnimationDirection, AnimationType, AnimationWhen},
        color::Color,
        ecs::{
            EcsSystem, EntityId, World,
            components::*,
            resources::{
                EntryExitAnimationStateResource, RequestReLayoutResource, WgpuQueueResource,
            },
        },
    },
    utils::create_entity_buffer_data,
};

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

        let entry_exit_anim_state_resource = world
            .resources
            .get_resource_mut::<EntryExitAnimationStateResource>()
            .expect("Expected EntryExitAnimationStateResource to be present");

        // Get relevant entities and build update list
        {
            let entities_with_animation = world
                .components
                .query_combined_2::<AnimationComponent, InteractionComponent>();

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
                    if animation.config.when == AnimationWhen::Entry {
                        if raw_progress >= 1.0 && interaction_comp.is_just_activated {
                            entities_with_entry_anim_completed.push(entity_id);
                        }
                        entry_exit_anim_state_resource
                            .entry_animation_state
                            .insert(entity_id, true);
                    }

                    if animation.config.when == AnimationWhen::Exit {
                        if raw_progress >= 1.0 && interaction_comp.is_just_deactivated {
                            entities_with_exit_anim_completed.push(entity_id);
                        }
                        entry_exit_anim_state_resource
                            .exit_animation_state
                            .insert(entity_id, true);
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
        for entity_id in &entities_with_entry_anim_completed {
            let interaction_comp = world
                .components
                .get_component_mut::<InteractionComponent>(*entity_id)
                .expect("Expected InteractionComponent to be present for modal child entity");
            interaction_comp.is_just_activated = false;
        }

        for entity_id in &entities_with_exit_anim_completed {
            let interaction_comp = world
                .components
                .get_component_mut::<InteractionComponent>(*entity_id)
                .expect("Expected InteractionComponent to be present for modal child entity");
            interaction_comp.is_just_deactivated = false;
            interaction_comp.is_active = false;
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
