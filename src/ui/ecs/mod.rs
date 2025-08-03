use crate::{
    app::AppEvent,
    ui::{
        color::Color,
        ecs::resources::{EntryExitAnimationStateResource, NamedRefsResource},
    },
    utils::AppFonts,
};
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

pub mod builders;
pub mod components;
pub mod resources;
pub mod systems;

/// Ultra-efficient entity ID using sequential integers
/// This eliminates UUID overhead and enables direct array indexing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId {
    pub index: u32,
    pub generation: u32,
}

impl EntityId {
    pub fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.index, self.generation)
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ComponentType {
    Container,
    Text,
    Image,
    BackgroundColor,
    BackgroundGradient,
    FrostedGlass,
}

impl ComponentType {
    pub fn is_renderable(&self) -> bool {
        !matches!(self, ComponentType::Container)
    }
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
    pub color: [f32; 4],         // Base Color (RGBA)
    pub position: [f32; 2],      // Position in pixels (top-left corner)
    pub size: [f32; 2],          // Size in pixels (width, height)
    pub border_radius: [f32; 4], // Corner radii in pixels (top-left, top-right, bottom-left, bottom-right)
    pub screen_size: [f32; 2],   // Viewport dimensions in pixels
    pub use_texture: u32,        // Flag: 0 for color, 1 for texture, 2 for frosted glass
    pub blur_radius: f32,        // Blur amount for frosted glass (0-10)
    pub opacity: f32,            // Overall opacity for frosted glass (0.0-1.0)
    pub tint_intensity: f32,
    pub border_width: f32,            // Border thickness in pixels
    pub border_position: u32,         // Border position: 0=inside, 1=center, 2=outside
    pub border_color: [f32; 4],       // Border color
    pub bounds_with_border: [f32; 4], // (outer_min.x, outer_min.y, outer_max.x, outer_max.y)
    // Shadow properties
    pub shadow_color: [f32; 4],  // Shadow color with alpha
    pub shadow_offset: [f32; 2], // Shadow offset (x, y)
    pub shadow_blur: f32,        // Shadow blur radius
    pub shadow_opacity: f32,     // Shadow opacity
    // Clipping bounds
    pub clip_bounds: [f32; 4],        // (min_x, min_y, max_x, max_y)
    pub clip_border_radius: [f32; 4], // (top-left, top-right, bottom-left, bottom-right)
    pub clip_enabled: [f32; 2],       // (clip_x, clip_y) as 0.0 or 1.0
    // Notch properties
    pub notch_edge: u32,        // 0=disabled, 1=top, 2=right, 3=bottom, 4=left
    pub notch_depth: f32,       // Depth of notch in pixels
    pub notch_flat_width: f32,  // Flat width of notch in pixels
    pub notch_total_width: f32, // Total width of notch in pixels
    pub notch_offset: f32,      // Offset along edge in pixels (positive values move right/down)
    pub notch_position: u32,    // 0=left, 1=center, 2=right
    pub _padding: [f32; 2],     // Padding for alignment
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

/// Ultra-efficient component storage using direct array indexing
/// No more HashMap lookups - direct O(1) array access using entity index
pub struct ComponentStorage<T: EcsComponent> {
    /// Dense array of components indexed directly by entity index
    components: Vec<Option<T>>,
    /// Generation counter to match entity generations for safety
    generations: Vec<u32>,
    /// List of free indices for reuse
    free_indices: Vec<u32>,
}

/// Iterator for mutable component access
pub struct IterMut<'a, T> {
    components: std::iter::Enumerate<std::slice::IterMut<'a, Option<T>>>,
    generations: &'a [u32],
}

impl<'a, T: EcsComponent> Iterator for IterMut<'a, T> {
    type Item = (EntityId, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        for (index, component) in self.components.by_ref() {
            if let Some(comp) = component.as_mut() {
                let generation = self.generations[index];
                return Some((EntityId::new(index as u32, generation), comp));
            }
        }
        None
    }
}

impl<T: EcsComponent> ComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            generations: Vec::new(),
            free_indices: Vec::new(),
        }
    }

    /// Ensure storage can accommodate the given entity index
    fn ensure_capacity(&mut self, index: usize) {
        if index >= self.components.len() {
            // Use a growth factor (double the current length or at least index+1)
            let current_len = self.components.len();
            let mut new_len = current_len.max(1);
            while new_len <= index {
                new_len *= 2;
            }
            self.components.resize_with(new_len, || None);
            self.generations.resize(new_len, 0);
        }
    }

    pub fn insert(&mut self, entity: EntityId, component: T) -> bool {
        let index = entity.index as usize;

        // Ensure we have enough capacity
        self.ensure_capacity(index);

        // Check if entity generation matches (prevents use-after-free with recycled IDs)
        if self.generations[index] != entity.generation {
            return false;
        }

        // Check if component already exists
        if self.components[index].is_some() {
            return false;
        }

        self.components[index] = Some(component);
        true
    }

    pub fn get(&self, entity: EntityId) -> Option<&T> {
        let index = entity.index as usize;

        if index >= self.components.len() {
            return None;
        }

        // Verify generation matches
        if self.generations[index] != entity.generation {
            return None;
        }

        self.components[index].as_ref()
    }

    pub fn get_mut(&mut self, entity: EntityId) -> Option<&mut T> {
        let index = entity.index as usize;

        if index >= self.components.len() {
            return None;
        }

        // Verify generation matches
        if self.generations[index] != entity.generation {
            return None;
        }

        self.components[index].as_mut()
    }

    pub fn remove(&mut self, entity: EntityId) -> Option<T> {
        let index = entity.index as usize;

        if index >= self.components.len() {
            return None;
        }

        // Verify generation matches
        if self.generations[index] != entity.generation {
            return None;
        }

        self.components[index].take()
    }

    /// Iterate over all components (skips None slots automatically)
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> + '_ {
        self.components
            .iter()
            .enumerate()
            .filter_map(move |(index, component)| {
                component
                    .as_ref()
                    .map(|comp| (EntityId::new(index as u32, self.generations[index]), comp))
            })
    }

    /// Mutable iteration over all components
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            components: self.components.iter_mut().enumerate(),
            generations: &self.generations,
        }
    }

    pub fn len(&self) -> usize {
        self.components.iter().filter(|c| c.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.components.iter().all(|c| c.is_none())
    }

    pub fn clear(&mut self) {
        self.components.clear();
        self.generations.clear();
        self.free_indices.clear();
    }

    /// Mark an entity slot as free for the given generation
    pub fn mark_entity_removed(&mut self, entity: EntityId) {
        let index = entity.index as usize;
        if index < self.generations.len() {
            // Increment generation to invalidate old entity references
            self.generations[index] = self.generations[index].wrapping_add(1);
            self.free_indices.push(entity.index);
        }
    }

    /// Check if entity exists in this storage
    pub fn has_entity(&self, entity: EntityId) -> bool {
        let index = entity.index as usize;
        index < self.components.len()
            && self.generations[index] == entity.generation
            && self.components[index].is_some()
    }
}

/// Type-erased component storage for dynamic access
trait ComponentStorageErased: Send + Sync {
    fn remove_entity(&mut self, entity: EntityId) -> bool;
    fn has_entity(&self, entity: EntityId) -> bool;
    fn len(&self) -> usize;
    fn clear(&mut self);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: EcsComponent> ComponentStorageErased for ComponentStorage<T> {
    fn remove_entity(&mut self, entity: EntityId) -> bool {
        let removed = self.remove(entity).is_some();
        if removed {
            self.mark_entity_removed(entity);
        }
        removed
    }

    fn has_entity(&self, entity: EntityId) -> bool {
        self.has_entity(entity)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn clear(&mut self) {
        self.clear()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct EcsComponents {
    storages: HashMap<TypeId, Box<dyn ComponentStorageErased>>,
}

impl EcsComponents {
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        for storage in self.storages.values_mut() {
            storage.clear();
        }
    }

    /// Get or create storage for a component type
    fn get_storage<T: EcsComponent>(&mut self) -> &mut ComponentStorage<T> {
        let type_id = TypeId::of::<T>();
        self.storages
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentStorage::<T>::new()))
            .as_any_mut()
            .downcast_mut::<ComponentStorage<T>>()
            .unwrap()
    }

    /// Get storage for a component type (read-only)
    pub fn get_storage_ref<T: EcsComponent>(&self) -> Option<&ComponentStorage<T>> {
        let type_id = TypeId::of::<T>();
        self.storages
            .get(&type_id)?
            .as_any()
            .downcast_ref::<ComponentStorage<T>>()
    }

    pub fn add_component<T: EcsComponent>(&mut self, entity: EntityId, component: T) -> bool {
        self.get_storage::<T>().insert(entity, component)
    }

    pub fn get_component<T: EcsComponent + 'static>(&self, entity_id: EntityId) -> Option<&T> {
        self.get_storage_ref::<T>()?.get(entity_id)
    }

    pub fn get_component_mut<T: EcsComponent + 'static>(
        &mut self,
        entity_id: EntityId,
    ) -> Option<&mut T> {
        self.get_storage::<T>().get_mut(entity_id)
    }

    pub fn remove_component<T: EcsComponent>(&mut self, entity: EntityId) -> Option<T> {
        self.get_storage::<T>().remove(entity)
    }

    pub fn has_component<T: EcsComponent>(&self, entity: EntityId) -> bool {
        self.get_storage_ref::<T>()
            .map(|storage| storage.has_entity(entity))
            .unwrap_or(false)
    }

    /// Remove all components for an entity
    pub fn remove_entity(&mut self, entity: EntityId) {
        for storage in self.storages.values_mut() {
            storage.remove_entity(entity);
        }
    }

    /// Efficient iteration over single component type
    pub fn for_each<T: EcsComponent, F>(&self, mut f: F)
    where
        F: FnMut(EntityId, &T),
    {
        if let Some(storage) = self.get_storage_ref::<T>() {
            for (entity, component) in storage.iter() {
                f(entity, component);
            }
        }
    }

    /// Efficient mutable iteration over single component type
    pub fn for_each_mut<T: EcsComponent, F>(&mut self, mut f: F)
    where
        F: FnMut(EntityId, &mut T),
    {
        let storage = self.get_storage::<T>();
        for (entity, component) in storage.iter_mut() {
            f(entity, component);
        }
    }

    pub fn query_combined_2<T: EcsComponent + 'static, U: EcsComponent + 'static>(
        &self,
    ) -> Vec<(EntityId, &T, &U)> {
        let mut result = Vec::new();

        if let (Some(storage_t), Some(storage_u)) =
            (self.get_storage_ref::<T>(), self.get_storage_ref::<U>())
        {
            // Iterate over the smaller storage for efficiency
            if storage_t.len() <= storage_u.len() {
                for (entity, t_comp) in storage_t.iter() {
                    if let Some(u_comp) = storage_u.get(entity) {
                        result.push((entity, t_comp, u_comp));
                    }
                }
            } else {
                for (entity, u_comp) in storage_u.iter() {
                    if let Some(t_comp) = storage_t.get(entity) {
                        result.push((entity, t_comp, u_comp));
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
        let mut result = Vec::new();

        if let (Some(storage_t), Some(storage_u), Some(storage_v)) = (
            self.get_storage_ref::<T>(),
            self.get_storage_ref::<U>(),
            self.get_storage_ref::<V>(),
        ) {
            // Find smallest storage for iteration efficiency
            let storages = [
                (storage_t.len(), 0),
                (storage_u.len(), 1),
                (storage_v.len(), 2),
            ];
            let (_, smallest_idx) = storages.iter().min().unwrap();

            match smallest_idx {
                0 => {
                    for (entity, t_comp) in storage_t.iter() {
                        if let (Some(u_comp), Some(v_comp)) =
                            (storage_u.get(entity), storage_v.get(entity))
                        {
                            result.push((entity, t_comp, u_comp, v_comp));
                        }
                    }
                }
                1 => {
                    for (entity, u_comp) in storage_u.iter() {
                        if let (Some(t_comp), Some(v_comp)) =
                            (storage_t.get(entity), storage_v.get(entity))
                        {
                            result.push((entity, t_comp, u_comp, v_comp));
                        }
                    }
                }
                _ => {
                    for (entity, v_comp) in storage_v.iter() {
                        if let (Some(t_comp), Some(u_comp)) =
                            (storage_t.get(entity), storage_u.get(entity))
                        {
                            result.push((entity, t_comp, u_comp, v_comp));
                        }
                    }
                }
            }
        }

        result
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
    /// List of all entity IDs (for iteration)
    entities: Vec<EntityId>,
    /// Next entity index to assign
    next_entity_index: u32,
    /// Free entity indices for reuse
    free_entity_indices: Vec<u32>,
    /// Generation counter for each entity index
    entity_generations: Vec<u32>,
    pub components: EcsComponents,
    pub resources: EcsResources,
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
            next_entity_index: 0,
            free_entity_indices: Vec::new(),
            entity_generations: Vec::new(),
            components: EcsComponents::new(),
            resources: EcsResources::new(),
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
        if let Some(storage) = self.components.get_storage_ref::<T>() {
            storage.iter().map(|(entity, _)| entity).collect()
        } else {
            Vec::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Initialize all world resources including persistent and resettable ones
    pub fn initialize_resources(
        &mut self,
        queue: &wgpu::Queue,
        event_sender: &UnboundedSender<AppEvent>,
    ) {
        self.initialize_persistent_resources(queue, event_sender);
        self.initialize_resettable_resources();
    }

    /// Initialize persistent resources that should not be reset
    fn initialize_persistent_resources(
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
        self.add_resource(TextRenderingResource::with_custom_font_assets(vec![
            AppFonts::CenturyGothic.as_str(),
            AppFonts::CenturyGothicBold.as_str(),
        ]));
    }

    pub fn create_entity(&mut self, debug_name: String, component_type: ComponentType) -> EntityId {
        // Validate that the debug name is unique
        self.validate_unique_debug_name(&debug_name);

        // Get the next entity index (reuse if available)
        let index = if let Some(free_index) = self.free_entity_indices.pop() {
            free_index
        } else {
            let index = self.next_entity_index;
            self.next_entity_index += 1;
            index
        };

        // Ensure we have enough generation storage
        if index as usize >= self.entity_generations.len() {
            self.entity_generations.resize(index as usize + 1, 0);
        }

        let generation = self.entity_generations[index as usize];
        let entity_id = EntityId::new(index, generation);
        self.entities.push(entity_id);

        // Add IdentityComponent by default
        self.add_component(
            entity_id,
            IdentityComponent {
                debug_name: debug_name.clone(),
                component_type,
            },
        );

        self.log_entity_creation(&debug_name, entity_id);
        entity_id
    }

    fn validate_unique_debug_name(&self, debug_name: &str) {
        self.for_each_component::<IdentityComponent, _>(|_, component| {
            if component.debug_name == debug_name {
                panic!("Entity with debug name {debug_name} already exists");
            }
        });
    }

    fn log_entity_creation(&self, debug_name: &str, entity_id: EntityId) {
        let truncated_name = if debug_name.len() > 40 {
            format!("{}...", &debug_name[..37])
        } else {
            debug_name.to_string()
        };

        log::trace!("New Entity | {truncated_name:<40} | ID: {entity_id}");
    }

    pub fn reset(&mut self) {
        self.entities.clear();
        self.next_entity_index = 0;
        self.free_entity_indices.clear();
        self.entity_generations.clear();
        self.components.clear();
        self.reset_resettable_resources();
        log::trace!("World reset");
    }

    pub fn add_component<T: EcsComponent>(&mut self, entity_id: EntityId, component: T) {
        if !self.entities.contains(&entity_id) {
            panic!("Tried to add component to non existent entity with Entity ID {entity_id}");
        }
        if self.components.has_component::<T>(entity_id) {
            panic!("Entity already has a component of this type");
        }
        self.components.add_component(entity_id, component);
    }

    /// Remove an entity and all its components
    pub fn remove_entity(&mut self, entity_id: EntityId) {
        // Remove from entities list
        self.entities.retain(|&e| e != entity_id);

        // Remove all components for this entity
        self.components.remove_entity(entity_id);

        // Mark the entity index as free and increment generation
        let index = entity_id.index as usize;
        if index < self.entity_generations.len() {
            self.entity_generations[index] = self.entity_generations[index].wrapping_add(1);
            self.free_entity_indices.push(entity_id.index);
        }
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
        f: F,
    ) {
        self.components.for_each_mut::<T, _>(f);
    }

    pub fn for_each_component<T: EcsComponent + 'static, F: FnMut(EntityId, &T)>(&self, f: F) {
        self.components.for_each::<T, _>(f);
    }
}

/// Trait for managing resettable resources in the ECS world
pub trait ResourceManager {
    fn initialize_resettable_resources(&mut self);
    fn reset_resettable_resources(&mut self);
}

/// Helper macro to define resettable resources in one place
macro_rules! resettable_resources {
    ($($resource_type:ty),* $(,)?) => {
        impl ResourceManager for World {
            fn initialize_resettable_resources(&mut self) {
                $(
                    self.add_resource(<$resource_type>::default());
                )*
            }

            fn reset_resettable_resources(&mut self) {
                $(
                    self.resources.remove_resource::<$resource_type>();
                )*
                self.initialize_resettable_resources();
            }
        }
    };
}

// Define all resettable resources in one place
resettable_resources! {
    RenderOrderResource,
    RenderGroupsResource,
    MouseResource,
    RequestReLayoutResource,
    NamedRefsResource,
    EntryExitAnimationStateResource
}
