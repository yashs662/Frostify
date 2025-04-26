use crate::ui::component::ComponentType;
use crate::ui::ecs::{EcsSystem, World, components::*, resources::RenderOrderResource};
use std::collections::HashMap;
use uuid::Uuid;

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

// Define render group structure for ECS rendering
#[derive(Debug, Clone)]
pub struct RenderGroup {
    pub entity_ids: Vec<Uuid>,
    pub is_frosted_glass: bool,
    pub is_text: bool,
}

pub struct RenderPrepareSystem;

impl EcsSystem for RenderPrepareSystem {
    fn run(&mut self, world: &mut World) {
        // Get the render order from the resource, if available
        let render_order =
            if let Some(render_order_resource) = world.get_resource::<RenderOrderResource>() {
                render_order_resource.render_order.clone()
            } else {
                // Fallback to the old method if resource is not available
                compute_fallback_render_order(world)
            };

        // Create render groups similar to the original approach
        let mut render_groups = Vec::new();
        let mut current_group = RenderGroup {
            entity_ids: Vec::new(),
            is_frosted_glass: false,
            is_text: false,
        };

        for component_id in render_order {
            // Get visual and identity components
            let Some(identity) = world.get_component::<IdentityComponent>(component_id) else {
                continue;
            };

            let Some(visual) = world.get_component::<VisualComponent>(component_id) else {
                continue;
            };

            // Skip container components and inactive components
            if identity.component_type == ComponentType::Container || !visual.is_visible {
                continue;
            }

            let is_frosted = identity.component_type == ComponentType::FrostedGlass;
            let is_text = identity.component_type == ComponentType::Text;

            // Check if we need to start a new group
            if (is_frosted != current_group.is_frosted_glass || is_text != current_group.is_text)
                && !current_group.entity_ids.is_empty()
            {
                render_groups.push(current_group);
                current_group = RenderGroup {
                    entity_ids: Vec::new(),
                    is_frosted_glass: is_frosted,
                    is_text,
                };
            } else if current_group.entity_ids.is_empty() {
                current_group.is_frosted_glass = is_frosted;
                current_group.is_text = is_text;
            }

            current_group.entity_ids.push(component_id);
        }

        // Add the last group if not empty
        if !current_group.entity_ids.is_empty() {
            render_groups.push(current_group);
        }

        // Store render groups as a resource
        world.add_resource(RenderGroupsResource {
            groups: render_groups,
        });
    }
}

// Fallback method for calculating render order when render_order resource is not available
fn compute_fallback_render_order(world: &World) -> Vec<Uuid> {
    // Get all visible components with visual and identity components
    let entities = world.query_combined::<VisualComponent, IdentityComponent>();

    // Filter entities and convert to a Vec for sorting and processing
    let mut visible_entities: Vec<_> = entities
        .into_iter()
        .filter(|(_, visual, _)| visual.is_visible)
        .map(|(entity_id, _, _)| entity_id)
        .collect();

    // Sort by z-index
    let mut z_index_map = HashMap::new();
    for &entity_id in &visible_entities {
        if let Some(transform) = world.get_component::<TransformComponent>(entity_id) {
            z_index_map.insert(entity_id, transform.z_index);
        }
    }

    visible_entities.sort_by(|&a_id, &b_id| {
        let a_z = z_index_map.get(&a_id).copied().unwrap_or(0);
        let b_z = z_index_map.get(&b_id).copied().unwrap_or(0);
        a_z.cmp(&b_z)
    });

    visible_entities
}

// Resource to store render groups
pub struct RenderGroupsResource {
    pub groups: Vec<RenderGroup>,
}

impl crate::ui::ecs::EcsResource for RenderGroupsResource {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub struct RenderSystem<'a> {
    pub render_pass: &'a mut wgpu::RenderPass<'a>,
    pub app_pipelines: &'a mut crate::wgpu_ctx::AppPipelines,
}

impl EcsSystem for RenderSystem<'_> {
    fn run(&mut self, world: &mut World) {
        // Render entities based on their component types
        let visual_entities = world.query::<VisualComponent>();

        // Create a mapping of entity ID to its components for quick access
        let visual_map: HashMap<_, _> = visual_entities.into_iter().collect();

        // Get the render groups
        if let Some(render_groups) = world.get_resource::<RenderGroupsResource>() {
            for group in &render_groups.groups {
                for &entity_id in &group.entity_ids {
                    if let Some(visual) = visual_map.get(&entity_id) {
                        self.render_entity(entity_id, visual, world);
                    }
                }
            }
        }
    }
}

impl RenderSystem<'_> {
    pub fn render_entity(&mut self, entity_id: Uuid, visual: &VisualComponent, world: &World) {
        // Skip if not visible
        if !visual.is_visible {
            return;
        }

        // Access render data
        if let Some(render_data) = world.get_component::<RenderDataComponent>(entity_id) {
            match visual.component_type {
                ComponentType::BackgroundColor => {
                    // Set pipeline and bind groups for background color rendering
                    if let Some(bind_group) = &render_data.bind_group {
                        self.render_pass
                            .set_pipeline(&self.app_pipelines.unified_pipeline);
                        self.render_pass.set_bind_group(0, bind_group, &[]);
                        self.render_pass.draw(0..6, 0..1); // Draw a quad (2 triangles)
                    }
                }
                ComponentType::BackgroundGradient => {
                    // Similar to background color but with gradient settings
                    if let Some(bind_group) = &render_data.bind_group {
                        self.render_pass
                            .set_pipeline(&self.app_pipelines.unified_pipeline);
                        self.render_pass.set_bind_group(0, bind_group, &[]);
                        self.render_pass.draw(0..6, 0..1);
                    }
                }
                ComponentType::Image => {
                    // Image rendering
                    if let Some(bind_group) = &render_data.bind_group {
                        self.render_pass
                            .set_pipeline(&self.app_pipelines.unified_pipeline);
                        self.render_pass.set_bind_group(0, bind_group, &[]);
                        self.render_pass.draw(0..6, 0..1);
                    }
                }
                ComponentType::FrostedGlass => {
                    // Frosted glass requires special handling
                    if let Some(bind_group) = &render_data.bind_group {
                        self.render_pass
                            .set_pipeline(&self.app_pipelines.unified_pipeline);
                        self.render_pass.set_bind_group(0, bind_group, &[]);
                        self.render_pass.draw(0..6, 0..1);
                    }
                }
                ComponentType::Text => {
                    // Text rendering would be handled by the text handler
                    // This would require special integration with TextHandler
                }
                ComponentType::Container => {
                    // Containers are not rendered directly
                }
            }
        }
    }
}
