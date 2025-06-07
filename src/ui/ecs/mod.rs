use crate::{app::AppEvent, ui::color::Color};
use components::IdentityComponent;
use frostify_derive::EntityCategories;
use resources::{
    EventSenderResource, MouseResource, RenderGroupsResource, RenderOrderResource,
    RequestReLayoutResource, TextRenderingResource, WgpuQueueResource,
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};
use strum_macros::Display;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

pub mod builders;
pub mod components;
pub mod resources;
pub mod systems;

pub type EntityId = Uuid;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ComponentType {
    Container,
    Text,
    Image,
    BackgroundColor,
    BackgroundGradient,
    FrostedGlass,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(dead_code)]
pub enum BorderPosition {
    /// Border drawn inside the component's bounds
    Inside,
    /// Border straddles the component's edges
    Center,
    /// Border drawn outside the component's bounds
    #[default]
    Outside,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderBufferData {
    pub color: [f32; 4],
    pub position: [f32; 2],      // Position in pixels (top-left corner)
    pub size: [f32; 2],          // Size in pixels (width, height)
    pub border_radius: [f32; 4], // Corner radii in pixels (top-left, top-right, bottom-left, bottom-right)
    pub screen_size: [f32; 2],   // Viewport dimensions in pixels
    pub use_texture: u32,        // Flag: 0 for color, 1 for texture, 2 for frosted glass
    pub blur_radius: f32,        // Blur amount for frosted glass (0-10)
    pub opacity: f32,            // Overall opacity for frosted glass (0.0-1.0)
    pub tint_intensity: f32,
    pub border_width: f32,      // Border thickness in pixels
    pub border_position: u32,   // Border position: 0=inside, 1=center, 2=outside
    pub border_color: [f32; 4], // Border color
    // Pre-computed values for optimization
    pub inner_bounds: [f32; 4], // (inner_min.x, inner_min.y, inner_max.x, inner_max.y)
    pub outer_bounds: [f32; 4], // (outer_min.x, outer_min.y, outer_max.x, outer_max.y)
    pub corner_centers: [f32; 4], // (tl_center.x, tl_center.y, tr_center.x, tr_center.y)
    pub corner_centers2: [f32; 4], // (bl_center.x, bl_center.y, br_center.x, br_center.y)
    pub corner_radii: [f32; 4], // (inner_tl_radius, inner_tr_radius, inner_bl_radius, inner_br_radius)
    pub corner_radii2: [f32; 4], // (outer_tl_radius, outer_tr_radius, outer_bl_radius, outer_br_radius)
    // Shadow properties
    pub shadow_color: [f32; 4],  // Shadow color with alpha
    pub shadow_offset: [f32; 2], // Shadow offset (x, y)
    pub shadow_blur: f32,        // Shadow blur radius
    pub shadow_opacity: f32,     // Shadow opacity
    // Clipping bounds
    pub clip_bounds: [f32; 4],        // (min_x, min_y, max_x, max_y)
    pub clip_border_radius: [f32; 4], // (top-left, top-right, bottom-left, bottom-right)
    pub clip_enabled: [f32; 2],       // (clip_x, clip_y) as 0.0 or 1.0
    pub _padding3: [f32; 8],          // Padding for alignment
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientType {
    Linear,
    Radial,
    // Can be extended with Conic later
}

#[derive(Debug, Clone)]
pub struct GradientColorStop {
    pub color: Color,
    pub position: f32, // 0.0 to 1.0 representing the position along the gradient line
}

/// This enum is used to access certain named entities via their EntityId.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Hash, EntityCategories)]
pub enum NamedRef {
    #[modal]
    SettingsModal,
    #[player]
    CurrentSongAlbumArt,
}

pub trait ModalEntity {
    fn is_modal(&self) -> bool;
}

pub trait PlayerEntity {
    fn is_player_entity(&self) -> bool;
}

// Component trait - data containers
pub trait EcsComponent: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// System trait - logic processors
pub trait EcsSystem {
    fn run(&mut self, world: &mut World);
}

// Resource trait - global data
pub trait EcsResource: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct EcsComponents {
    inner: HashMap<TypeId, HashMap<EntityId, Box<dyn EcsComponent>>>,
}

impl EcsComponents {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn get_component<T: EcsComponent + 'static>(&self, entity_id: EntityId) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.inner
            .get(&type_id)
            .and_then(|entity_map| entity_map.get(&entity_id))
            .and_then(|boxed_component| boxed_component.as_any().downcast_ref::<T>())
    }

    pub fn get_component_mut<T: EcsComponent + 'static>(
        &mut self,
        entity_id: EntityId,
    ) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.inner
            .get_mut(&type_id)
            .and_then(|entity_map| entity_map.get_mut(&entity_id))
            .and_then(|boxed_component| boxed_component.as_any_mut().downcast_mut::<T>())
    }

    pub fn get_components_mut_pair<T1: EcsComponent + 'static, T2: EcsComponent + 'static>(
        &mut self,
        entity_id: EntityId,
    ) -> Option<(&mut T1, &mut T2)> {
        let type1_id = TypeId::of::<T1>();
        let type2_id = TypeId::of::<T2>();
        let type_ids = [&type1_id, &type2_id];

        match self.inner.get_disjoint_mut(type_ids) {
            [Some(map1), Some(map2)] => {
                let comp1 = map1
                    .get_mut(&entity_id)
                    .and_then(|boxed| boxed.as_any_mut().downcast_mut::<T1>());
                let comp2 = map2
                    .get_mut(&entity_id)
                    .and_then(|boxed| boxed.as_any_mut().downcast_mut::<T2>());
                if let (Some(c1), Some(c2)) = (comp1, comp2) {
                    Some((c1, c2))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn entry(&mut self, type_id: TypeId) -> &mut HashMap<EntityId, Box<dyn EcsComponent>> {
        self.inner.entry(type_id).or_default()
    }

    pub fn get(&self, type_id: &TypeId) -> Option<&HashMap<EntityId, Box<dyn EcsComponent>>> {
        self.inner.get(type_id)
    }
}

pub struct EcsResources {
    inner: HashMap<TypeId, Box<dyn EcsResource>>,
}

impl EcsResources {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get_resource<T: EcsResource + 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.inner
            .get(&type_id)
            .and_then(|boxed_resource| boxed_resource.as_any().downcast_ref::<T>())
    }

    pub fn get_resource_mut<T: EcsResource + 'static>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.inner
            .get_mut(&type_id)
            .and_then(|boxed_resource| boxed_resource.as_any_mut().downcast_mut::<T>())
    }

    pub fn remove_resource<T: EcsResource + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.inner.remove(&type_id);
    }
}

// World - main ECS container
pub struct World {
    entities: Vec<EntityId>,
    pub components: EcsComponents,
    pub resources: EcsResources,
    pub named_entities: HashMap<NamedRef, EntityId>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            components: EcsComponents::new(),
            resources: EcsResources::new(),
            named_entities: HashMap::new(),
        }
    }

    pub fn queue_event(&mut self, event: AppEvent) {
        self.resources
            .get_resource_mut::<EventSenderResource>()
            .expect("Expected EventSenderResource to exist")
            .event_sender
            .send(event)
            .ok();
    }

    pub fn get_entities_with_component<T: EcsComponent + 'static>(&self) -> Vec<EntityId> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .map_or_else(Vec::new, |entity_map| entity_map.keys().cloned().collect())
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn initialize_resources(
        &mut self,
        queue: &wgpu::Queue,
        event_sender: &UnboundedSender<AppEvent>,
    ) {
        self.add_resource(WgpuQueueResource {
            queue: std::sync::Arc::new(queue.clone()),
        });
        self.add_resource(EventSenderResource {
            event_sender: event_sender.clone(),
        });
        self.initialize_resetable_resources();
    }

    fn initialize_resetable_resources(&mut self) {
        self.add_resource(RenderOrderResource::default());
        self.add_resource(RenderGroupsResource::default());
        self.add_resource(MouseResource::default());
        self.add_resource(RequestReLayoutResource::default());
        self.add_resource(TextRenderingResource::default());
    }

    fn reset_resources(&mut self) {
        self.resources.remove_resource::<RenderOrderResource>();
        self.resources.remove_resource::<RenderGroupsResource>();
        self.resources.remove_resource::<MouseResource>();
        self.resources.remove_resource::<TextRenderingResource>();
        self.initialize_resetable_resources();
    }

    pub fn create_entity(&mut self, debug_name: String, component_type: ComponentType) -> EntityId {
        let entity_id = Uuid::new_v4();
        self.entities.push(entity_id);

        // check if any other entity has the same debug name
        // TODO: I know this is not the best way to do it, but it works for now
        self.for_each_component::<IdentityComponent, _>(|_, component| {
            if component.debug_name == debug_name {
                panic!("Entity with debug name {} already exists", debug_name);
            }
        });

        // Add IdentityComponent by default
        self.add_component(
            entity_id,
            IdentityComponent {
                debug_name: debug_name.clone(),
                component_type,
            },
        );

        let truncated_name = if debug_name.len() > 40 {
            format!("{}...", &debug_name[..37])
        } else {
            debug_name.clone()
        };

        log::trace!(
            "New Entity | {:<40} | ID: {}",
            truncated_name,
            entity_id,
        );
        entity_id
    }

    pub fn reset(&mut self) {
        self.entities.clear();
        self.components.clear();
        self.named_entities.clear();
        self.reset_resources();
        log::trace!("World reset");
    }

    pub fn add_component<T: EcsComponent>(&mut self, entity_id: EntityId, component: T) {
        if !self.entities.contains(&entity_id) {
            panic!(
                "Tried to add component to non existent entity with Entity ID {}",
                entity_id
            );
        }
        if self
            .components
            .get(&TypeId::of::<T>())
            .and_then(|entity_map| entity_map.get(&entity_id))
            .is_some()
        {
            panic!("Entity already has a component of this type");
        }
        let type_id = TypeId::of::<T>();
        let entity_map = self.components.entry(type_id);
        entity_map.insert(entity_id, Box::new(component));
    }

    pub fn add_resource<T: EcsResource>(&mut self, resource: T) {
        log::trace!("Adding resource: {:?}", std::any::type_name::<T>());
        let type_id = TypeId::of::<T>();
        self.resources.inner.insert(type_id, Box::new(resource));
    }

    pub fn run_system<S: EcsSystem>(&mut self, mut system: S) {
        system.run(self);
    }

    pub fn for_each_component_mut<T: EcsComponent + 'static, F: FnMut(EntityId, &mut T)>(
        &mut self,
        mut f: F,
    ) {
        let type_id = TypeId::of::<T>();
        if let Some(entity_map) = self.components.inner.get_mut(&type_id) {
            for (entity_id, component) in entity_map {
                if let Some(typed_component) = component.as_any_mut().downcast_mut::<T>() {
                    f(*entity_id, typed_component);
                }
            }
        }
    }

    pub fn for_each_component<T: EcsComponent + 'static, F: FnMut(EntityId, &T)>(&self, mut f: F) {
        let type_id = TypeId::of::<T>();
        if let Some(entity_map) = self.components.get(&type_id) {
            for (entity_id, component) in entity_map {
                if let Some(typed_component) = component.as_any().downcast_ref::<T>() {
                    f(*entity_id, typed_component);
                }
            }
        }
    }

    pub fn query_combined_2<T: EcsComponent + 'static, U: EcsComponent + 'static>(
        &self,
    ) -> Vec<(EntityId, &T, &U)> {
        let type_id_t = TypeId::of::<T>();
        let type_id_u = TypeId::of::<U>();

        let mut result = Vec::new();

        if let (Some(t_map), Some(u_map)) = (
            self.components.get(&type_id_t),
            self.components.get(&type_id_u),
        ) {
            // Find entities that have both component types
            for (entity_id, t_component) in t_map {
                if let Some(u_component) = u_map.get(entity_id) {
                    if let (Some(t), Some(u)) = (
                        t_component.as_any().downcast_ref::<T>(),
                        u_component.as_any().downcast_ref::<U>(),
                    ) {
                        result.push((*entity_id, t, u));
                    }
                }
            }
        }

        result
    }

    pub fn query_combined_3<
        T: EcsComponent + 'static,
        U: EcsComponent + 'static,
        V: EcsComponent + 'static,
    >(
        &self,
    ) -> Vec<(EntityId, &T, &U, &V)> {
        let type_id_t = TypeId::of::<T>();
        let type_id_u = TypeId::of::<U>();
        let type_id_v = TypeId::of::<V>();

        let mut result = Vec::new();

        if let (Some(t_map), Some(u_map), Some(v_map)) = (
            self.components.get(&type_id_t),
            self.components.get(&type_id_u),
            self.components.get(&type_id_v),
        ) {
            // Find entities that have all three component types
            for (entity_id, t_component) in t_map {
                if let Some(u_component) = u_map.get(entity_id) {
                    if let Some(v_component) = v_map.get(entity_id) {
                        if let (Some(t), Some(u), Some(v)) = (
                            t_component.as_any().downcast_ref::<T>(),
                            u_component.as_any().downcast_ref::<U>(),
                            v_component.as_any().downcast_ref::<V>(),
                        ) {
                            result.push((*entity_id, t, u, v));
                        }
                    }
                }
            }
        }

        result
    }
}
