use crate::ui::ecs::{EcsSystem, World, components::*};
use crate::wgpu_ctx::WgpuCtx;

pub struct AnimationSystem {
    pub frame_time: f32,
}

impl EcsSystem for AnimationSystem {
    fn run(&mut self, world: &mut World) {
        let entities_with_animation =
            world.query_combined::<AnimationComponent, InteractionComponent>();
        let mut entities_to_update = Vec::new();

        // First pass: collect entities that need update
        for (entity_id, anim_comp, interaction) in entities_with_animation {
            let need_update =
                anim_comp.needs_update || (interaction.is_hoverable && interaction.is_hovered);

            if need_update {
                entities_to_update.push((entity_id, interaction.is_hovered));
            }
        }

        // Second pass: process animations
        for (entity_id, is_hovered) in entities_to_update {
            let mut scale_changed = false;
            let mut tint_changed = false;
            let mut color_changed = false;

            // Process animation
            if let Some(anim_comp) = world.get_component_mut::<AnimationComponent>(entity_id) {
                for animation in &mut anim_comp.animations {
                    let progress = animation.update(self.frame_time, is_hovered);

                    // Process different animation types
                    match &animation.config.animation_type {
                        crate::ui::animation::AnimationType::Scale { .. } => {
                            scale_changed = true;
                        }
                        crate::ui::animation::AnimationType::FrostedGlassTint { .. } => {
                            tint_changed = true;
                        }
                        crate::ui::animation::AnimationType::Color { .. } => {
                            color_changed = true;
                        }
                    }
                }
            }

            // Apply animation effects
            if scale_changed {
                // Handle scale changes
            }

            if tint_changed {
                // Handle tint changes
            }

            if color_changed {
                // Handle color changes
            }
        }
    }
}

pub struct RenderSystem<'a> {
    pub wgpu_ctx: &'a mut WgpuCtx<'a>,
    pub render_pass: &'a mut wgpu::RenderPass<'a>,
    pub app_pipelines: &'a mut crate::wgpu_ctx::AppPipelines,
}

impl<'a> EcsSystem for RenderSystem<'a> {
    fn run(&mut self, world: &mut World) {
        // Get sorted render order
        let mut render_entities = Vec::new();

        for (entity_id, identity) in world.query::<IdentityComponent>() {
            if let (Some(visual), Some(bounds), Some(transform)) = (
                world.get_component::<VisualComponent>(entity_id),
                world.get_component::<BoundsComponent>(entity_id),
                world.get_component::<TransformComponent>(entity_id),
            ) {
                if visual.is_visible {
                    render_entities.push((entity_id, transform.z_index));
                }
            }
        }

        // Sort by z-index
        render_entities.sort_by_key(|&(_, z)| z);

        // Render each entity
        for (entity_id, _) in render_entities {
            if let (Some(visual), Some(render_data)) = (
                world.get_component::<VisualComponent>(entity_id),
                world.get_component::<RenderDataComponent>(entity_id),
            ) {
                // Render based on component type
                match visual.component_type {
                    crate::ui::component::ComponentType::BackgroundColor => {
                        // Render background color
                    }
                    // Handle other component types...
                    _ => {}
                }
            }
        }
    }
}
