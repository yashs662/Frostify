use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use uuid::Uuid;

pub mod components;
pub mod integration;
pub mod resources;
pub mod systems;

pub type EntityId = Uuid;

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

// World - main ECS container
pub struct World {
    entities: Vec<EntityId>,
    components: HashMap<TypeId, HashMap<EntityId, Box<dyn EcsComponent>>>,
    resources: HashMap<TypeId, Box<dyn EcsResource>>,
    entity_generations: HashMap<EntityId, u32>,
    removed_entities: Vec<EntityId>,
}

impl Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("entities", &self.entities)
            .field("components", &self.components.keys())
            .field("resources", &self.resources.keys())
            .finish()
    }
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
            components: HashMap::new(),
            resources: HashMap::new(),
            entity_generations: HashMap::new(),
            removed_entities: Vec::new(),
        }
    }

    pub fn create_entity(&mut self) -> EntityId {
        let entity_id = Uuid::new_v4();
        self.entities.push(entity_id);
        self.entity_generations.insert(entity_id, 1);
        entity_id
    }

    pub fn delete_entity(&mut self, entity_id: EntityId) {
        if let Some(generation) = self.entity_generations.get_mut(&entity_id) {
            *generation += 1;
            self.removed_entities.push(entity_id);
        }
    }

    pub fn add_component<T: EcsComponent>(&mut self, entity_id: EntityId, component: T) {
        let type_id = TypeId::of::<T>();

        let entity_map = self.components.entry(type_id).or_default();

        entity_map.insert(entity_id, Box::new(component));
    }

    pub fn get_component<T: EcsComponent + 'static>(&self, entity_id: EntityId) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        self.components
            .get(&type_id)
            .and_then(|entity_map| entity_map.get(&entity_id))
            .and_then(|boxed_component| boxed_component.as_any().downcast_ref::<T>())
    }

    pub fn get_component_mut<T: EcsComponent + 'static>(
        &mut self,
        entity_id: EntityId,
    ) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();

        self.components
            .get_mut(&type_id)
            .and_then(|entity_map| entity_map.get_mut(&entity_id))
            .and_then(|boxed_component| boxed_component.as_any_mut().downcast_mut::<T>())
    }

    pub fn query_combined<T: EcsComponent + 'static, U: EcsComponent + 'static>(
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

    pub fn add_resource<T: EcsResource>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources.insert(type_id, Box::new(resource));
    }

    pub fn get_resource<T: EcsResource + 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();

        self.resources
            .get(&type_id)
            .and_then(|boxed_resource| boxed_resource.as_any().downcast_ref::<T>())
    }

    pub fn get_resource_mut<T: EcsResource + 'static>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();

        self.resources
            .get_mut(&type_id)
            .and_then(|boxed_resource| boxed_resource.as_any_mut().downcast_mut::<T>())
    }

    pub fn run_system<S: EcsSystem>(&mut self, mut system: S) {
        system.run(self);
    }

    pub fn cleanup(&mut self) {
        for entity_id in &self.removed_entities {
            self.entities.retain(|e| e != entity_id);
            for entity_map in self.components.values_mut() {
                entity_map.remove(entity_id);
            }
        }
        self.removed_entities.clear();
    }

    pub fn query<T: EcsComponent + 'static>(&self) -> Vec<(EntityId, &T)> {
        let type_id = TypeId::of::<T>();

        let mut result = Vec::new();
        if let Some(entity_map) = self.components.get(&type_id) {
            for (entity_id, component) in entity_map {
                if let Some(typed_component) = component.as_any().downcast_ref::<T>() {
                    result.push((*entity_id, typed_component));
                }
            }
        }
        result
    }

    pub fn query_mut<T: EcsComponent + 'static>(&mut self) -> Vec<(EntityId, &mut T)> {
        let type_id = TypeId::of::<T>();

        let mut result = Vec::new();
        if let Some(entity_map) = self.components.get_mut(&type_id) {
            for (entity_id, component) in entity_map {
                if let Some(typed_component) = component.as_any_mut().downcast_mut::<T>() {
                    result.push((*entity_id, typed_component));
                }
            }
        }
        result
    }
}
