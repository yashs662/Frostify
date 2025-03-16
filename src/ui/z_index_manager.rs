use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct ZIndexManager {
    /// Base z-index assigned to each component
    base_indices: HashMap<Uuid, i32>,
    /// Manual adjustment applied to component (relative to siblings)
    adjustments: HashMap<Uuid, i32>,
    /// Component hierarchy mapping (child -> parent)
    hierarchy: HashMap<Uuid, Option<Uuid>>,
    /// Cache of computed absolute z-indices
    computed_indices: HashMap<Uuid, i32>,
    /// Z-index increment between hierarchy levels
    level_increment: i32,
    /// Whether the z-indices need to be recomputed
    dirty: bool,
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
        }
    }

    pub fn clear(&mut self) {
        self.base_indices.clear();
        self.adjustments.clear();
        self.hierarchy.clear();
        self.computed_indices.clear();
        self.dirty = true;
    }

    /// Register a component and its parent relationship
    pub fn register_component(&mut self, component_id: Uuid, parent_id: Option<Uuid>) {
        self.hierarchy.insert(component_id, parent_id);

        // Only set base index if not already set
        self.base_indices.entry(component_id).or_insert(0);

        self.dirty = true;
    }

    /// Set a manual z-index adjustment for a component (relative to siblings)
    pub fn set_adjustment(&mut self, component_id: Uuid, adjustment: i32) {
        self.adjustments.insert(component_id, adjustment);
        self.dirty = true;
    }

    /// Get the computed absolute z-index for a component
    pub fn get_z_index(&mut self, component_id: &Uuid) -> i32 {
        if self.dirty {
            self.compute_all_z_indices();
        }

        *self.computed_indices.get(component_id).unwrap_or(&0)
    }

    /// Compute z-indices for all registered components
    fn compute_all_z_indices(&mut self) {
        self.computed_indices.clear();

        // Find root components (those without parents)
        let root_components: Vec<Uuid> = self
            .hierarchy
            .iter()
            .filter_map(|(id, parent_id)| if parent_id.is_none() { Some(*id) } else { None })
            .collect();

        // Process each root component and its descendants
        for root_id in root_components {
            self.compute_component_z_index(root_id, 0);
        }

        self.dirty = false;
    }

    /// Recursively compute z-index for a component and its descendants
    fn compute_component_z_index(&mut self, component_id: Uuid, depth: i32) {
        // Calculate absolute z-index:
        // depth * level_increment + base_index + adjustment
        let base = self.base_indices.get(&component_id).cloned().unwrap_or(0);
        let adjustment = self.adjustments.get(&component_id).cloned().unwrap_or(0);
        let absolute_z_index = depth * self.level_increment + base + adjustment;

        // Store the computed z-index
        self.computed_indices.insert(component_id, absolute_z_index);

        // Find and process all children
        let children: Vec<Uuid> = self
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
    pub fn sort_render_order(&mut self) -> Vec<Uuid> {
        if self.dirty {
            self.compute_all_z_indices();
        }

        // Create a list of (id, z-index) pairs for all components
        let mut component_z_indices: Vec<(Uuid, i32)> = self
            .computed_indices
            .iter()
            .map(|(id, z_index)| (*id, *z_index))
            .collect();

        // Sort by z-index (low to high, so rendered bottom to top)
        component_z_indices.sort_by_key(|(_, z_index)| *z_index);

        // Extract just the IDs in sorted order
        component_z_indices.into_iter().map(|(id, _)| id).collect()
    }
}
