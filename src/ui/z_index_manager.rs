use super::ecs::EntityId;
use std::collections::{HashMap, HashSet};
#[derive(Debug)]
pub struct ZIndexManager {
    /// Base z-index assigned to each component
    base_indices: HashMap<EntityId, i32>,
    /// Manual adjustment applied to component (relative to siblings)
    adjustments: HashMap<EntityId, i32>,
    /// Component hierarchy mapping (child -> parent)
    hierarchy: HashMap<EntityId, Option<EntityId>>,
    /// Cache of computed absolute z-indices
    computed_indices: HashMap<EntityId, i32>,
    /// Z-index increment between hierarchy levels
    level_increment: i32,
    /// Whether the z-indices need to be recomputed
    dirty: bool,
    /// Root entity ID
    root_id: Option<EntityId>,
    /// Registration order for stable sorting
    registration_order: HashMap<EntityId, usize>,
    /// Counter for registration order
    registration_counter: usize,
}

impl Default for ZIndexManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ZIndexManager {
    pub fn new() -> Self {
        Self {
            base_indices: HashMap::new(),
            adjustments: HashMap::new(),
            hierarchy: HashMap::new(),
            computed_indices: HashMap::new(),
            level_increment: 1000, // Large increment to allow for many adjustments at each level
            dirty: true,
            root_id: None,
            registration_order: HashMap::new(),
            registration_counter: 0,
        }
    }

    pub fn clear(&mut self) {
        self.base_indices.clear();
        self.adjustments.clear();
        self.hierarchy.clear();
        self.computed_indices.clear();
        self.dirty = true;
        self.root_id = None;
        self.registration_order.clear();
        self.registration_counter = 0;
    }

    pub fn set_root_id(&mut self, root_id: EntityId) {
        self.root_id = Some(root_id);
    }

    /// Check if adding a parent relationship would create a cycle
    fn would_create_cycle(&self, component_id: EntityId, parent_id: EntityId) -> bool {
        let mut current = Some(parent_id);
        let mut visited = HashSet::new();

        while let Some(id) = current {
            // If we've seen this ID before, we have a cycle
            if !visited.insert(id) || id == component_id {
                return true;
            }

            current = self.hierarchy.get(&id).copied().flatten();
        }

        false
    }

    /// Register a component and its parent relationship
    pub fn register_component(&mut self, component_id: EntityId, parent_id: Option<EntityId>) {
        // Check if already registered
        if self.base_indices.contains_key(&component_id) {
            panic!("Warning: Attempted to register an already registered component");
        }
        // Check for cycles when setting a parent
        if let Some(pid) = parent_id {
            if pid == component_id || self.would_create_cycle(component_id, pid) {
                panic!("Warning: Attempted to create a cycle in component hierarchy");
            }
        }

        self.hierarchy.insert(component_id, parent_id);

        // Only set base index if not already set
        self.base_indices.entry(component_id).or_insert(0);

        // Track registration order
        self.registration_order
            .insert(component_id, self.registration_counter);
        self.registration_counter += 1;

        self.dirty = true;
    }

    /// Set a manual z-index adjustment for a component (relative to siblings)
    pub fn set_adjustment(&mut self, component_id: EntityId, adjustment: i32) {
        self.adjustments.insert(component_id, adjustment);
        self.dirty = true;
    }

    /// Compute z-indices for all registered components
    fn compute_all_z_indices(&mut self) {
        self.computed_indices.clear();
        self.compute_component_z_index(
            self.root_id
                .expect("Expected Root ID to be set before computing z indices"),
            0,
        );
        self.dirty = false;
    }

    /// Recursively compute z-index for a component and its descendants
    fn compute_component_z_index(&mut self, component_id: EntityId, depth: i32) {
        // Calculate absolute z-index:
        // depth * level_increment + base_index + adjustment
        let base = self.base_indices.get(&component_id).cloned().unwrap_or(0);
        let adjustment = self.adjustments.get(&component_id).cloned().unwrap_or(0);
        let parent_adjustment =
            if let Some(parent_id) = self.hierarchy.get(&component_id).unwrap_or(&None) {
                // Inherit parent's adjustment to ensure children maintain relative position
                self.adjustments.get(parent_id).cloned().unwrap_or(0)
            } else {
                0
            };
        // Parent's adjustment is factored in to ensure all children inherit parent's positioning
        let absolute_z_index = depth * self.level_increment + base + adjustment + parent_adjustment;

        // Store the computed z-index
        self.computed_indices.insert(component_id, absolute_z_index);

        // Find and process all children
        let children: Vec<EntityId> = self
            .hierarchy
            .iter()
            .filter_map(|(id, parent)| {
                if *parent == Some(component_id) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        // Process children with the next depth level
        for child_id in children {
            self.compute_component_z_index(child_id, depth + 1);
        }
    }

    /// Update rendering order based on z-indices
    pub fn generate_render_order(&mut self) -> Vec<EntityId> {
        if self.dirty {
            self.compute_all_z_indices();
        }

        // We'll build the render order in a hierarchical way
        let mut render_order = Vec::new();

        if let Some(root_id) = self.root_id {
            // Start with the root
            render_order.push(root_id);

            // Then add all elements in hierarchical order
            self.add_children_to_render_order(root_id, &mut render_order);
        }

        render_order
    }

    /// Helper function to recursively add children to render order
    fn add_children_to_render_order(&self, parent_id: EntityId, render_order: &mut Vec<EntityId>) {
        // Find all direct children of this parent
        let mut children: Vec<(EntityId, i32, usize)> = self
            .hierarchy
            .iter()
            .filter_map(|(id, parent)| {
                if *parent == Some(parent_id) {
                    let z_index = *self.computed_indices.get(id).unwrap_or(&0);
                    let registration_order =
                        *self.registration_order.get(id).unwrap_or(&usize::MAX);
                    Some((*id, z_index, registration_order))
                } else {
                    None
                }
            })
            .collect();

        // Sort children by z-index first, then by registration order
        children.sort_by(|a, b| {
            let (_, z_a, reg_a) = a;
            let (_, z_b, reg_b) = b;

            z_a.cmp(z_b).then_with(|| reg_a.cmp(reg_b))
        });

        // Add each child to the render order and then recursively add its children
        for (child_id, _, _) in children {
            render_order.push(child_id);
            self.add_children_to_render_order(child_id, render_order);
        }
    }
}
