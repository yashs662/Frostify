use crate::ui::ecs::{
    ComponentType, EcsSystem, EntityId, World,
    components::*,
    resources::{RenderGroupsResource, RenderOrderResource},
};

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
