use crate::{
    app::AppEvent,
    ui::{
        ecs::{EcsResource, EntityId, NamedRef, systems::RenderGroup},
        layout::ComponentPosition,
    },
};
use cosmic_text::{FontSystem, SwashCache};
use frostify_derive::EcsResource;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;

// New resource to store the render order from layout context
#[derive(EcsResource, Default)]
pub struct RenderOrderResource {
    pub render_order: Vec<EntityId>,
}

// Resource for WGPU device and queue access
#[derive(Clone, EcsResource)]
pub struct WgpuQueueResource {
    pub queue: std::sync::Arc<wgpu::Queue>,
}

// Resource to store render groups
#[derive(Clone, EcsResource, Default)]
pub struct RenderGroupsResource {
    pub groups: Vec<RenderGroup>,
}

#[derive(EcsResource, Default)]
pub struct MouseResource {
    pub position: ComponentPosition,
    pub is_pressed: bool,
    pub is_released: bool,
    pub is_dragging: bool,
    pub is_scrolling: bool,
    pub scroll_delta: f32,
    pub press_position: Option<ComponentPosition>,
}

#[derive(EcsResource)]
pub struct EventSenderResource {
    pub event_sender: UnboundedSender<AppEvent>,
}

#[derive(EcsResource, Default)]
pub struct RequestReLayoutResource {
    pub request_relayout: bool,
}

#[derive(EcsResource, Default)]
pub struct NamedRefsResource {
    pub named_refs_map: HashMap<NamedRef, EntityId>,
}

impl NamedRefsResource {
    pub fn get_entity_id(&self, named_ref: &NamedRef) -> Option<EntityId> {
        self.named_refs_map.get(named_ref).cloned()
    }

    pub fn get_named_ref(&self, entity_id: EntityId) -> Option<NamedRef> {
        self.named_refs_map.iter().find_map(|(named_ref, id)| {
            if *id == entity_id {
                Some(*named_ref)
            } else {
                None
            }
        })
    }

    pub fn set_entity_id(&mut self, named_ref: NamedRef, entity_id: EntityId) {
        if let Some(existing_id) = self.named_refs_map.get(&named_ref) {
            if *existing_id != entity_id {
                log::warn!(
                    "Overwriting existing entity ID for named reference '{}': {} -> {}",
                    named_ref,
                    existing_id,
                    entity_id
                );
            }
        }
        self.named_refs_map.insert(named_ref, entity_id);
    }
}

#[derive(EcsResource)]
pub struct TextRenderingResource {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
}

impl Default for TextRenderingResource {
    fn default() -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
        }
    }
}
