use crate::ui::ecs::{
    EcsSystem, EntityId, NamedRef, World,
    components::*,
    resources::{EntryExitAnimationStateResource, NamedRefsResource, RequestReLayoutResource},
};
use frostify_derive::time_system;

pub struct ModalAnimationObserverSystem;

impl EcsSystem for ModalAnimationObserverSystem {
    fn run(&mut self, world: &mut World) {
        let mut active_modals = Vec::new();
        world.for_each_component::<ModalComponent, _>(|entity_id, modal_comp| {
            if modal_comp.is_open {
                active_modals.push(entity_id);
            }
        });

        if active_modals.is_empty() {
            return; // No active modals to process
        }

        let entry_exit_anim_state_resource = world
            .resources
            .get_resource_mut::<EntryExitAnimationStateResource>()
            .expect("Expected EntryExitAnimationStateResource to be present");

        for modal_parent_id in active_modals {
            let modal_comp = world
                .components
                .get_component_mut::<ModalComponent>(modal_parent_id)
                .expect("Expected ModalComponent to be present for modal parent entity");

            if modal_comp.is_opening {
                // check if all children has entry as true
                if modal_comp.renderable_children.iter().all(|child_id| {
                    *entry_exit_anim_state_resource
                        .entry_animation_state
                        .get(child_id)
                        .unwrap_or(&false)
                }) {
                    // All children have completed entry animation
                    modal_comp.is_opening = false;
                    modal_comp.is_open = true; // Ensure modal is marked as open
                    modal_comp.is_closing = false; // Reset closing state
                    for child_id in &modal_comp.renderable_children {
                        entry_exit_anim_state_resource
                            .entry_animation_state
                            .remove(child_id);
                    }

                    log::debug!("Modal {modal_parent_id} is now open");
                }
            } else if modal_comp.is_closing {
                // Check if all children have completed exit animation
                if modal_comp.renderable_children.iter().all(|child_id| {
                    *entry_exit_anim_state_resource
                        .exit_animation_state
                        .get(child_id)
                        .unwrap_or(&false)
                }) {
                    // All children have completed exit animation
                    modal_comp.is_closing = false;
                    modal_comp.is_open = false; // Ensure modal is marked as closed
                    for child_id in &modal_comp.renderable_children {
                        entry_exit_anim_state_resource
                            .exit_animation_state
                            .remove(child_id);
                    }

                    log::debug!("Modal {modal_parent_id} is now closed");
                }
            }
        }
    }
}

pub struct ModalToggleSystem {
    pub activate: bool,
    pub named_ref: NamedRef,
}

#[time_system]
impl EcsSystem for ModalToggleSystem {
    fn run(&mut self, world: &mut World) {
        let named_ref_resource = world
            .resources
            .get_resource::<NamedRefsResource>()
            .expect("Expected NamedRefsResource to be present");
        let modal_parent_id = named_ref_resource
            .get_entity_id(&self.named_ref)
            .expect("Expected named reference to be present in NamedRefsResource");

        // First, get the modal component data we need without holding a mutable reference
        let (has_entry_animation, has_exit_animation, children) = {
            let modal_component = world
                .components
                .get_component::<ModalComponent>(modal_parent_id)
                .expect("Expected ModalComponent to be present for modal parent entity");

            let mut all_children = modal_component.renderable_children.clone();
            all_children.extend(modal_component.non_renderable_children.clone());
            (
                modal_component.has_entry_animation,
                modal_component.has_exit_animation,
                all_children,
            )
        };

        // Now update the modal component state
        let modal_component = world
            .components
            .get_component_mut::<ModalComponent>(modal_parent_id)
            .expect("Expected ModalComponent to be present for modal parent entity");

        if self.activate {
            // Opening the modal
            modal_component.is_open = true;
            modal_component.is_closing = false; // Reset closing state
            modal_component.is_opening = has_entry_animation;
        } else {
            // Closing the modal
            if has_exit_animation {
                modal_component.is_closing = true;
                modal_component.is_opening = false; // Reset opening state
            } else {
                // No exit animation, deactivate immediately
                modal_component.is_open = false;
                modal_component.is_closing = false;
                modal_component.is_opening = false;
            }
        }

        // Now handle the entity activation/deactivation
        if self.activate {
            if has_entry_animation {
                // Don't activate interaction components yet - let animation system handle it
                Self::prepare_entities_for_entry_animation(world, modal_parent_id, &children);
            } else {
                // No entry animation, activate immediately
                Self::activate_modal_entities(world, modal_parent_id, &children);
            }
        } else if has_exit_animation {
            // Don't deactivate interaction components yet - let animation system handle it
            Self::prepare_entities_for_exit_animation(world, modal_parent_id, &children);
        } else {
            // No exit animation, deactivate immediately
            Self::deactivate_modal_entities(world, modal_parent_id, &children);
        }

        // Request a relayout after activating/deactivating components
        let request_relayout_resource = world
            .resources
            .get_resource_mut::<RequestReLayoutResource>()
            .expect("Expected RequestReLayoutResource to be present");
        request_relayout_resource.request_relayout = true;
    }
}

impl ModalToggleSystem {
    /// Prepare entities for entry animation - activate them and mark for entry animation
    fn prepare_entities_for_entry_animation(
        world: &mut World,
        modal_parent_id: EntityId,
        children: &[EntityId],
    ) {
        // Activate the modal parent first
        if let Some(interaction_comp) = world
            .components
            .get_component_mut::<InteractionComponent>(modal_parent_id)
        {
            interaction_comp.is_active = true;
            interaction_comp.is_just_activated = true;
            interaction_comp.is_just_deactivated = false;
        }

        // Activate all children
        for &child_id in children {
            if let Some(interaction_comp) = world
                .components
                .get_component_mut::<InteractionComponent>(child_id)
            {
                interaction_comp.is_active = true;
                interaction_comp.is_just_activated = true;
                interaction_comp.is_just_deactivated = false;
            }
        }
    }

    /// Prepare entities for exit animation - mark for exit animation but keep them active
    fn prepare_entities_for_exit_animation(
        world: &mut World,
        modal_parent_id: EntityId,
        children: &[EntityId],
    ) {
        // Mark the modal parent for exit animation
        if let Some(interaction_comp) = world
            .components
            .get_component_mut::<InteractionComponent>(modal_parent_id)
        {
            interaction_comp.is_just_deactivated = true;
            interaction_comp.is_just_activated = false;
            // Keep is_active = true so the entity stays visible during exit animation
        }

        // Mark all children for exit animation
        for &child_id in children {
            if let Some(interaction_comp) = world
                .components
                .get_component_mut::<InteractionComponent>(child_id)
            {
                interaction_comp.is_just_deactivated = true;
                interaction_comp.is_just_activated = false;
                // Keep is_active = true so the entity stays visible during exit animation
            }
        }
    }

    /// Activate modal entities immediately (no animation)
    fn activate_modal_entities(
        world: &mut World,
        modal_parent_id: EntityId,
        children: &[EntityId],
    ) {
        // Activate the modal parent
        if let Some(interaction_comp) = world
            .components
            .get_component_mut::<InteractionComponent>(modal_parent_id)
        {
            interaction_comp.is_active = true;
            interaction_comp.is_just_activated = false;
            interaction_comp.is_just_deactivated = false;
        }

        // Activate all children
        for &child_id in children {
            if let Some(interaction_comp) = world
                .components
                .get_component_mut::<InteractionComponent>(child_id)
            {
                interaction_comp.is_active = true;
                interaction_comp.is_just_activated = false;
                interaction_comp.is_just_deactivated = false;
            }
        }
    }

    /// Deactivate modal entities immediately (no animation)
    fn deactivate_modal_entities(
        world: &mut World,
        modal_parent_id: EntityId,
        children: &[EntityId],
    ) {
        // Deactivate the modal parent
        let interaction_comp = world
            .components
            .get_component_mut::<InteractionComponent>(modal_parent_id)
            .expect("Expected InteractionComponent to be present for modal parent entity");

        interaction_comp.is_active = false;
        interaction_comp.is_just_activated = false;
        interaction_comp.is_just_deactivated = false;

        // Deactivate all children
        for &child_id in children {
            let interaction_comp = world
                .components
                .get_component_mut::<InteractionComponent>(child_id)
                .expect("Expected InteractionComponent to be present for modal child entity");

            interaction_comp.is_active = false;
            interaction_comp.is_just_activated = false;
            interaction_comp.is_just_deactivated = false;
        }
    }
}
