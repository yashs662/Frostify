use crate::{
    ui::component::{Component, ComponentType},
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use log::{debug, error, trace};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct ComponentOffset {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ComponentSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Default)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

// JustifyContent enum
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

// FlexWrap enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

// Enhanced FlexValue enum
#[derive(Debug, Clone, PartialEq)]
pub enum FlexValue {
    Auto,
    Fixed(f32),
    Fraction(f32),                       // Similar to flex-grow in CSS
    Percentage(f32),                     // 0.0 to 1.0
    Viewport(f32),                       // Percentage of viewport
    Min(Box<FlexValue>, Box<FlexValue>), // min(a, b)
    Max(Box<FlexValue>, Box<FlexValue>), // max(a, b)
    Fit,                                 // fit-content
    Fill,                                // 100%
}

// FlexItem struct for layout calculation
#[derive(Debug, Clone)]
struct FlexItem {
    id: Uuid,
    bounds: Bounds,
    margin: Edges,
    flex_grow: f32,
    flex_shrink: f32,
    align_self: Option<AlignItems>,
}

// Edges struct for padding, margin, and border
#[derive(Debug, Clone, Copy)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

// Position enum for positioning system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Flex,               // Default - follows flex layout rules
    Fixed(Anchor),      // Fixed position relative to parent
    Absolute(Anchor),   // Absolute position relative to root
    Grid(usize, usize), // Position in a grid (row, column)
}

// Anchor enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
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
}

// Size struct to replace ComponentSize
#[derive(Debug, Clone)]
pub struct Size {
    pub width: FlexValue,
    pub height: FlexValue,
    pub min_width: FlexValue,
    pub min_height: FlexValue,
    pub max_width: FlexValue,
    pub max_height: FlexValue,
}

// Enhanced Layout struct
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
}

// LayoutContext to manage component relationships and computed layouts
#[derive(Debug, Default)]
pub struct LayoutContext {
    components: HashMap<Uuid, Component>,
    computed_bounds: HashMap<Uuid, Bounds>,
    viewport_size: ComponentSize,
    render_order: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct GridLayout {
    pub columns: Vec<FlexValue>, // Width of each column
    pub rows: Vec<FlexValue>,    // Height of each row
    pub column_gap: f32,         // Gap between columns
    pub row_gap: f32,            // Gap between rows
}

// Event types
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    Click,
    Hover,
    Press,
    Release,
    DragStart,
    DragMove,
    DragEnd,
    Focus,
    Blur,
    TextInput,
    KeyPress,
    KeyRelease,
    Scroll,
    Resize,
}

// Event data
#[derive(Debug, Clone)]
pub struct InputEvent {
    pub event_type: EventType,
    pub position: Option<ComponentPosition>,
    pub button: Option<usize>,
    pub key: Option<String>,
    pub text: Option<String>,
}

// Implementation for Size
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

    pub fn with_padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_margin(mut self, margin: Edges) -> Self {
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
}

// Implementation for Edges
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

// Layout Context implementation
impl LayoutContext {
    pub fn draw(&self, render_pass: &mut wgpu::RenderPass, app_pipelines: &mut AppPipelines) {
        for id in &self.render_order {
            trace!("Drawing component id: {}", id);
            if let Some(component) = self.get_component(id) {
                if component.component_type == ComponentType::Label {
                    // Text rendering is done in a separate pass
                    continue;
                }

                if let Some(render_bounds) = self.computed_bounds.get(id) {
                    component.draw(render_pass, app_pipelines);
                } else {
                    // check if component has a parent if so get the parent's bounds
                    if let Some(parent_id) = &component.parent {
                        if let Some(parent_bounds) = self.computed_bounds.get(parent_id) {
                            component.draw(render_pass, app_pipelines);
                        } else {
                            error!(
                                "Parent component with id: {} not found for rendering, render order is corrupt",
                                parent_id
                            );
                        }
                    } else {
                        error!(
                            "Component with id: {} not found for rendering, render order is corrupt",
                            id
                        );
                    }
                }
            } else {
                error!(
                    "Component with id: {} not found for rendering, render order is corrupt",
                    id
                );
            }
        }
    }

    pub fn add_component(&mut self, component: Component) {
        self.components.insert(component.id, component);
    }

    pub fn get_component(&self, id: &Uuid) -> Option<&Component> {
        self.components.get(id)
    }

    pub fn resize_viewport(&mut self, width: f32, height: f32, wgpu_ctx: &mut WgpuCtx) {
        self.viewport_size.width = width;
        self.viewport_size.height = height;
        // Recompute all layouts
        self.compute_layout();

        let screen_size = ComponentSize { width, height };

        // for every component in computed_bounds, update the position
        for (id, bounds) in self.computed_bounds.iter() {
            let mut children = Vec::new();
            if let Some(component) = self.components.get_mut(id) {
                component.set_position(wgpu_ctx, *bounds, screen_size);

                // set_position for children
                children.extend(component.children.clone());
            }

            while let Some(child_id) = children.pop() {
                if let Some(child) = self.components.get_mut(&child_id) {
                    if let Some(parent_bounds) = self.computed_bounds.get(&child.parent.unwrap()) {
                        child.set_position(wgpu_ctx, *parent_bounds, screen_size);
                    }
                    children.extend(child.children.clone());
                }
            }
        }
    }

    pub fn compute_layout(&mut self) {
        // Find root components (those without parents)
        let root_components: Vec<Uuid> = self
            .components
            .values()
            .filter(|c| c.parent.is_none())
            .map(|c| c.id)
            .collect();

        // Compute layout for each root component
        for root_id in root_components {
            self.compute_component_layout(&root_id, None);
        }

        // compute render_order
        let mut render_order = Vec::new();
        for component in self.components.values() {
            render_order.push((component.transform.z_index, component.id));
        }

        // Sort by z-index lowest to highest
        render_order.sort_by(|a, b| a.0.cmp(&b.0));
        self.render_order = render_order.iter().map(|(_, id)| *id).collect();
    }

    fn compute_component_layout(&mut self, component_id: &Uuid, parent_bounds: Option<Bounds>) {
        let component = self.get_component(component_id).unwrap().clone();

        // Determine the available space based on parent bounds or viewport
        let available_space = parent_bounds.unwrap_or(Bounds {
            position: ComponentPosition { x: 0.0, y: 0.0 },
            size: self.viewport_size,
        });

        // Calculate this component's bounds based on layout properties
        let bounds = self.calculate_bounds(&component, available_space);
        self.computed_bounds.insert(*component_id, bounds);

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
        // Get available width and height, accounting for parent's padding/margin/border
        let available_width = available_space.size.width;
        let available_height = available_space.size.height;

        // Resolve component size based on FlexValues
        let width = component
            .transform
            .size
            .width
            .resolve(available_width, self.viewport_size.width);

        let height = component
            .transform
            .size
            .height
            .resolve(available_height, self.viewport_size.height);

        // Apply min/max constraints
        let min_width = component
            .transform
            .size
            .min_width
            .resolve(available_width, self.viewport_size.width);

        let min_height = component
            .transform
            .size
            .min_height
            .resolve(available_height, self.viewport_size.height);

        let max_width = component
            .transform
            .size
            .max_width
            .resolve(available_width, self.viewport_size.width);

        let max_height = component
            .transform
            .size
            .max_height
            .resolve(available_height, self.viewport_size.height);

        // Apply constraints
        let constrained_width = if max_width > 0.0 {
            f32::min(width, max_width)
        } else {
            width
        };

        let constrained_width = f32::max(constrained_width, min_width);

        let constrained_height = if max_height > 0.0 {
            f32::min(height, max_height)
        } else {
            height
        };

        let constrained_height = f32::max(constrained_height, min_height);

        // Default position (will be adjusted by parent's layout_children)
        let position = available_space.position;

        Bounds {
            position,
            size: ComponentSize {
                width: constrained_width,
                height: constrained_height,
            },
        }
    }

    fn calculate_fixed_bounds(
        &self,
        component: &Component,
        parent_bounds: Bounds,
        anchor: Anchor,
    ) -> Bounds {
        let offset = component.transform.offset;

        // Resolve component size
        let width = component
            .transform
            .size
            .width
            .resolve(parent_bounds.size.width, self.viewport_size.width);

        let height = component
            .transform
            .size
            .height
            .resolve(parent_bounds.size.height, self.viewport_size.height);

        // Calculate position based on anchor
        let position = match anchor {
            Anchor::TopLeft => ComponentPosition {
                x: parent_bounds.position.x + offset.x,
                y: parent_bounds.position.y + offset.y,
            },
            Anchor::Top => ComponentPosition {
                x: parent_bounds.position.x + (parent_bounds.size.width - width) / 2.0 + offset.x,
                y: parent_bounds.position.y + offset.y,
            },
            Anchor::TopRight => ComponentPosition {
                x: parent_bounds.position.x + parent_bounds.size.width - width - offset.x,
                y: parent_bounds.position.y + offset.y,
            },
            Anchor::Left => ComponentPosition {
                x: parent_bounds.position.x + offset.x,
                y: parent_bounds.position.y + (parent_bounds.size.height - height) / 2.0 + offset.y,
            },
            Anchor::Center => ComponentPosition {
                x: parent_bounds.position.x + (parent_bounds.size.width - width) / 2.0 + offset.x,
                y: parent_bounds.position.y + (parent_bounds.size.height - height) / 2.0 + offset.y,
            },
            Anchor::Right => ComponentPosition {
                x: parent_bounds.position.x + parent_bounds.size.width - width - offset.x,
                y: parent_bounds.position.y + (parent_bounds.size.height - height) / 2.0 + offset.y,
            },
            Anchor::BottomLeft => ComponentPosition {
                x: parent_bounds.position.x + offset.x,
                y: parent_bounds.position.y + parent_bounds.size.height - height - offset.y,
            },
            Anchor::Bottom => ComponentPosition {
                x: parent_bounds.position.x + (parent_bounds.size.width - width) / 2.0 + offset.x,
                y: parent_bounds.position.y + parent_bounds.size.height - height - offset.y,
            },
            Anchor::BottomRight => ComponentPosition {
                x: parent_bounds.position.x + parent_bounds.size.width - width - offset.x,
                y: parent_bounds.position.y + parent_bounds.size.height - height - offset.y,
            },
        };

        Bounds {
            position,
            size: ComponentSize { width, height },
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
            if let Some(parent_id) = &component.parent {
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

        // If no children or component not visible, skip layout
        if parent.children.is_empty() || !layout.visible {
            return;
        }

        // Calculate available space for children
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

        // Sort children by order property (CSS order)
        let mut sorted_children: Vec<(Uuid, &Component)> = parent
            .children
            .iter()
            .filter_map(|id| self.components.get(id).map(|component| (*id, component)))
            .collect();

        sorted_children.sort_by_key(|(_, component)| component.layout.order);

        // First pass: Measure all children to determine total flex and sizes
        let mut flex_items: Vec<FlexItem> = Vec::new();
        let mut total_main_size = 0.0;
        let mut total_flex_grow = 0.0;
        let mut total_flex_shrink = 0.0;

        let is_horizontal = match layout.direction {
            FlexDirection::Row | FlexDirection::RowReverse => true,
            FlexDirection::Column | FlexDirection::ColumnReverse => false,
        };

        // First pass: Calculate child sizes
        for (child_id, child) in &sorted_children {
            // Skip invisible children
            if !child.layout.visible {
                continue;
            }

            // Get measure size for the child
            let child_bounds = self.calculate_bounds(child, content_space);

            // Create a flex item for layout calculations
            let flex_item = FlexItem {
                id: *child_id,
                bounds: child_bounds,
                margin: child.layout.margin,
                flex_grow: child.layout.flex_grow,
                flex_shrink: child.layout.flex_shrink,
                align_self: child.layout.align_self,
            };

            // Add to total sizes
            let main_size = if is_horizontal {
                flex_item.bounds.size.width + child.layout.margin.left + child.layout.margin.right
            } else {
                flex_item.bounds.size.height + child.layout.margin.top + child.layout.margin.bottom
            };

            total_main_size += main_size;
            total_flex_grow += child.layout.flex_grow;
            total_flex_shrink += child.layout.flex_shrink;

            flex_items.push(flex_item);
        }

        // Second pass: Apply flex layout
        let main_axis_size = if is_horizontal {
            content_space.size.width
        } else {
            content_space.size.height
        };

        let free_space = main_axis_size - total_main_size;

        // Distribute available space based on flex-grow/shrink
        if free_space > 0.0 && total_flex_grow > 0.0 {
            // Positive free space - distribute according to flex-grow
            for flex_item in &mut flex_items {
                if flex_item.flex_grow > 0.0 {
                    let grow_amount = free_space * (flex_item.flex_grow / total_flex_grow);

                    if is_horizontal {
                        flex_item.bounds.size.width += grow_amount;
                    } else {
                        flex_item.bounds.size.height += grow_amount;
                    }
                }
            }
        } else if free_space < 0.0 && total_flex_shrink > 0.0 {
            // Negative free space - distribute according to flex-shrink
            for flex_item in &mut flex_items {
                if flex_item.flex_shrink > 0.0 {
                    let shrink_amount = (-free_space) * (flex_item.flex_shrink / total_flex_shrink);

                    if is_horizontal {
                        flex_item.bounds.size.width =
                            f32::max(0.0, flex_item.bounds.size.width - shrink_amount);
                    } else {
                        flex_item.bounds.size.height =
                            f32::max(0.0, flex_item.bounds.size.height - shrink_amount);
                    }
                }
            }
        }

        // Third pass: Position items according to justification and alignment
        let mut main_pos = if is_horizontal {
            content_space.position.x
        } else {
            content_space.position.y
        };

        // Apply justification
        match layout.justify_content {
            JustifyContent::Start => {
                // Start alignment is the default
            }
            JustifyContent::Center => {
                // Center items along the main axis
                main_pos += free_space / 2.0;
            }
            JustifyContent::End => {
                // End alignment
                main_pos += free_space;
            }
            JustifyContent::SpaceBetween => {
                // Space items evenly with no space at the ends
                if flex_items.len() > 1 {
                    let space_between = free_space / (flex_items.len() as f32 - 1.0);

                    for (i, flex_item) in flex_items.iter_mut().enumerate() {
                        if is_horizontal {
                            flex_item.bounds.position.x = main_pos + (i as f32 * space_between);
                        } else {
                            flex_item.bounds.position.y = main_pos + (i as f32 * space_between);
                        }
                    }
                }
            }
            JustifyContent::SpaceAround => {
                // Space items evenly with half-size spaces at the ends
                if !flex_items.is_empty() {
                    let space_around = free_space / flex_items.len() as f32;
                    main_pos += space_around / 2.0;

                    for (i, flex_item) in flex_items.iter_mut().enumerate() {
                        if is_horizontal {
                            flex_item.bounds.position.x = main_pos + (i as f32 * space_around);
                        } else {
                            flex_item.bounds.position.y = main_pos + (i as f32 * space_around);
                        }
                    }
                }
            }
            JustifyContent::SpaceEvenly => {
                // Space items evenly with equal spaces including at the ends
                if !flex_items.is_empty() {
                    let space_evenly = free_space / (flex_items.len() as f32 + 1.0);
                    main_pos += space_evenly;

                    for (i, flex_item) in flex_items.iter_mut().enumerate() {
                        if is_horizontal {
                            flex_item.bounds.position.x = main_pos + (i as f32 * space_evenly);
                        } else {
                            flex_item.bounds.position.y = main_pos + (i as f32 * space_evenly);
                        }
                    }
                }
            }
        }

        // Position items along the main axis (for Start justification)
        if matches!(layout.justify_content, JustifyContent::Start) {
            for flex_item in &mut flex_items {
                if is_horizontal {
                    flex_item.bounds.position.x = main_pos + flex_item.margin.left;
                    main_pos += flex_item.bounds.size.width
                        + flex_item.margin.left
                        + flex_item.margin.right;
                } else {
                    flex_item.bounds.position.y = main_pos + flex_item.margin.top;
                    main_pos += flex_item.bounds.size.height
                        + flex_item.margin.top
                        + flex_item.margin.bottom;
                }
            }
        }

        // Position items along the cross axis
        for flex_item in &mut flex_items {
            let align = flex_item.align_self.unwrap_or(layout.align_items);

            let cross_size = if is_horizontal {
                content_space.size.height
            } else {
                content_space.size.width
            };

            let item_cross_size = if is_horizontal {
                flex_item.bounds.size.height
            } else {
                flex_item.bounds.size.width
            };

            match align {
                AlignItems::Start => {
                    // Start alignment is the default
                    if is_horizontal {
                        flex_item.bounds.position.y =
                            content_space.position.y + flex_item.margin.top;
                    } else {
                        flex_item.bounds.position.x =
                            content_space.position.x + flex_item.margin.left;
                    }
                }
                AlignItems::Center => {
                    if is_horizontal {
                        flex_item.bounds.position.y = content_space.position.y
                            + (cross_size
                                - item_cross_size
                                - flex_item.margin.top
                                - flex_item.margin.bottom)
                                / 2.0
                            + flex_item.margin.top;
                    } else {
                        flex_item.bounds.position.x = content_space.position.x
                            + (cross_size
                                - item_cross_size
                                - flex_item.margin.left
                                - flex_item.margin.right)
                                / 2.0
                            + flex_item.margin.left;
                    }
                }
                AlignItems::End => {
                    if is_horizontal {
                        flex_item.bounds.position.y = content_space.position.y + cross_size
                            - item_cross_size
                            - flex_item.margin.bottom;
                    } else {
                        flex_item.bounds.position.x = content_space.position.x + cross_size
                            - item_cross_size
                            - flex_item.margin.right;
                    }
                }
                AlignItems::Stretch => {
                    // Stretch to fill the container
                    if is_horizontal {
                        flex_item.bounds.position.y =
                            content_space.position.y + flex_item.margin.top;
                        flex_item.bounds.size.height =
                            cross_size - flex_item.margin.top - flex_item.margin.bottom;
                    } else {
                        flex_item.bounds.position.x =
                            content_space.position.x + flex_item.margin.left;
                        flex_item.bounds.size.width =
                            cross_size - flex_item.margin.left - flex_item.margin.right;
                    }
                }
                AlignItems::Baseline => {
                    // For simplicity, treat baseline as start for now
                    // Real baseline alignment would need to know the baselines of text elements
                    if is_horizontal {
                        flex_item.bounds.position.y =
                            content_space.position.y + flex_item.margin.top;
                    } else {
                        flex_item.bounds.position.x =
                            content_space.position.x + flex_item.margin.left;
                    }
                }
            }
        }

        // Reverse item positions if using reverse direction
        if matches!(
            layout.direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        ) {
            let main_end = if is_horizontal {
                content_space.position.x + content_space.size.width
            } else {
                content_space.position.y + content_space.size.height
            };

            for flex_item in &mut flex_items {
                if is_horizontal {
                    flex_item.bounds.position.x =
                        main_end - flex_item.bounds.position.x - flex_item.bounds.size.width;
                } else {
                    flex_item.bounds.position.y =
                        main_end - flex_item.bounds.position.y - flex_item.bounds.size.height;
                }
            }
        }

        // Layout children
        for flex_item in flex_items {
            self.compute_component_layout(&flex_item.id, Some(flex_item.bounds));
        }
    }

    pub fn handle_event(&mut self, event: InputEvent) -> Vec<(Uuid, EventType)> {
        let mut components_affected = Vec::new();

        match event.event_type {
            EventType::Click | EventType::Press | EventType::Release | EventType::Hover => {
                if let Some(position) = event.position {
                    // Find components at this position (from top to bottom)
                    let mut hit_components: Vec<(Uuid, i32)> = Vec::new();

                    for (id, bounds) in &self.computed_bounds {
                        if position.x >= bounds.position.x
                            && position.x <= bounds.position.x + bounds.size.width
                            && position.y >= bounds.position.y
                            && position.y <= bounds.position.y + bounds.size.height
                        {
                            if let Some(component) = self.get_component(id) {
                                hit_components.push((*id, component.transform.z_index));
                            }
                        }
                    }

                    // Sort by z-index (highest first)
                    hit_components.sort_by(|a, b| b.1.cmp(&a.1));

                    // Add affected components to result
                    for (id, _) in hit_components {
                        components_affected.push((id, event.event_type.clone()));
                    }
                }
            }
            _ => {
                // Handle other event types
            }
        }

        components_affected
    }
}
