use crate::{
    app::AppEvent,
    ui::{
        component::{Component, ComponentType},
        component_update::CanProvideUpdates,
        components::slider::SliderBehavior,
        z_index_manager::ZIndexManager,
    },
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use log::{error, trace};
use std::collections::BTreeMap;
use uuid::Uuid;
use winit::event::MouseButton;

#[derive(Debug, Clone)]
pub struct ComponentOffset {
    pub x: FlexValue,
    pub y: FlexValue,
}

impl Default for ComponentOffset {
    fn default() -> Self {
        Self {
            x: 0.0.into(),
            y: 0.0.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ComponentSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ComponentPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Bounds {
    pub position: ComponentPosition,
    pub size: ComponentSize,
}

// FlexDirection enum
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

// JustifyContent enum
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

// AlignItems enum
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

// FlexWrap enum
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

// Enhanced FlexValue enum
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum FlexValue {
    Auto,
    Fixed(f32),
    Fraction(f32),                       // Similar to flex-grow in CSS
    Percentage(f32),                     // 0.0 to 1.0
    Viewport(f32),                       // Percentage of viewport
    Min(Box<FlexValue>, Box<FlexValue>), // min(a, b)
    Max(Box<FlexValue>, Box<FlexValue>), // max(a, b)
    Fit,                                 // fit-content
    #[default]
    Fill,                  // 100%
}

// BorderRadius struct
#[derive(Debug, Clone, Copy, Default)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

// Edges struct for padding, margin, and border
#[derive(Debug, Clone, Copy, Default)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

// Position enum for positioning system
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Position {
    #[default]
    Flex, // Default - follows flex layout rules
    Fixed(Anchor),      // Fixed position relative to parent
    Absolute(Anchor),   // Absolute position relative to root
    Grid(usize, usize), // Position in a grid (row, column)
}

// Anchor enum
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    #[default]
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

// ComponentTransform struct
#[derive(Debug, Clone)]
pub struct ComponentTransform {
    pub size: Size,
    pub offset: ComponentOffset,
    pub position_type: Position,
    pub z_index: i32,
    pub border_radius: BorderRadius,
    pub max_scale_factor: f32,
    pub min_scale_factor: f32,
    pub scale_factor: f32,
    pub scale_anchor: Anchor,
}

impl Default for ComponentTransform {
    fn default() -> Self {
        Self {
            size: Size::default(),
            offset: ComponentOffset::default(),
            position_type: Position::default(),
            z_index: 0,
            border_radius: BorderRadius::zero(),
            max_scale_factor: 1.0,
            min_scale_factor: 1.0,
            scale_factor: 1.0,
            scale_anchor: Anchor::default(),
        }
    }
}

// Size struct to replace ComponentSize
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct Size {
    pub width: FlexValue,
    pub height: FlexValue,
    pub min_width: FlexValue,
    pub min_height: FlexValue,
    pub max_width: FlexValue,
    pub max_height: FlexValue,
}

// Enhanced Layout struct
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Layout {
    // Flex container properties
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignItems,

    // Flex item properties
    pub align_self: Option<AlignItems>,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: FlexValue,
    pub order: i32,

    // Grid layout properties
    pub grid: Option<GridLayout>,

    // Spacing properties
    pub padding: Edges,
    pub margin: Edges,
    pub border: Edges,

    // Visibility and display
    pub visible: bool,
    pub opacity: f32,

    // Scrolling properties
    pub is_scrollable: bool,
    pub scroll_orientation: ScrollOrientation,
    pub scroll_position: f32,
    pub max_scroll: f32,

    // Overflow properties
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollOrientation {
    Vertical,
    Horizontal,
}

// Add Overflow enum after ScrollOrientation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
}

// LayoutContext to manage component relationships and computed layouts
#[derive(Debug, Default)]
pub struct LayoutContext {
    components: BTreeMap<Uuid, Component>,
    root_component_ids: Vec<Uuid>,
    computed_bounds: BTreeMap<Uuid, Bounds>,
    viewport_size: ComponentSize,
    render_order: Vec<Uuid>,
    initialized: bool,
    z_index_manager: ZIndexManager,
}

#[derive(Debug, Clone)]
pub struct GridLayout {
    pub columns: Vec<FlexValue>,
    pub rows: Vec<FlexValue>,
    pub column_gap: f32,
    pub row_gap: f32,
}

#[allow(dead_code)]
impl BorderRadius {
    pub fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    pub fn custom(top_left: f32, top_right: f32, bottom_left: f32, bottom_right: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    pub fn zero() -> Self {
        Self::all(0.0)
    }

    pub fn values(&self) -> [f32; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_left,
            self.bottom_right,
        ]
    }
}

// Event types
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    Press,
    Release,
    Drag,
    ScrollUp,
    ScrollDown,
    Hover,
    None,
}

// Event data
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InputEvent {
    pub event_type: EventType,
    pub position: Option<ComponentPosition>,
    pub button: Option<MouseButton>,
    pub key: Option<String>,
    pub text: Option<String>,
}

// Implementation for Size
#[allow(dead_code)]
impl Size {
    pub fn fixed(width: f32, height: f32) -> Self {
        Self {
            width: FlexValue::Fixed(width),
            height: FlexValue::Fixed(height),
            min_width: FlexValue::Auto,
            min_height: FlexValue::Auto,
            max_width: FlexValue::Auto,
            max_height: FlexValue::Auto,
        }
    }

    pub fn fill() -> Self {
        Self {
            width: FlexValue::Fill,
            height: FlexValue::Fill,
            min_width: FlexValue::Auto,
            min_height: FlexValue::Auto,
            max_width: FlexValue::Auto,
            max_height: FlexValue::Auto,
        }
    }

    pub fn fit() -> Self {
        Self {
            width: FlexValue::Fit,
            height: FlexValue::Fit,
            min_width: FlexValue::Auto,
            min_height: FlexValue::Auto,
            max_width: FlexValue::Auto,
            max_height: FlexValue::Auto,
        }
    }
}

// Implementation for Layout
#[allow(dead_code)]
impl Layout {
    pub fn new() -> Self {
        Self {
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            align_content: AlignItems::Start,
            align_self: None,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: FlexValue::Auto,
            grid: None,
            order: 0,
            padding: Edges::zero(),
            margin: Edges::zero(),
            border: Edges::zero(),
            visible: true,
            opacity: 1.0,
            is_scrollable: false,
            scroll_orientation: ScrollOrientation::Vertical,
            scroll_position: 0.0,
            max_scroll: 0.0,
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
        }
    }

    // Utility methods for creating common layout patterns

    pub fn flex_column() -> Self {
        let mut layout = Self::new();
        layout.direction = FlexDirection::Column;
        layout
    }

    pub fn flex_row() -> Self {
        let mut layout = Self::new();
        layout.direction = FlexDirection::Row;
        layout
    }

    // Tailwind-like utilities

    pub fn center() -> Self {
        let mut layout = Self::new();
        layout.justify_content = JustifyContent::Center;
        layout.align_items = AlignItems::Center;
        layout
    }

    pub fn space_between() -> Self {
        let mut layout = Self::new();
        layout.justify_content = JustifyContent::SpaceBetween;
        layout
    }

    pub fn grow() -> Self {
        let mut layout = Self::new();
        layout.flex_grow = 1.0;
        layout
    }

    pub fn shrink() -> Self {
        let mut layout = Self::new();
        layout.flex_shrink = 1.0;
        layout
    }

    pub fn with_padding(&mut self, padding: Edges) -> &mut Self {
        self.padding = padding;
        self
    }

    pub fn with_margin(&mut self, margin: Edges) -> &mut Self {
        self.margin = margin;
        self
    }

    pub fn grid(columns: Vec<FlexValue>, rows: Vec<FlexValue>) -> Self {
        let mut layout = Self::new();
        layout.grid = Some(GridLayout {
            columns,
            rows,
            column_gap: 0.0,
            row_gap: 0.0,
        });
        layout
    }

    // Make sure we have proper methods for calculating available space
    // that factor in both margin and padding
    pub fn get_available_width(&self, container_width: f32) -> f32 {
        container_width - self.padding.left - self.padding.right
    }

    pub fn get_available_height(&self, container_height: f32) -> f32 {
        container_height - self.padding.top - self.padding.bottom
    }

    // Ensure that position calculations include padding
    pub fn get_content_x(&self, container_x: f32) -> f32 {
        container_x + self.padding.left
    }

    pub fn get_content_y(&self, container_y: f32) -> f32 {
        container_y + self.padding.top
    }

    // Scrollable container helpers
    pub fn scrollable_vertical() -> Self {
        let mut layout = Self::flex_column();
        layout.is_scrollable = true;
        layout.scroll_orientation = ScrollOrientation::Vertical;
        layout.overflow_y = Overflow::Scroll;
        layout
    }

    pub fn scrollable_horizontal() -> Self {
        let mut layout = Self::flex_row();
        layout.is_scrollable = true;
        layout.scroll_orientation = ScrollOrientation::Horizontal;
        layout.overflow_x = Overflow::Scroll;
        layout
    }

    pub fn update_scroll_position(&mut self, delta: f32) -> bool {
        let old_position = self.scroll_position;
        self.scroll_position = (self.scroll_position + delta).clamp(0.0, self.max_scroll);
        old_position != self.scroll_position
    }

    // Add helper methods for overflow
    pub fn with_overflow(&mut self, overflow: Overflow) -> &mut Self {
        self.overflow_x = overflow;
        self.overflow_y = overflow;
        self
    }

    pub fn with_overflow_x(&mut self, overflow: Overflow) -> &mut Self {
        self.overflow_x = overflow;
        self
    }

    pub fn with_overflow_y(&mut self, overflow: Overflow) -> &mut Self {
        self.overflow_y = overflow;
        self
    }

    pub fn hidden_overflow() -> Self {
        let mut layout = Self::new();
        layout.overflow_x = Overflow::Hidden;
        layout.overflow_y = Overflow::Hidden;
        layout
    }
}

// Implementation for Edges
#[allow(dead_code)]
impl Edges {
    pub fn zero() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }

    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn horizontal(value: f32) -> Self {
        Self {
            top: 0.0,
            right: value,
            bottom: 0.0,
            left: value,
        }
    }

    pub fn vertical(value: f32) -> Self {
        Self {
            top: value,
            right: 0.0,
            bottom: value,
            left: 0.0,
        }
    }

    pub fn left(value: f32) -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: value,
        }
    }

    pub fn right(value: f32) -> Self {
        Self {
            top: 0.0,
            right: value,
            bottom: 0.0,
            left: 0.0,
        }
    }

    pub fn top(value: f32) -> Self {
        Self {
            top: value,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }

    pub fn bottom(value: f32) -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: value,
            left: 0.0,
        }
    }

    pub fn custom(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

// Implementation for FlexValue
impl FlexValue {
    pub fn resolve(&self, available_space: f32, viewport_dimension: f32) -> f32 {
        match self {
            FlexValue::Auto => available_space,
            FlexValue::Fixed(value) => *value,
            FlexValue::Fraction(frac) => available_space * frac,
            FlexValue::Percentage(perc) => available_space * perc,
            FlexValue::Viewport(perc) => viewport_dimension * perc,
            FlexValue::Min(a, b) => f32::min(
                a.resolve(available_space, viewport_dimension),
                b.resolve(available_space, viewport_dimension),
            ),
            FlexValue::Max(a, b) => f32::max(
                a.resolve(available_space, viewport_dimension),
                b.resolve(available_space, viewport_dimension),
            ),
            FlexValue::Fit => 0.0, // Need to calculate based on content
            FlexValue::Fill => available_space,
        }
    }
}

struct SpacingData<'a> {
    justify_content: JustifyContent,
    is_row: bool,
    content_space: Bounds,
    flex_children: &'a [(Uuid, &'a Component)],
    space_per_flex_unit: f32,
    total_margins: f32,
}

// Layout Context implementation
impl LayoutContext {
    pub fn initialize(&mut self, width: f32, height: f32) {
        self.viewport_size = ComponentSize { width, height };
        self.initialized = true;
        self.compute_layout();
    }

    pub fn clear(&mut self) {
        self.components.clear();
        self.computed_bounds.clear();
        self.render_order.clear();
        self.z_index_manager.clear();
        self.root_component_ids.clear();
    }

    #[cfg(test)]
    pub fn get_computed_bounds(&self) -> &BTreeMap<Uuid, Bounds> {
        &self.computed_bounds
    }

    pub fn draw_group(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        app_pipelines: &mut AppPipelines,
        group: Vec<Uuid>,
    ) {
        for id in group {
            self.draw_single(render_pass, app_pipelines, &id);
        }
    }

    pub fn update_components(&mut self, wgpu_ctx: &mut WgpuCtx, frame_time: f32) {
        let mut updates = Vec::new();
        let mut needs_relayout = false;
        // First pass: collect all updates from components
        for (_id, component) in self.components.iter_mut() {
            // Update component state and check for update requests
            if component.needs_update() && component.update(wgpu_ctx, frame_time) {
                needs_relayout = true;
            }

            // If component provides updates, collect them
            if component.has_updates() {
                if let Some(update_data) = component.get_update_data() {
                    updates.push(update_data);
                    component.reset_update_state();
                }
            }

            // Special case for sliders - store current bounds
            if component.is_a_slider() {
                component.update_track_bounds(component.computed_bounds);
            }
        }

        // Second pass: apply collected updates
        for update in updates {
            // Find the target component and apply the update
            if let Some(target_component) = self.components.get_mut(&update.target_id()) {
                update.apply(target_component, wgpu_ctx);
            }

            // Also apply to any additional target components
            for additional_id in update.additional_target_ids() {
                if let Some(additional_target) = self.components.get_mut(&additional_id) {
                    update.apply(additional_target, wgpu_ctx);
                }
            }
        }

        if needs_relayout {
            self.compute_layout_and_update_components(wgpu_ctx);
        }
    }

    // Helper method to draw a single component by ID
    pub fn draw_single(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        app_pipelines: &mut AppPipelines,
        component_id: &Uuid,
    ) {
        if let Some(component) = self.components.get_mut(component_id) {
            if !component.requires_to_be_drawn() {
                return;
            }

            if self.computed_bounds.contains_key(component_id) {
                // Update the component's z-index with the computed value from manager
                let computed_z = self.z_index_manager.get_z_index(component_id);
                component.transform.z_index = computed_z;

                component.draw(render_pass, app_pipelines);
            } else {
                error!(
                    "Computed bounds not found for component id: {}, unable to draw single component",
                    component_id
                );
            }
        } else {
            error!(
                "Component with id: {} not found for single rendering",
                component_id
            );
        }
    }

    pub fn get_render_order(&self) -> &Vec<Uuid> {
        &self.render_order
    }

    pub fn get_component_mut(&mut self, id: &Uuid) -> Option<&mut Component> {
        self.components.get_mut(id)
    }

    fn debug_print_component_insertion(&self, component: &Component) {
        if component.debug_name.is_some() {
            trace!(
                "Adding {:?} component '{}' with id {:?}",
                component.component_type,
                component.debug_name.as_ref().unwrap(),
                component.id
            );
        } else {
            trace!(
                "Adding component {:?} with position type {:?}",
                component.id, component.transform.position_type
            );
        }
    }

    pub fn add_component(&mut self, component: Component) {
        let component_id = component.id;
        let is_root_component = component.get_parent_id().is_none();
        let mut children = Vec::new();

        // Extract children if needed
        if component.requires_children_extraction() {
            if let Some(extracted_children) = component.get_children_from_metadata() {
                children = extracted_children.clone();
            } else {
                panic!(
                    "Component {:?} requires children extraction but none found",
                    component.debug_name
                );
            }
        }

        // Keep track of the children IDs in the parent's children vector
        let child_ids = children
            .iter()
            .map(|child| (child.id, child.component_type))
            .collect();

        // Register the component with the z-index manager
        self.z_index_manager
            .register_component(component_id, component.get_parent_id());

        // Register any manual z-index adjustment
        if component.transform.z_index != 0 {
            self.z_index_manager
                .set_adjustment(component_id, component.transform.z_index);
        }

        // Add the parent component first
        self.debug_print_component_insertion(&component);
        self.components.insert(component_id, component);
        if is_root_component {
            self.root_component_ids.push(component_id);
        }

        // Update the parent's children vector
        if let Some(parent) = self.components.get_mut(&component_id) {
            parent.children_ids = child_ids;
        }

        // Then recursively add all children
        for child in children {
            self.add_component(child);
        }
    }

    pub fn get_component(&self, id: &Uuid) -> Option<&Component> {
        self.components.get(id)
    }

    pub fn compute_layout_and_update_components(&mut self, wgpu_ctx: &mut WgpuCtx) {
        let mut re_layout_required = true;
        let mut re_layout_attempts = 0;
        let max_re_layout_attempts = 5;

        while re_layout_required && re_layout_attempts < max_re_layout_attempts {
            re_layout_required = false;
            re_layout_attempts += 1;

            // Compute layout for all components
            self.compute_layout();

            // Update all component positions with new screen size
            for (id, bounds) in self.computed_bounds.iter() {
                if let Some(component) = self.components.get_mut(id) {
                    component.set_position(wgpu_ctx, *bounds, self.viewport_size);

                    // If this is a slider, refresh it to ensure thumb and track positions are correct
                    if component.is_a_slider() {
                        component.update_track_bounds(component.computed_bounds);
                        component.refresh_slider();
                    }
                }
            }

            // check if any components have the can be resized to metadata if so resize them and calculate layout again, remove metadata
            for (_, component) in self.components.iter_mut() {
                if component.can_be_resized_to_metadata().is_some() {
                    re_layout_required = true;
                    component.resize_to_metadata();
                    component.remove_resize_metadata();
                }
            }
        }
    }

    pub fn resize_viewport(&mut self, wgpu_ctx: &mut WgpuCtx) {
        self.viewport_size = wgpu_ctx.get_screen_size();
        self.compute_layout_and_update_components(wgpu_ctx);
    }

    pub fn compute_layout(&mut self) {
        if !self.initialized {
            error!("Attempting to compute layout before initialization");
            return;
        }

        // Clear previous computed bounds
        self.computed_bounds.clear();

        // Compute layout for each root component
        for root_id in self.root_component_ids.clone() {
            self.compute_component_layout(&root_id, None);
        }

        // Use the z-index manager to determine render order
        self.render_order = self.z_index_manager.sort_render_order();
    }

    fn compute_component_layout(&mut self, component_id: &Uuid, parent_bounds: Option<Bounds>) {
        let component = match self.get_component(component_id) {
            Some(c) => c.clone(),
            None => {
                error!("Component {:?} not found during layout", component_id);
                return;
            }
        };

        // For root components with Position::Absolute, use viewport as available space
        let available_space = match (parent_bounds, component.transform.position_type) {
            (None, Position::Absolute(_)) => Bounds {
                position: ComponentPosition { x: 0.0, y: 0.0 },
                size: self.viewport_size,
            },
            (Some(bounds), _) => bounds,
            _ => Bounds {
                position: ComponentPosition { x: 0.0, y: 0.0 },
                size: self.viewport_size,
            },
        };

        // Calculate this component's bounds based on layout properties
        let bounds = self.calculate_bounds(&component, available_space);
        self.computed_bounds.insert(*component_id, bounds);

        // Determine clip bounds based on parent's overflow settings
        let clip_bounds = if let Some(parent_id) = &component.get_parent_id() {
            if let Some(parent) = self.get_component(parent_id) {
                // Use parent bounds as clip bounds when overflow is hidden
                if parent.layout.overflow_x == Overflow::Hidden
                    || parent.layout.overflow_y == Overflow::Hidden
                {
                    let parent_bounds = self
                        .computed_bounds
                        .get(parent_id)
                        .cloned()
                        .unwrap_or_default();
                    let content_space = Bounds {
                        position: ComponentPosition {
                            x: parent_bounds.position.x
                                + parent.layout.padding.left
                                + parent.layout.border.left,
                            y: parent_bounds.position.y
                                + parent.layout.padding.top
                                + parent.layout.border.top,
                        },
                        size: ComponentSize {
                            width: parent_bounds.size.width
                                - (parent.layout.padding.left
                                    + parent.layout.padding.right
                                    + parent.layout.border.left
                                    + parent.layout.border.right),
                            height: parent_bounds.size.height
                                - (parent.layout.padding.top
                                    + parent.layout.padding.bottom
                                    + parent.layout.border.top
                                    + parent.layout.border.bottom),
                        },
                    };

                    // Store clip bounds with parent's overflow setting
                    Some((
                        content_space,
                        parent.layout.overflow_x == Overflow::Hidden,
                        parent.layout.overflow_y == Overflow::Hidden,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Pass the clip bounds to the component only if it should be clipped
        if let Some(component) = self.components.get_mut(component_id) {
            if component.clip_self {
                component.clip_bounds = clip_bounds;
            } else {
                // If this component doesn't clip itself, clear any clip bounds
                component.clip_bounds = None;
            }
        }

        // If this is a container, layout its children
        if component.component_type == ComponentType::Container {
            self.layout_children(component_id, bounds);
        }
    }

    fn calculate_bounds(&self, component: &Component, available_space: Bounds) -> Bounds {
        match component.transform.position_type {
            Position::Flex => self.calculate_flex_bounds(component, available_space),
            Position::Fixed(anchor) => {
                self.calculate_fixed_bounds(component, available_space, anchor)
            }
            Position::Absolute(anchor) => self.calculate_absolute_bounds(component, anchor),
            Position::Grid(_, _) => self.calculate_grid_bounds(component, available_space),
        }
    }

    fn calculate_flex_bounds(&self, component: &Component, available_space: Bounds) -> Bounds {
        // For flex items, inherit full size from parent when not explicitly set
        let width = match &component.transform.size.width {
            FlexValue::Fixed(w) => *w,
            FlexValue::Fill | FlexValue::Auto => available_space.size.width,
            _ => available_space.size.width, // Default to parent width
        };
        let scaled_width = width * component.transform.scale_factor;
        let height = match &component.transform.size.height {
            FlexValue::Fixed(h) => *h,
            FlexValue::Fill | FlexValue::Auto => available_space.size.height,
            _ => available_space.size.height, // Default to parent height
        };
        let scaled_height = height * component.transform.scale_factor;
        // Base position (without offset)
        let position = ComponentPosition {
            x: available_space.position.x,
            y: available_space.position.y,
        };

        // Apply offset if needed
        let offset_x = component
            .transform
            .offset
            .x
            .resolve(available_space.size.width, self.viewport_size.width);
        let offset_y = component
            .transform
            .offset
            .y
            .resolve(available_space.size.height, self.viewport_size.height);

        // let (scale_offset_x, scale_offset_y) = calc_scale_anchor_offsets(
        //     component.transform.scale_anchor,
        //     width,
        //     height,
        //     scaled_width,
        //     scaled_height,
        // );

        // TODO: handle scale offset not affecting siblings,
        // should be added below to all the siblings together for the effect to work, currently we can only add it to the scaled component so it looks weird

        let final_position = ComponentPosition {
            x: position.x + offset_x,
            y: position.y + offset_y,
        };

        Bounds {
            position: final_position,
            size: ComponentSize {
                width: scaled_width,
                height: scaled_height,
            },
        }
    }

    fn calculate_fixed_bounds(
        &self,
        component: &Component,
        parent_bounds: Bounds,
        anchor: Anchor,
    ) -> Bounds {
        // Resolve component size
        let width = component
            .transform
            .size
            .width
            .resolve(parent_bounds.size.width, self.viewport_size.width);
        let scaled_width = width * component.transform.scale_factor;
        let height = component
            .transform
            .size
            .height
            .resolve(parent_bounds.size.height, self.viewport_size.height);
        let scaled_height = height * component.transform.scale_factor;

        // Calculate position based on anchor - without applying offset yet
        let position = match anchor {
            Anchor::TopLeft => ComponentPosition {
                x: parent_bounds.position.x,
                y: parent_bounds.position.y,
            },
            Anchor::Top => ComponentPosition {
                x: parent_bounds.position.x + (parent_bounds.size.width - width) / 2.0,
                y: parent_bounds.position.y,
            },
            Anchor::TopRight => ComponentPosition {
                x: parent_bounds.position.x + parent_bounds.size.width - width,
                y: parent_bounds.position.y,
            },
            Anchor::Left => ComponentPosition {
                x: parent_bounds.position.x,
                y: parent_bounds.position.y + (parent_bounds.size.height - height) / 2.0,
            },
            Anchor::Center => ComponentPosition {
                x: parent_bounds.position.x + (parent_bounds.size.width - width) / 2.0,
                y: parent_bounds.position.y + (parent_bounds.size.height - height) / 2.0,
            },
            Anchor::Right => ComponentPosition {
                x: parent_bounds.position.x + parent_bounds.size.width - width,
                y: parent_bounds.position.y + (parent_bounds.size.height - height) / 2.0,
            },
            Anchor::BottomLeft => ComponentPosition {
                x: parent_bounds.position.x,
                y: parent_bounds.position.y + parent_bounds.size.height - height,
            },
            Anchor::Bottom => ComponentPosition {
                x: parent_bounds.position.x + (parent_bounds.size.width - width) / 2.0,
                y: parent_bounds.position.y + parent_bounds.size.height - height,
            },
            Anchor::BottomRight => ComponentPosition {
                x: parent_bounds.position.x + parent_bounds.size.width - width,
                y: parent_bounds.position.y + parent_bounds.size.height - height,
            },
        };

        // Apply offset after anchor positioning
        let offset_x = component
            .transform
            .offset
            .x
            .resolve(parent_bounds.size.width, self.viewport_size.width);
        let offset_y = component
            .transform
            .offset
            .y
            .resolve(parent_bounds.size.height, self.viewport_size.height);

        // let (scale_offset_x, scale_offset_y) = calc_scale_anchor_offsets(
        //     component.transform.scale_anchor,
        //     width,
        //     height,
        //     scaled_width,
        //     scaled_height,
        // );

        // TODO: handle scale offset not affecting siblings,
        // should be added below to all the siblings together for the effect to work, currently we can only add it to the scaled component so it looks weird

        let final_position = ComponentPosition {
            x: position.x + offset_x,
            y: position.y + offset_y,
        };

        Bounds {
            position: final_position,
            size: ComponentSize {
                width: scaled_width,
                height: scaled_height,
            },
        }
    }

    fn calculate_absolute_bounds(&self, component: &Component, anchor: Anchor) -> Bounds {
        // For absolute positioning, we position relative to the viewport
        let parent_bounds = Bounds {
            position: ComponentPosition { x: 0.0, y: 0.0 },
            size: self.viewport_size,
        };

        self.calculate_fixed_bounds(component, parent_bounds, anchor)
    }

    fn calculate_grid_bounds(&self, component: &Component, available_space: Bounds) -> Bounds {
        if let Position::Grid(row, col) = component.transform.position_type {
            if let Some(parent_id) = &component.get_parent_id() {
                if let Some(parent) = self.get_component(parent_id) {
                    if let Some(grid) = &parent.layout.grid {
                        // Calculate cell position and size
                        let mut x = available_space.position.x;
                        let mut y = available_space.position.y;
                        let mut width = 0.0;
                        let mut height = 0.0;

                        // Calculate x position based on column
                        for i in 0..col.min(grid.columns.len()) {
                            if i > 0 {
                                x += grid.column_gap;
                            }
                            let col_width = grid.columns[i]
                                .resolve(available_space.size.width, self.viewport_size.width);
                            x += col_width;
                        }

                        // Get this column's width
                        if col < grid.columns.len() {
                            width = grid.columns[col]
                                .resolve(available_space.size.width, self.viewport_size.width);
                        }

                        // Calculate y position based on row
                        for i in 0..row.min(grid.rows.len()) {
                            if i > 0 {
                                y += grid.row_gap;
                            }
                            let row_height = grid.rows[i]
                                .resolve(available_space.size.height, self.viewport_size.height);
                            y += row_height;
                        }

                        // Get this row's height
                        if row < grid.rows.len() {
                            height = grid.rows[row]
                                .resolve(available_space.size.height, self.viewport_size.height);
                        }

                        return Bounds {
                            position: ComponentPosition { x, y },
                            size: ComponentSize { width, height },
                        };
                    }
                }
            }
        }

        // Fall back to flex layout if grid information is missing
        self.calculate_flex_bounds(component, available_space)
    }

    fn layout_children(&mut self, parent_id: &Uuid, parent_bounds: Bounds) {
        let parent = self.get_component(parent_id).unwrap().clone();
        let layout = &parent.layout;

        if parent.get_all_children_ids().is_empty() || !layout.visible {
            return;
        }

        // Calculate available space for children inside the parent's content area
        let content_space = Bounds {
            position: ComponentPosition {
                x: parent_bounds.position.x + layout.padding.left + layout.border.left,
                y: parent_bounds.position.y + layout.padding.top + layout.border.top,
            },
            size: ComponentSize {
                width: parent_bounds.size.width
                    - (layout.padding.left
                        + layout.padding.right
                        + layout.border.left
                        + layout.border.right),
                height: parent_bounds.size.height
                    - (layout.padding.top
                        + layout.padding.bottom
                        + layout.border.top
                        + layout.border.bottom),
            },
        };

        let self_components = self.components.clone();
        let mut children: Vec<(Uuid, &Component)> = parent
            .get_all_children_ids()
            .iter()
            .filter_map(|id| self_components.get(id).map(|component| (*id, component)))
            .collect();

        children.sort_by_key(|(_, comp)| comp.layout.order);

        // Split children into fixed/absolute and flex
        let (positioned_children, flex_children): (Vec<_>, Vec<_>) =
            children.into_iter().partition(|(_, child)| {
                matches!(
                    child.transform.position_type,
                    Position::Fixed(_) | Position::Absolute(_)
                )
            });

        let is_row = matches!(
            layout.direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );

        // First handle flex layout to establish container boundaries
        let mut total_fixed_size = 0.0;
        let mut total_flex_grow = 0.0;
        let mut num_auto_sized = 0;
        let mut num_flex_items = 0;
        let mut total_margins = 0.0;
        let mut content_size = 0.0; // To calculate max scroll

        // Only consider flex children for space calculations
        for (_, child) in &flex_children {
            // Apply scale factor if component requires relayout
            let scale_factor = child.transform.scale_factor;
            // Sum up margins in the main axis
            if is_row {
                total_margins += child.layout.margin.left + child.layout.margin.right;
                match &child.transform.size.width {
                    FlexValue::Fixed(w) => {
                        let scaled_width = w * scale_factor;
                        total_fixed_size += scaled_width;
                        content_size +=
                            scaled_width + child.layout.margin.left + child.layout.margin.right;
                    }
                    FlexValue::Fill => {
                        total_flex_grow += child.layout.flex_grow.max(1.0);
                        num_flex_items += 1;
                    }
                    FlexValue::Auto => num_auto_sized += 1,
                    _ => {}
                }
            } else {
                total_margins += child.layout.margin.top + child.layout.margin.bottom;
                match &child.transform.size.height {
                    FlexValue::Fixed(h) => {
                        let scaled_height = h * scale_factor;
                        total_fixed_size += scaled_height;
                        content_size +=
                            scaled_height + child.layout.margin.top + child.layout.margin.bottom;
                    }
                    FlexValue::Fill => {
                        total_flex_grow += child.layout.flex_grow.max(1.0);
                        num_flex_items += 1;
                    }
                    FlexValue::Auto => num_auto_sized += 1,
                    _ => {}
                }
            }
        }

        let main_axis_size = if is_row {
            content_space.size.width
        } else {
            content_space.size.height
        };

        // Subtract margins from available space before distributing
        let remaining_space = (main_axis_size - total_fixed_size - total_margins).max(0.0);
        let space_per_flex_unit = if total_flex_grow > 0.0 {
            remaining_space / total_flex_grow
        } else if num_auto_sized > 0 {
            remaining_space / num_auto_sized as f32
        } else {
            0.0
        };

        let spacing_data = SpacingData {
            justify_content: layout.justify_content,
            is_row,
            content_space,
            flex_children: &flex_children,
            space_per_flex_unit,
            total_margins,
        };

        // Calculate spacing based on justify_content for flex items only
        let (start_pos, spacing_between) = self.calculate_spacing(spacing_data);

        // Update max scroll value if container is scrollable
        if let Some(component) = self.components.get_mut(parent_id) {
            if component.layout.is_scrollable {
                // Calculate total content size
                let additional_flex_space = space_per_flex_unit * total_flex_grow;
                let total_content_size = if component.layout.is_scrollable {
                    content_size
                        + additional_flex_space
                        + (spacing_between * (flex_children.len() as f32 - 1.0))
                } else {
                    0.0
                };

                // Calculate max scroll based on orientation
                let container_size = if is_row {
                    content_space.size.width
                } else {
                    content_space.size.height
                };

                let max_scroll = (total_content_size - container_size).max(0.0);
                component.layout.max_scroll = max_scroll;
            }
        }

        // Get scroll position for this container
        let scroll_offset = if let Some(component) = self.components.get(parent_id) {
            if component.layout.is_scrollable {
                component.layout.scroll_position
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Layout flex children with scroll offset
        let mut current_main = start_pos - scroll_offset;

        for (child_id, child) in flex_children {
            // Apply margins at the start of each item's positioning
            let margin_before = if is_row {
                child.layout.margin.left
            } else {
                child.layout.margin.top
            };

            current_main += margin_before;

            let (main_size, cross_size) = self.calculate_child_sizes(
                child,
                is_row,
                content_space,
                space_per_flex_unit,
                num_flex_items,
                num_auto_sized,
            );

            let cross = self.calculate_cross_position(
                layout.align_items,
                child.layout.align_self,
                is_row,
                content_space,
                cross_size,
                if is_row {
                    (child.layout.margin.top, child.layout.margin.bottom)
                } else {
                    (child.layout.margin.left, child.layout.margin.right)
                },
            );

            let child_bounds = if is_row {
                Bounds {
                    position: ComponentPosition {
                        x: current_main,
                        y: cross,
                    },
                    size: ComponentSize {
                        width: main_size,
                        height: cross_size,
                    },
                }
            } else {
                Bounds {
                    position: ComponentPosition {
                        x: cross,
                        y: current_main,
                    },
                    size: ComponentSize {
                        width: cross_size,
                        height: main_size,
                    },
                }
            };

            self.computed_bounds.insert(child_id, child_bounds);
            self.compute_component_layout(&child_id, Some(child_bounds));

            // Apply margin after the item and move to next position
            let margin_after = if is_row {
                child.layout.margin.right
            } else {
                child.layout.margin.bottom
            };

            current_main += main_size + margin_after + spacing_between;
        }

        // Handle fixed and absolute positioned items last
        for (child_id, child) in positioned_children {
            match child.transform.position_type {
                Position::Fixed(anchor) => {
                    let fixed_bounds = self.calculate_fixed_bounds(child, content_space, anchor);
                    self.computed_bounds.insert(child_id, fixed_bounds);
                    self.compute_component_layout(&child_id, Some(fixed_bounds));
                }
                Position::Absolute(anchor) => {
                    let absolute_bounds = self.calculate_absolute_bounds(child, anchor);
                    self.computed_bounds.insert(child_id, absolute_bounds);
                    self.compute_component_layout(&child_id, Some(absolute_bounds));
                }
                _ => unreachable!(),
            }
        }
    }

    // Add these helper functions to LayoutContext impl
    fn calculate_spacing(&self, spacing_data: SpacingData) -> (f32, f32) {
        let main_axis_size = if spacing_data.is_row {
            spacing_data.content_space.size.width
        } else {
            spacing_data.content_space.size.height
        };

        // Calculate total scaled size including flex items
        let mut total_used_space = spacing_data.total_margins;
        let mut total_flex_size = 0.0;

        // Recalculate total used space considering scaled components
        for (_, child) in spacing_data.flex_children {
            let scale_factor = child.transform.scale_factor;

            if spacing_data.is_row {
                match &child.transform.size.width {
                    FlexValue::Fixed(w) => {
                        total_used_space += w * scale_factor;
                    }
                    FlexValue::Fill => {
                        total_flex_size += spacing_data.space_per_flex_unit
                            * child.layout.flex_grow.max(1.0)
                            * scale_factor;
                    }
                    _ => {}
                }
            } else {
                match &child.transform.size.height {
                    FlexValue::Fixed(h) => {
                        total_used_space += h * scale_factor;
                    }
                    FlexValue::Fill => {
                        total_flex_size += spacing_data.space_per_flex_unit
                            * child.layout.flex_grow.max(1.0)
                            * scale_factor;
                    }
                    _ => {}
                }
            }
        }

        total_used_space += total_flex_size;
        let free_space = (main_axis_size - total_used_space).max(0.0);

        match spacing_data.justify_content {
            JustifyContent::Start => (
                if spacing_data.is_row {
                    spacing_data.content_space.position.x
                } else {
                    spacing_data.content_space.position.y
                },
                0.0,
            ),
            JustifyContent::Center => (
                if spacing_data.is_row {
                    spacing_data.content_space.position.x + free_space / 2.0
                } else {
                    spacing_data.content_space.position.y + free_space / 2.0
                },
                0.0,
            ),
            JustifyContent::End => (
                if spacing_data.is_row {
                    spacing_data.content_space.position.x + free_space
                } else {
                    spacing_data.content_space.position.y + free_space
                },
                0.0,
            ),
            JustifyContent::SpaceBetween => {
                let between = if spacing_data.flex_children.len() > 1 {
                    free_space / (spacing_data.flex_children.len() - 1) as f32
                } else {
                    0.0
                };
                (
                    if spacing_data.is_row {
                        spacing_data.content_space.position.x
                    } else {
                        spacing_data.content_space.position.y
                    },
                    between,
                )
            }
            JustifyContent::SpaceAround => {
                let around = if !spacing_data.flex_children.is_empty() {
                    free_space / spacing_data.flex_children.len() as f32
                } else {
                    0.0
                };
                (
                    if spacing_data.is_row {
                        spacing_data.content_space.position.x + around / 2.0
                    } else {
                        spacing_data.content_space.position.y + around / 2.0
                    },
                    around,
                )
            }
            JustifyContent::SpaceEvenly => {
                let evenly = if spacing_data.flex_children.len() + 1 > 0 {
                    free_space / (spacing_data.flex_children.len() + 1) as f32
                } else {
                    0.0
                };
                (
                    if spacing_data.is_row {
                        spacing_data.content_space.position.x + evenly
                    } else {
                        spacing_data.content_space.position.y + evenly
                    },
                    evenly,
                )
            }
        }
    }

    fn calculate_child_sizes(
        &self,
        child: &Component,
        is_row: bool,
        content_space: Bounds,
        space_per_flex_unit: f32,
        num_flex_items: usize,
        num_auto_sized: usize,
    ) -> (f32, f32) {
        // Calculate available space after accounting for margins
        let main_axis_available = if is_row {
            content_space.size.width - (child.layout.margin.left + child.layout.margin.right)
        } else {
            content_space.size.height - (child.layout.margin.top + child.layout.margin.bottom)
        };

        let cross_axis_available = if is_row {
            content_space.size.height - (child.layout.margin.top + child.layout.margin.bottom)
        } else {
            content_space.size.width - (child.layout.margin.left + child.layout.margin.right)
        };

        let mut main_size = if is_row {
            match &child.transform.size.width {
                FlexValue::Fixed(w) => *w,
                FlexValue::Fill => space_per_flex_unit * child.layout.flex_grow.max(1.0),
                FlexValue::Fraction(frac) => main_axis_available * frac,
                FlexValue::Auto => {
                    if num_flex_items == 0 && num_auto_sized > 0 {
                        space_per_flex_unit
                    } else {
                        main_axis_available
                    }
                }
                _ => child
                    .transform
                    .size
                    .width
                    .resolve(main_axis_available, self.viewport_size.width),
            }
        } else {
            match &child.transform.size.height {
                FlexValue::Fixed(h) => *h,
                FlexValue::Fill => space_per_flex_unit * child.layout.flex_grow.max(1.0),
                FlexValue::Fraction(frac) => main_axis_available * frac,
                FlexValue::Auto => {
                    if num_flex_items == 0 && num_auto_sized > 0 {
                        space_per_flex_unit
                    } else {
                        main_axis_available
                    }
                }
                _ => child
                    .transform
                    .size
                    .height
                    .resolve(main_axis_available, self.viewport_size.height),
            }
        };

        let mut cross_size = if is_row {
            match &child.transform.size.height {
                FlexValue::Fixed(h) => *h,
                FlexValue::Fraction(frac) => cross_axis_available * frac,
                FlexValue::Fill | FlexValue::Auto => cross_axis_available,
                _ => child
                    .transform
                    .size
                    .height
                    .resolve(cross_axis_available, self.viewport_size.height),
            }
        } else {
            match &child.transform.size.width {
                FlexValue::Fixed(w) => *w,
                FlexValue::Fraction(frac) => cross_axis_available * frac,
                FlexValue::Fill | FlexValue::Auto => cross_axis_available,
                _ => child
                    .transform
                    .size
                    .width
                    .resolve(cross_axis_available, self.viewport_size.width),
            }
        };

        // Apply scale factor to sizes if component requires relayout for scaling
        if child.transform.scale_factor != 1.0 {
            main_size *= child.transform.scale_factor;
            cross_size *= child.transform.scale_factor;
        }

        (main_size, cross_size)
    }

    fn calculate_cross_position(
        &self,
        align_items: AlignItems,
        align_self: Option<AlignItems>,
        is_row: bool,
        content_space: Bounds,
        cross_size: f32,
        margins: (f32, f32), // (top/left margin, bottom/right margin) depending on axis
    ) -> f32 {
        // Use align_self if provided, otherwise use parent's align_items
        let alignment = align_self.unwrap_or(align_items);

        let (margin_start, margin_end) = margins;
        let available_cross = if is_row {
            content_space.size.height - margin_start - margin_end
        } else {
            content_space.size.width - margin_start - margin_end
        };

        let content_start = if is_row {
            content_space.position.y + margin_start
        } else {
            content_space.position.x + margin_start
        };

        match alignment {
            AlignItems::Start => content_start,
            AlignItems::Center => content_start + (available_cross - cross_size) / 2.0,
            AlignItems::End => content_start + available_cross - cross_size,
            AlignItems::Stretch => {
                // For Stretch, we've already set the cross_size to fill available space
                content_start
            }
            AlignItems::Baseline => {
                // Simplified baseline implementation - just align to start
                content_start
            }
        }
    }

    pub fn handle_event(
        &mut self,
        event: InputEvent,
    ) -> Option<(Uuid, EventType, Option<AppEvent>)> {
        if let Some(position) = event.position {
            // For hover events, we need to track components that were previously hovered
            // but are no longer under the cursor
            if event.event_type == EventType::Hover {
                // Reset hover state for all components first
                for (_, component) in self.components.iter_mut() {
                    if component.is_hovered() {
                        component.set_hover_state(false);
                    }
                }

                // Find all hoverable components under the cursor and set their state
                for id in self.render_order.iter().rev() {
                    if let Some(component) = self.components.get_mut(id) {
                        if component.is_visible()
                            && component.is_hoverable()
                            && component.is_hit(position)
                        {
                            component.set_hover_state(true);
                        }
                    }
                }
            }

            // Handle scroll events for sliders and scrollable containers
            if matches!(
                event.event_type,
                EventType::ScrollUp | EventType::ScrollDown
            ) {
                // First check for scrollable containers under the cursor
                for id in self.render_order.iter().rev() {
                    if let Some(component) = self.components.get_mut(id) {
                        if component.is_visible()
                            && component.is_hit(position)
                            && component.layout.is_scrollable
                        {
                            let scroll_delta = match event.event_type {
                                EventType::ScrollUp => 30.0,
                                EventType::ScrollDown => -30.0,
                                _ => 0.0,
                            };

                            // If this scroll orientation matches the event, handle it
                            let scroll_applied = match component.layout.scroll_orientation {
                                ScrollOrientation::Vertical => {
                                    if matches!(
                                        component.layout.direction,
                                        FlexDirection::Column | FlexDirection::ColumnReverse
                                    ) {
                                        component.layout.update_scroll_position(scroll_delta)
                                    } else {
                                        false
                                    }
                                }
                                ScrollOrientation::Horizontal => {
                                    if matches!(
                                        component.layout.direction,
                                        FlexDirection::Row | FlexDirection::RowReverse
                                    ) {
                                        component.layout.update_scroll_position(scroll_delta)
                                    } else {
                                        false
                                    }
                                }
                            };

                            // If scroll position was changed, recalculate layout
                            if scroll_applied {
                                // Store needed values from the immutable borrow
                                let container_id = *id;

                                // Now safe to do mutable operations
                                self.compute_layout();

                                // Only update positions for components that are children of the scrolled container
                                // or the container itself
                                let mut affected_ids = vec![container_id];
                                self.get_all_descendants(container_id, &mut affected_ids);

                                for id in affected_ids {
                                    if let (Some(component), Some(bounds)) = (
                                        self.components.get_mut(&id),
                                        self.computed_bounds.get(&id),
                                    ) {
                                        component.set_position_only_layout(bounds);
                                    }
                                }

                                return None; // Consume the scroll event
                            }
                        }
                    }
                }

                // If no scrollable container consumed the event, check for sliders
                let mut slider_id_to_update = None;

                // First pass: identify which slider needs updating (without mutable borrow)
                for id in self.render_order.iter().rev() {
                    if let Some(component) = self.components.get(id) {
                        if component.is_visible()
                            && component.is_hit(position)
                            && component.is_a_slider()
                        {
                            slider_id_to_update = Some(*id);
                            break;
                        }
                    }
                }

                // Second pass: update the identified slider with a clean mutable borrow
                if let Some(id) = slider_id_to_update {
                    // Convert scroll event to delta
                    let scroll_delta = match event.event_type {
                        EventType::ScrollUp => -1.0,  // Flipped
                        EventType::ScrollDown => 1.0, // Flipped
                        _ => 0.0,
                    };

                    // Now we can safely get a mutable reference and update
                    if let Some(slider) = self.components.get_mut(&id) {
                        slider.handle_scroll(scroll_delta);

                        // Make sure this triggers a proper update
                        if let Some(slider_data) = slider.get_slider_data_mut() {
                            slider_data.needs_update = true;
                        }
                    }

                    return None; // Consume the scroll event
                }
            }

            // Process clicking/dragging events
            if matches!(
                event.event_type,
                EventType::Press | EventType::Release | EventType::Drag
            ) {
                // First, check if we're interacting with any component
                let mut hit_component_id = None;

                // Check all components from top to bottom for hit testing
                for id in self.render_order.iter().rev() {
                    if let Some(component) = self.components.get(id) {
                        if component.is_visible() && component.is_hit(position) {
                            hit_component_id = Some(*id);
                            break;
                        }
                    }
                }

                // If we hit a component, handle the event
                if let Some(id) = hit_component_id {
                    // First, check if this component or any parent is a slider
                    if matches!(event.event_type, EventType::Press | EventType::Drag) {
                        self.update_slider_from_cursor(id, position);
                    }

                    // Check if the component itself can handle the event
                    if let Some(component) = self.components.get_mut(&id) {
                        let is_interactive = match &event.event_type {
                            EventType::Drag => component.is_draggable(),
                            EventType::Press => component.is_clickable(),
                            _ => false,
                        };

                        if is_interactive {
                            let return_event = match &event.event_type {
                                EventType::Drag => component.get_drag_event().cloned(),
                                _ => component.get_click_event().cloned(),
                            };
                            return Some((component.id, event.event_type.clone(), return_event));
                        }
                    }

                    // Otherwise bubble up events normally
                    return self.bubble_event_up(id, &event.event_type);
                }
            }
        }
        None
    }

    // Helper method to update slider when clicked/dragged
    fn update_slider_from_cursor(&mut self, start_id: Uuid, position: ComponentPosition) {
        let mut current_id = start_id;

        // Traverse up the component tree looking for sliders
        while let Some(component) = self.components.get(&current_id) {
            if component.is_a_slider() {
                // Found a slider, update its value
                if let Some(slider) = self.components.get_mut(&current_id) {
                    let bounds = slider.computed_bounds;
                    let track_start = bounds.position.x;
                    let track_width = bounds.size.width;

                    if track_width > 0.0 {
                        // Ensure we don't divide by zero
                        // Calculate relative position on the track (0.0 to 1.0)
                        let relative_pos = (position.x - track_start) / track_width;
                        let clamped_pos = relative_pos.clamp(0.0, 1.0);

                        // Get value range from slider data
                        if let Some(slider_data) = slider.get_slider_data() {
                            let range = slider_data.max - slider_data.min;
                            let new_value = slider_data.min + (range * clamped_pos);

                            // Update slider value which will mark it for update
                            slider.set_value(new_value);
                        }
                    }
                }

                break;
            }

            // Move up to parent
            if let Some(parent_id) = component.get_parent_id() {
                current_id = parent_id;
            } else {
                // Reached root component with no slider found
                break;
            }
        }
    }

    // Bubble up through parent hierarchy to find handler
    fn bubble_event_up(
        &mut self,
        start_id: Uuid,
        event_type: &EventType,
    ) -> Option<(Uuid, EventType, Option<AppEvent>)> {
        let mut current_id = start_id;

        while let Some(component) = self.components.get_mut(&current_id) {
            // Check if this component can handle the event
            let is_interactive = match &event_type {
                EventType::Drag => component.is_draggable(),
                EventType::Press => component.is_clickable(),
                EventType::Hover => component.is_hoverable(),
                _ => false,
            };

            // Special consideration for sliders
            let is_slider = component.is_a_slider();

            if is_interactive
                || (is_slider && matches!(event_type, EventType::Drag | EventType::Press))
            {
                let return_event = match &event_type {
                    EventType::Drag => component.get_drag_event().cloned(),
                    EventType::Hover => {
                        component.set_hover_state(true);
                        None
                    }
                    _ => component.get_click_event().cloned(),
                };

                return Some((component.id, event_type.clone(), return_event));
            }

            // Move up to parent
            if let Some(parent_id) = component.get_parent_id() {
                current_id = parent_id;
            } else {
                // Reached root component with no handler
                break;
            }
        }

        None
    }

    pub fn reset_all_hover_states(&mut self) {
        for (_, component) in self.components.iter_mut() {
            if component.is_hovered() {
                component.set_hover_state(false);
            }
        }
    }

    pub fn reset_all_drag_states(&mut self, wgpu_ctx: &mut WgpuCtx) {
        for (_, component) in self.components.iter_mut() {
            // Reset drag state for sliders
            if component.is_a_slider() {
                component.reset_drag_state();
            }

            // Reset hover states as well
            if component.is_hovered() {
                component.set_hover_state(false);
            }

            // Only update rendering if the component actually needs it
            if component.needs_update() {
                if let Some(buffer) = component.get_render_data_buffer() {
                    wgpu_ctx.queue.write_buffer(
                        buffer,
                        0,
                        bytemuck::cast_slice(&[
                            component.get_render_data(component.computed_bounds)
                        ]),
                    );
                }
            }
        }
    }

    // Add this helper method to get all descendants of a container
    fn get_all_descendants(&self, parent_id: Uuid, result: &mut Vec<Uuid>) {
        if let Some(parent) = self.components.get(&parent_id) {
            for (child_id, _) in &parent.children_ids {
                result.push(*child_id);
                self.get_all_descendants(*child_id, result);
            }
        }
    }

    pub fn refresh_all_sliders(&mut self) {
        // First collect all the update information without keeping multiple mutable borrows
        let mut slider_updates = Vec::new();

        for (_id, component) in self.components.iter_mut() {
            if component.is_a_slider() {
                // Update track bounds to make sure we have the latest bounds
                component.update_track_bounds(component.computed_bounds);

                // Get slider data and prepare update info
                if let Some(slider_data) = component.get_slider_data_mut() {
                    // Calculate normalized value for visual positioning
                    let normalized_value =
                        (slider_data.value - slider_data.min) / (slider_data.max - slider_data.min);

                    // Mark for update
                    slider_data.needs_update = true;

                    // Save the IDs and normalized value for the second pass
                    if let Some(track_bounds) = slider_data.track_bounds {
                        slider_updates.push((
                            slider_data.thumb_id,
                            slider_data.track_fill_id,
                            normalized_value,
                            track_bounds,
                        ));
                    }
                }

                // Force a refresh of the slider's visual components
                component.refresh_slider();
            }
        }

        // Now apply the updates in a separate pass to avoid multiple mutable borrows
        for (thumb_id, track_fill_id, normalized_value, _) in slider_updates {
            // Update the thumb position
            if let Some(thumb) = self.components.get_mut(&thumb_id) {
                thumb.transform.offset.x = FlexValue::Fraction(normalized_value);
                thumb.flag_for_update();
            }

            // Update the track fill width
            if let Some(track_fill) = self.components.get_mut(&track_fill_id) {
                track_fill.transform.size.width = FlexValue::Fraction(normalized_value);
                track_fill.flag_for_update();
            }
        }

        // No need to call update_components here as it will be called by the caller
    }
}

// fn calc_scale_anchor_offsets(
//     scale_anchor: Anchor,
//     original_width: f32,
//     original_height: f32,
//     scaled_width: f32,
//     scaled_height: f32,
// ) -> (f32, f32) {
//     match scale_anchor {
//         Anchor::TopLeft => (0.0, 0.0),
//         Anchor::Top => ((original_width - scaled_width) / 2.0, 0.0),
//         Anchor::TopRight => (original_width - scaled_width, 0.0),
//         Anchor::Left => (0.0, (original_height - scaled_height) / 2.0),
//         Anchor::Center => (
//             (original_width - scaled_width) / 2.0,
//             (original_height - scaled_height) / 2.0,
//         ),
//         Anchor::Right => (
//             original_width - scaled_width,
//             (original_height - scaled_height) / 2.0,
//         ),
//         Anchor::BottomLeft => (0.0, original_height - scaled_height),
//         Anchor::Bottom => (
//             (original_width - scaled_width) / 2.0,
//             original_height - scaled_height,
//         ),
//         Anchor::BottomRight => (
//             original_width - scaled_width,
//             original_height - scaled_height,
//         ),
//     }
// }

// Implement From<f32> for FlexValue to allow for convenient conversions
impl From<f32> for FlexValue {
    fn from(value: f32) -> Self {
        FlexValue::Fixed(value)
    }
}

// Implement From<i32> for FlexValue to allow for convenient conversions
impl From<i32> for FlexValue {
    fn from(value: i32) -> Self {
        FlexValue::Fixed(value as f32)
    }
}
