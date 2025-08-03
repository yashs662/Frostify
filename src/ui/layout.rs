use crate::{
    app::AppEvent,
    ui::{
        ecs::{
            ComponentType, EntityId, NamedRef, World,
            components::{
                BoundsComponent, HierarchyComponent, IdentityComponent, ImageComponent,
                InteractionComponent, LayoutComponent, ModalComponent, PreFitSizeComponent,
                RenderDataComponent, TextComponent, TransformComponent, VisualComponent,
            },
            resources::{
                NamedRefsResource, RenderOrderResource, TextRenderingResource, WgpuQueueResource,
            },
            systems::modal::ModalToggleSystem,
        },
        geometry::QuadGeometry,
        z_index_manager::ZIndexManager,
    },
    utils::create_entity_buffer_data,
    wgpu_ctx::WgpuCtx,
};
use frostify_derive::time_function;
use std::collections::{BTreeMap, HashMap};
use tokio::sync::mpsc::UnboundedSender;

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

impl ComponentOffset {
    pub fn new(x: impl Into<FlexValue>, y: impl Into<FlexValue>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl From<Size<u32>> for Size<f32> {
    fn from(val: Size<u32>) -> Self {
        Size {
            width: val.width as f32,
            height: val.height as f32,
        }
    }
}

impl From<Size<f32>> for Size<u32> {
    fn from(val: Size<f32>) -> Self {
        Size {
            width: val.width as u32,
            height: val.height as u32,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ComponentPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Bounds {
    pub position: ComponentPosition,
    pub size: Size<f32>,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ClipBounds {
    pub bounds: Bounds,
    pub clip_x: bool,
    pub clip_y: bool,
    pub border_radius: BorderRadius,
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
    /// Default - follows flex layout rules
    Flex,
    /// Fixed position relative to parent
    Fixed(Anchor),
    /// Absolute position relative to root
    Absolute(Anchor),
    /// Position in a grid (row, column)
    Grid(usize, usize),
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

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct LayoutSize {
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
#[derive(Default)]
pub struct LayoutContext {
    pub world: World,
    pub computed_bounds: BTreeMap<EntityId, Bounds>,
    pub root_component_id: Option<EntityId>,
    pub viewport_size: Size<u32>,
    initialized: bool,
    pub z_index_manager: ZIndexManager,
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
    pub fn all<T: Into<f32>>(radius: T) -> Self {
        let radius = radius.into();
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    pub fn custom<T: Into<f32>>(
        top_left: T,
        top_right: T,
        bottom_left: T,
        bottom_right: T,
    ) -> Self {
        Self {
            top_left: top_left.into(),
            top_right: top_right.into(),
            bottom_left: bottom_left.into(),
            bottom_right: bottom_right.into(),
        }
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

// Implementation for Size
#[allow(dead_code)]
impl LayoutSize {
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

    pub fn all<T: Into<f32>>(value: T) -> Self {
        let value = value.into();
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn horizontal<T: Into<f32>>(value: T) -> Self {
        let value = value.into();
        Self {
            top: 0.0,
            right: value,
            bottom: 0.0,
            left: value,
        }
    }

    pub fn vertical<T: Into<f32>>(value: T) -> Self {
        let value = value.into();
        Self {
            top: value,
            right: 0.0,
            bottom: value,
            left: 0.0,
        }
    }

    pub fn left<T: Into<f32>>(value: T) -> Self {
        let value = value.into();
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: value,
        }
    }

    pub fn right<T: Into<f32>>(value: T) -> Self {
        let value = value.into();
        Self {
            top: 0.0,
            right: value,
            bottom: 0.0,
            left: 0.0,
        }
    }

    pub fn top<T: Into<f32>>(value: T) -> Self {
        Self {
            top: value.into(),
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }

    pub fn bottom<T: Into<f32>>(value: T) -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: value.into(),
            left: 0.0,
        }
    }

    pub fn custom<T: Into<f32>>(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top: top.into(),
            right: right.into(),
            bottom: bottom.into(),
            left: left.into(),
        }
    }
}

// Implementation for FlexValue
impl FlexValue {
    pub fn resolve(&self, available_space: f32, viewport_dimension: u32) -> f32 {
        match self {
            FlexValue::Auto => available_space,
            FlexValue::Fixed(value) => *value,
            FlexValue::Fraction(frac) => available_space * frac,
            FlexValue::Percentage(perc) => available_space * perc,
            FlexValue::Viewport(perc) => viewport_dimension as f32 * perc,
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
    flex_children: &'a [(EntityId, LayoutComponent, TransformComponent)],
    space_per_flex_unit: f32,
    total_margins: f32,
}

// Layout Context implementation
impl LayoutContext {
    pub fn initialize(
        &mut self,
        viewport_size: Size<u32>,
        wgpu_ctx: &mut WgpuCtx,
        event_sender: &UnboundedSender<AppEvent>,
    ) {
        self.viewport_size = viewport_size;
        self.world
            .initialize_resources(&wgpu_ctx.queue, event_sender);
        self.initialized = true;
        log::trace!("Layout context initialized");
    }

    pub fn clear(&mut self) {
        log::trace!("Clearing layout context");
        self.world.reset();
        self.computed_bounds.clear();
        self.z_index_manager.clear();
    }

    #[cfg(test)]
    pub fn get_computed_bounds(&self) -> &BTreeMap<EntityId, Bounds> {
        &self.computed_bounds
    }

    pub fn resize_viewport(&mut self, wgpu_ctx: &mut WgpuCtx) {
        self.viewport_size = wgpu_ctx.get_screen_size();
        self.reset_fit_to_size_components();
        self.compute_layout_and_sync(wgpu_ctx);
    }

    fn reset_fit_to_size_components(&mut self) {
        let mut pre_fit_sizes = Vec::new();
        self.world
            .for_each_component::<PreFitSizeComponent, _>(|id, pre_fit_size| {
                pre_fit_sizes.push((id, pre_fit_size.clone()));
            });

        for (id, pre_fit_size) in pre_fit_sizes {
            let transform_comp = self
                .world
                .components
                .get_component_mut::<TransformComponent>(id)
                .expect("Expected TransformComponent to exist");
            transform_comp.size.width = pre_fit_size.original_width;
            transform_comp.size.height = pre_fit_size.original_height;
        }
    }

    pub fn find_root_component(&mut self) {
        let mut root_component_ids = Vec::new();
        self.world
            .for_each_component::<HierarchyComponent, _>(|id, hierarchy| {
                if hierarchy.parent.is_none() {
                    root_component_ids.push(id);
                }
            });

        if root_component_ids.is_empty() {
            panic!("No root components found in the world");
        } else if root_component_ids.len() > 1 {
            panic!("Multiple root components found in the world {root_component_ids:?}");
        }

        self.root_component_id = Some(root_component_ids[0]);
        self.z_index_manager.set_root_id(root_component_ids[0]);
    }

    pub fn add_child_to_parent(&mut self, parent_id: EntityId, child_id: EntityId) {
        let is_parent_active = self
            .world
            .components
            .get_component::<InteractionComponent>(parent_id)
            .expect("Parent entity must have an InteractionComponent")
            .is_active;

        // Deactivate the child if the parent is not active
        if !is_parent_active {
            self.world
                .components
                .get_component_mut::<InteractionComponent>(child_id)
                .expect("Child entity must have an InteractionComponent")
                .is_active = false;
        }

        // Update the child's parent reference
        let hierarchy = self
            .world
            .components
            .get_component_mut::<HierarchyComponent>(child_id)
            .expect("Child entity must have a HierarchyComponent");
        {
            hierarchy.parent = Some(parent_id);
        }

        // Add the child to the parent's children list
        let hierarchy = self
            .world
            .components
            .get_component_mut::<HierarchyComponent>(parent_id)
            .expect("Parent entity must have a HierarchyComponent");
        {
            hierarchy.children.push(child_id);
        }

        self.z_index_manager
            .register_component(child_id, Some(parent_id));
    }

    #[time_function]
    pub fn compute_layout_and_sync(&mut self, wgpu_ctx: &mut WgpuCtx) {
        if !self.initialized {
            panic!("Attempting to compute layout before initialization");
        }
        if self.root_component_id.is_none() {
            panic!("Expected root component to be set before computing layout");
        }

        // Clear previous computed bounds
        self.computed_bounds.clear();

        // Compute layout for each root component
        let mut requires_relayout = true;
        let max_relayout_attempts = 3;
        let mut relayout_attempts = 0;

        let text_entities: Vec<EntityId> =
            self.world.get_entities_with_component::<TextComponent>();
        let mut text_updated_bounds: HashMap<EntityId, Bounds> = HashMap::new();

        while requires_relayout && relayout_attempts < max_relayout_attempts {
            requires_relayout = false;

            self.compute_component_layout(&self.root_component_id.unwrap(), None);

            let mut entities_with_fit_to_size = Vec::new(); // (id, previously computed bounds)
            let mut entities_requiring_resizing = Vec::new(); // (id, new bounds to update entity with)

            // Sync computed bounds with the world
            self.world
                .for_each_component_mut::<BoundsComponent, _>(|id, bounds_comp| {
                    if let Some(computed_bounds) = self.computed_bounds.get(&id) {
                        bounds_comp.computed_bounds = *computed_bounds;
                        bounds_comp.screen_size = self.viewport_size;

                        if text_entities.contains(&id) {
                            text_updated_bounds.insert(id, bounds_comp.computed_bounds);
                        }
                    }
                    if bounds_comp.fit_to_size {
                        entities_with_fit_to_size.push((id, bounds_comp.computed_bounds));
                    }
                });

            for (entity_id, old_bounds) in entities_with_fit_to_size.iter() {
                // Check if any images need to be fit to size
                if let Some(image_comp) = self
                    .world
                    .components
                    .get_component::<ImageComponent>(*entity_id)
                {
                    if let Some((calc_size, calc_offset)) =
                        image_comp.calculate_fit_to_size(old_bounds)
                    {
                        if calc_size != old_bounds.size {
                            let new_bounds = Bounds {
                                position: old_bounds.position,
                                size: calc_size,
                            };
                            entities_requiring_resizing.push((*entity_id, new_bounds, calc_offset));
                        }
                    }
                }

                // Check if any text components need to be fit to size
                if let Some(text_comp) = self
                    .world
                    .components
                    .get_component::<TextComponent>(*entity_id)
                {
                    if let Some((calc_size, calc_offset)) =
                        text_comp.calculate_fit_to_size(old_bounds)
                    {
                        if calc_size != old_bounds.size {
                            let new_bounds = Bounds {
                                position: old_bounds.position,
                                size: calc_size,
                            };
                            entities_requiring_resizing.push((*entity_id, new_bounds, calc_offset));
                        }
                    }
                }
            }

            // update transform components of entities that need resizing
            for (entity_id, new_bounds, offset) in entities_requiring_resizing {
                let transform_comp = self
                    .world
                    .components
                    .get_component_mut::<TransformComponent>(entity_id)
                    .expect("Expected TransformComponent to exist");

                // Offset is calculated later for Absolute and Fixed positions
                if transform_comp.position_type == Position::Flex {
                    transform_comp.offset = offset;
                }
                transform_comp.size.width = FlexValue::Fixed(new_bounds.size.width);
                transform_comp.size.height = FlexValue::Fixed(new_bounds.size.height);

                requires_relayout = true;
            }

            relayout_attempts += 1;
        }

        if relayout_attempts == max_relayout_attempts {
            log::warn!(
                "Max relayout attempts reached, some components may not be properly laid out."
            );
        }

        // Update render data buffers
        let device_queue = self
            .world
            .resources
            .get_resource::<WgpuQueueResource>()
            .expect("expected WgpuQueueResource to exist")
            .clone();

        for entity in self
            .world
            .get_entities_with_component::<RenderDataComponent>()
        {
            let render_data = create_entity_buffer_data(&self.world.components, entity);
            let render_data_comp = self
                .world
                .components
                .get_component_mut::<RenderDataComponent>(entity)
                .expect("Expected RenderDataComponent to exist");
            // Generate quad geometry from component bounds and render data
            let quad_geometry =
                QuadGeometry::from_component_bounds(&render_data, self.viewport_size.into());

            let (vertex_buffer, index_buffer) = quad_geometry.create_buffers(&wgpu_ctx.device);

            render_data_comp.vertex_buffer = Some(vertex_buffer);
            render_data_comp.index_buffer = Some(index_buffer);

            device_queue.queue.write_buffer(
                &render_data_comp.render_data_buffer,
                0,
                bytemuck::cast_slice(&[render_data]),
            );
        }

        // Update text components with new bounds and handle texture updates
        let text_resource = self
            .world
            .resources
            .get_resource_mut::<TextRenderingResource>()
            .expect("TextRenderingResource should exist");

        for (entity, new_bounds) in text_updated_bounds {
            // Update text component bounds
            {
                let text_comp = self
                    .world
                    .components
                    .get_component_mut::<TextComponent>(entity)
                    .expect("Expected TextComponent to exist");

                text_comp.update_bounds(new_bounds, &mut text_resource.font_system);
                text_comp.update_texture_if_needed(
                    &wgpu_ctx.device,
                    &wgpu_ctx.queue,
                    &mut text_resource.font_system,
                    &mut text_resource.swash_cache,
                );
            }

            // Update render data component
            let texture_view_ptr = if let Some(text_comp) =
                self.world.components.get_component::<TextComponent>(entity)
            {
                if text_comp.bind_group_update_required() {
                    let texture_view = text_comp.get_texture_view().expect(
                        "Expected TextComponent to have a valid texture view after updating texture",
                    );
                    Some(texture_view as *const _)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(texture_view_ptr) = texture_view_ptr {
                if let Some(render_data_comp) = self
                    .world
                    .components
                    .get_component_mut::<RenderDataComponent>(entity)
                {
                    let texture_view = unsafe { &*texture_view_ptr };
                    let bind_group =
                        wgpu_ctx
                            .device
                            .create_bind_group(&wgpu::BindGroupDescriptor {
                                layout: &wgpu_ctx.unified_bind_group_layout,
                                entries: &[
                                    // Component uniform data
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: render_data_comp
                                            .render_data_buffer
                                            .as_entire_binding(),
                                    },
                                    // Text texture view
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::TextureView(texture_view),
                                    },
                                    // Sampler
                                    wgpu::BindGroupEntry {
                                        binding: 2,
                                        resource: wgpu::BindingResource::Sampler(
                                            &render_data_comp.sampler,
                                        ),
                                    },
                                ],
                                label: Some(format!("{entity} Text Bind Group").as_str()),
                            });

                    render_data_comp.bind_group = Some(bind_group);
                }

                // Reset bind group update flag on text component
                if let Some(text_comp) = self
                    .world
                    .components
                    .get_component_mut::<TextComponent>(entity)
                {
                    text_comp.reset_bind_group_update_required();
                }
            }
        }

        // Use the z-index manager to determine render order
        let named_ref_resource = self
            .world
            .resources
            .get_resource::<NamedRefsResource>()
            .expect("expected NamedRefsResource to exist");

        let render_order = self
            .z_index_manager
            .generate_render_order(named_ref_resource);

        let render_order_resource = self
            .world
            .resources
            .get_resource_mut::<RenderOrderResource>()
            .expect("expected RenderOrderResource to exist");
        render_order_resource.render_order = render_order;
    }

    pub fn open_modal(&mut self, modal_named_ref: NamedRef) {
        // Check if the modal is already open/opening/closing
        let modal_entity_id = self
            .world
            .resources
            .get_resource::<NamedRefsResource>()
            .expect("Expected NamedRefsResource to exist")
            .get_entity_id(&modal_named_ref)
            .expect("Expected modal named reference to have an entity ID");

        let modal_component = self
            .world
            .components
            .get_component::<ModalComponent>(modal_entity_id)
            .expect("Expected ModalComponent to exist for modal");

        if modal_component.is_open || modal_component.is_opening || modal_component.is_closing {
            log::warn!(
                "Modal {modal_named_ref} is already open, opening, or closing. Skipping open request."
            );
            return;
        }

        self.z_index_manager
            .modal_manager
            .open_modal(modal_named_ref);

        self.world.run_system(ModalToggleSystem {
            activate: true,
            named_ref: modal_named_ref,
        });
    }

    pub fn close_modal(&mut self, modal_named_ref: NamedRef) {
        // Check if the modal is already closed/closing
        let modal_entity_id = self
            .world
            .resources
            .get_resource::<NamedRefsResource>()
            .expect("Expected NamedRefsResource to exist")
            .get_entity_id(&modal_named_ref)
            .expect("Expected modal named reference to have an entity ID");

        let modal_component = self
            .world
            .components
            .get_component::<ModalComponent>(modal_entity_id)
            .expect("Expected ModalComponent to exist for modal");

        if !modal_component.is_open || modal_component.is_closing {
            log::warn!(
                "Modal {modal_named_ref} is already closed or closing. Skipping close request."
            );
            return;
        }

        self.z_index_manager
            .modal_manager
            .close_modal(modal_named_ref);

        self.world.run_system(ModalToggleSystem {
            activate: false,
            named_ref: modal_named_ref,
        });
    }

    fn compute_component_layout(&mut self, component_id: &EntityId, parent_bounds: Option<Bounds>) {
        let (Some(transform_comp), Some(heirarchy_comp)) = (
            self.world
                .components
                .get_component::<TransformComponent>(*component_id),
            self.world
                .components
                .get_component::<HierarchyComponent>(*component_id),
        ) else {
            panic!("Expected TransformComponent and HierarchyComponent to exist to compute layout");
        };

        // For root components with Position::Absolute, use viewport as available space
        let available_space = match (parent_bounds, transform_comp.position_type) {
            (None, Position::Absolute(_)) => Bounds {
                position: ComponentPosition { x: 0.0, y: 0.0 },
                size: self.viewport_size.into(),
            },
            (Some(bounds), _) => bounds,
            _ => Bounds {
                position: ComponentPosition { x: 0.0, y: 0.0 },
                size: self.viewport_size.into(),
            },
        };

        // Calculate this component's bounds based on layout properties
        let bounds = self.calculate_bounds(
            transform_comp,
            heirarchy_comp,
            available_space,
            transform_comp.position_type,
        );
        self.computed_bounds.insert(*component_id, bounds);

        // Determine clip bounds based on parent's overflow settings
        let clip_bounds = if let Some(parent_id) = &heirarchy_comp.parent {
            if let (Some(parent_layout_comp), Some(parent_visual_comp)) = (
                self.world
                    .components
                    .get_component::<LayoutComponent>(*parent_id),
                self.world
                    .components
                    .get_component::<VisualComponent>(*parent_id),
            ) {
                // Use parent bounds as clip bounds when overflow is hidden
                if parent_layout_comp.layout.overflow_x == Overflow::Hidden
                    || parent_layout_comp.layout.overflow_y == Overflow::Hidden
                {
                    let parent_bounds = self
                        .computed_bounds
                        .get(parent_id)
                        .cloned()
                        .unwrap_or_default();
                    let content_space = Bounds {
                        position: ComponentPosition {
                            x: parent_bounds.position.x
                                + parent_layout_comp.layout.padding.left
                                + parent_layout_comp.layout.border.left,
                            y: parent_bounds.position.y
                                + parent_layout_comp.layout.padding.top
                                + parent_layout_comp.layout.border.top,
                        },
                        size: Size {
                            width: parent_bounds.size.width
                                - (parent_layout_comp.layout.padding.left
                                    + parent_layout_comp.layout.padding.right
                                    + parent_layout_comp.layout.border.left
                                    + parent_layout_comp.layout.border.right),
                            height: parent_bounds.size.height
                                - (parent_layout_comp.layout.padding.top
                                    + parent_layout_comp.layout.padding.bottom
                                    + parent_layout_comp.layout.border.top
                                    + parent_layout_comp.layout.border.bottom),
                        },
                    };

                    // Store clip bounds with parent's overflow setting
                    Some(ClipBounds {
                        bounds: content_space,
                        clip_x: parent_layout_comp.layout.overflow_x == Overflow::Hidden,
                        clip_y: parent_layout_comp.layout.overflow_y == Overflow::Hidden,
                        border_radius: parent_visual_comp.border_radius,
                    })
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
        if let Some(component) = self
            .world
            .components
            .get_component_mut::<BoundsComponent>(*component_id)
        {
            if component.clip_self {
                component.clip_bounds = clip_bounds;
            } else {
                // If this component doesn't clip itself, clear any clip bounds
                component.clip_bounds = None;
            }
        }

        let Some(identity_comp) = self
            .world
            .components
            .get_component::<IdentityComponent>(*component_id)
        else {
            panic!("Expected IdentityComponent to exist to compute layout")
        };

        // If this is a container, layout its children
        if identity_comp.component_type == ComponentType::Container {
            self.layout_children(component_id, bounds);
        }
    }

    fn calculate_bounds(
        &self,
        transform_comp: &TransformComponent,
        heirarchy_comp: &HierarchyComponent,
        available_space: Bounds,
        position_type: Position,
    ) -> Bounds {
        match position_type {
            Position::Flex => self.calculate_flex_bounds(transform_comp, available_space),
            Position::Fixed(anchor) => {
                self.calculate_fixed_bounds(transform_comp, available_space, anchor)
            }
            Position::Absolute(anchor) => self.calculate_absolute_bounds(transform_comp, anchor),
            Position::Grid(_, _) => {
                self.calculate_grid_bounds(transform_comp, heirarchy_comp, available_space)
            }
        }
    }

    fn calculate_flex_bounds(
        &self,
        transform_comp: &TransformComponent,
        available_space: Bounds,
    ) -> Bounds {
        // For flex items, inherit full size from parent when not explicitly set
        let width = match &transform_comp.size.width {
            FlexValue::Fixed(w) => *w,
            FlexValue::Fill | FlexValue::Auto => available_space.size.width,
            _ => available_space.size.width, // Default to parent width
        };
        let scaled_width = width * transform_comp.scale_factor;
        let height = match &transform_comp.size.height {
            FlexValue::Fixed(h) => *h,
            FlexValue::Fill | FlexValue::Auto => available_space.size.height,
            _ => available_space.size.height, // Default to parent height
        };
        let scaled_height = height * transform_comp.scale_factor;
        // Base position (without offset)
        let position = ComponentPosition {
            x: available_space.position.x,
            y: available_space.position.y,
        };

        // Apply offset if needed
        let offset_x = transform_comp
            .offset
            .x
            .resolve(available_space.size.width, self.viewport_size.width);
        let offset_y = transform_comp
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
            size: Size {
                width: scaled_width,
                height: scaled_height,
            },
        }
    }

    fn calculate_fixed_bounds(
        &self,
        transform_comp: &TransformComponent,
        parent_bounds: Bounds,
        anchor: Anchor,
    ) -> Bounds {
        // Resolve component size
        let width = transform_comp
            .size
            .width
            .resolve(parent_bounds.size.width, self.viewport_size.width);
        let scaled_width = width * transform_comp.scale_factor;
        let height = transform_comp
            .size
            .height
            .resolve(parent_bounds.size.height, self.viewport_size.height);
        let scaled_height = height * transform_comp.scale_factor;

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
        let offset_x = transform_comp
            .offset
            .x
            .resolve(parent_bounds.size.width, self.viewport_size.width);
        let offset_y = transform_comp
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
            size: Size {
                width: scaled_width,
                height: scaled_height,
            },
        }
    }

    fn calculate_absolute_bounds(
        &self,
        transform_comp: &TransformComponent,
        anchor: Anchor,
    ) -> Bounds {
        // For absolute positioning, we position relative to the viewport
        let parent_bounds = Bounds {
            position: ComponentPosition { x: 0.0, y: 0.0 },
            size: self.viewport_size.into(),
        };

        self.calculate_fixed_bounds(transform_comp, parent_bounds, anchor)
    }

    fn calculate_grid_bounds(
        &self,
        transform_comp: &TransformComponent,
        heirarchy_comp: &HierarchyComponent,
        available_space: Bounds,
    ) -> Bounds {
        if let Position::Grid(row, col) = transform_comp.position_type {
            if let Some(parent_id) = &heirarchy_comp.parent {
                if let Some(parent) = self
                    .world
                    .components
                    .get_component::<LayoutComponent>(*parent_id)
                {
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
                            size: Size { width, height },
                        };
                    }
                }
            }
        }

        // Fall back to flex layout if grid information is missing
        panic!("Grid layout not found, falling back to flex layout, this should not happen");
    }

    fn layout_children(&mut self, parent_id: &EntityId, parent_bounds: Bounds) {
        let parent_layout_comp = self
            .world
            .components
            .get_component::<LayoutComponent>(*parent_id)
            .cloned()
            .expect("Expected LayoutComponent to exist to compute layout");
        let interaction_comp = self
            .world
            .components
            .get_component::<InteractionComponent>(*parent_id)
            .expect("Expected InteractionComponent to exist to compute layout");
        let heirarchy_comp = self
            .world
            .components
            .get_component::<HierarchyComponent>(*parent_id)
            .expect("Expected HierarchyComponent to exist to compute layout");

        if heirarchy_comp.children.is_empty() || !interaction_comp.is_active {
            return;
        }

        // Calculate available space for children inside the parent's content area
        let content_space = Bounds {
            position: ComponentPosition {
                x: parent_bounds.position.x
                    + parent_layout_comp.layout.padding.left
                    + parent_layout_comp.layout.border.left,
                y: parent_bounds.position.y
                    + parent_layout_comp.layout.padding.top
                    + parent_layout_comp.layout.border.top,
            },
            size: Size {
                width: parent_bounds.size.width
                    - (parent_layout_comp.layout.padding.left
                        + parent_layout_comp.layout.padding.right
                        + parent_layout_comp.layout.border.left
                        + parent_layout_comp.layout.border.right),
                height: parent_bounds.size.height
                    - (parent_layout_comp.layout.padding.top
                        + parent_layout_comp.layout.padding.bottom
                        + parent_layout_comp.layout.border.top
                        + parent_layout_comp.layout.border.bottom),
            },
        };

        let mut child_components = heirarchy_comp
            .children
            .iter()
            .filter_map(|child_id| {
                let layout_comp = self
                    .world
                    .components
                    .get_component::<LayoutComponent>(*child_id);
                let transform_comp = self
                    .world
                    .components
                    .get_component::<TransformComponent>(*child_id);
                if layout_comp.is_none() || transform_comp.is_none() {
                    return None;
                }
                let layout_comp = layout_comp.unwrap();
                let transform_comp = transform_comp.unwrap();
                Some((*child_id, layout_comp.clone(), transform_comp.clone()))
            })
            .collect::<Vec<(EntityId, LayoutComponent, TransformComponent)>>();

        child_components.sort_by_key(|(_, layout_comp, _)| layout_comp.layout.order);

        // Split children into fixed/absolute and flex
        let (positioned_children, flex_children): (Vec<_>, Vec<_>) = child_components
            .into_iter()
            .partition(|(_, _, transform_comp)| {
                matches!(
                    transform_comp.position_type,
                    Position::Fixed(_) | Position::Absolute(_)
                )
            });

        let is_row = matches!(
            parent_layout_comp.layout.direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );

        // First handle flex layout to establish container boundaries
        let mut total_fixed_size = 0.0;
        let mut total_flex_grow = 0.0;
        let mut num_auto_sized = 0;
        let mut num_flex_items = 0;
        let mut num_fraction_items = 0;
        let mut total_fraction = 0.0;
        let mut total_margins = 0.0;
        let mut content_size = 0.0; // To calculate max scroll

        // Only consider flex children for space calculations
        for (_, layout_comp, transform_comp) in &flex_children {
            // Apply scale factor if component requires relayout
            let scale_factor = transform_comp.scale_factor;
            // Sum up margins in the main axis
            if is_row {
                total_margins += layout_comp.layout.margin.left + layout_comp.layout.margin.right;
                match &transform_comp.size.width {
                    FlexValue::Fixed(w) => {
                        let scaled_width = w * scale_factor;
                        total_fixed_size += scaled_width;
                        content_size += scaled_width
                            + layout_comp.layout.margin.left
                            + layout_comp.layout.margin.right;
                    }
                    FlexValue::Fill => {
                        // For Fill items, we need to ensure they all get equal space if flex_grow is 0
                        // by setting a default of 1.0
                        let effective_flex_grow = if layout_comp.layout.flex_grow == 0.0 {
                            1.0
                        } else {
                            layout_comp.layout.flex_grow
                        };
                        total_flex_grow += effective_flex_grow;
                        num_flex_items += 1;
                    }
                    FlexValue::Auto => num_auto_sized += 1,
                    FlexValue::Fraction(frac) => {
                        total_fraction += frac;
                        num_fraction_items += 1;
                    }
                    _ => {}
                }
            } else {
                total_margins += layout_comp.layout.margin.top + layout_comp.layout.margin.bottom;
                match &transform_comp.size.height {
                    FlexValue::Fixed(h) => {
                        let scaled_height = h * scale_factor;
                        total_fixed_size += scaled_height;
                        content_size += scaled_height
                            + layout_comp.layout.margin.top
                            + layout_comp.layout.margin.bottom;
                    }
                    FlexValue::Fill => {
                        // For Fill items, we need to ensure they all get equal space if flex_grow is 0
                        // by setting a default of 1.0
                        let effective_flex_grow = if layout_comp.layout.flex_grow == 0.0 {
                            1.0
                        } else {
                            layout_comp.layout.flex_grow
                        };
                        total_flex_grow += effective_flex_grow;
                        num_flex_items += 1;
                    }
                    FlexValue::Auto => num_auto_sized += 1,
                    FlexValue::Fraction(frac) => {
                        total_fraction += frac;
                        num_fraction_items += 1;
                    }
                    _ => {}
                }
            }
        }

        let main_axis_size = if is_row {
            content_space.size.width
        } else {
            content_space.size.height
        };

        // Calculate space taken up by fractional items
        let total_fraction_space = if total_fraction > 0.0 && num_fraction_items > 0 {
            // Handle the case where total fraction exceeds 1.0 by normalizing
            let effective_fraction = if total_fraction > 1.0 {
                1.0
            } else {
                total_fraction
            };
            main_axis_size * effective_fraction
        } else {
            0.0
        };

        // Subtract margins and fractional space from available space before distributing to flexbox
        let remaining_space =
            (main_axis_size - total_fixed_size - total_margins - total_fraction_space).max(0.0);

        // Calculate how much space each flex unit gets
        let space_per_flex_unit = if total_flex_grow > 0.0 {
            remaining_space / total_flex_grow
        } else if num_auto_sized > 0 {
            remaining_space / num_auto_sized as f32
        } else {
            0.0
        };

        let spacing_data = SpacingData {
            justify_content: parent_layout_comp.layout.justify_content,
            is_row,
            content_space,
            flex_children: &flex_children,
            space_per_flex_unit,
            total_margins,
        };

        // Calculate spacing based on justify_content for flex items only
        let (start_pos, spacing_between) = self.calculate_spacing(spacing_data);

        // Update max scroll value if container is scrollable
        if let Some(component) = self
            .world
            .components
            .get_component_mut::<LayoutComponent>(*parent_id)
        {
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
        let scroll_offset = if let Some(component) = self
            .world
            .components
            .get_component::<LayoutComponent>(*parent_id)
        {
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

        for (child_id, layout_comp, transform_comp) in flex_children {
            // Apply margins at the start of each item's positioning
            let margin_before = if is_row {
                layout_comp.layout.margin.left
            } else {
                layout_comp.layout.margin.top
            };

            current_main += margin_before;

            let (main_size, cross_size) = self.calculate_child_sizes(
                (&layout_comp, &transform_comp),
                is_row,
                content_space,
                space_per_flex_unit,
                num_flex_items,
                num_auto_sized,
            );

            let cross = self.calculate_cross_position(
                parent_layout_comp.layout.align_items,
                layout_comp.layout.align_self,
                is_row,
                content_space,
                cross_size,
                if is_row {
                    (
                        layout_comp.layout.margin.top,
                        layout_comp.layout.margin.bottom,
                    )
                } else {
                    (
                        layout_comp.layout.margin.left,
                        layout_comp.layout.margin.right,
                    )
                },
            );

            let child_bounds = if is_row {
                Bounds {
                    position: ComponentPosition {
                        x: current_main,
                        y: cross,
                    },
                    size: Size {
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
                    size: Size {
                        width: cross_size,
                        height: main_size,
                    },
                }
            };

            self.computed_bounds.insert(child_id, child_bounds);
            self.compute_component_layout(&child_id, Some(child_bounds));

            // Apply margin after the item and move to next position
            let margin_after = if is_row {
                layout_comp.layout.margin.right
            } else {
                layout_comp.layout.margin.bottom
            };

            current_main += main_size + margin_after + spacing_between;
        }

        // Handle fixed and absolute positioned items last
        for (child_id, _, transform_comp) in positioned_children {
            match transform_comp.position_type {
                Position::Fixed(anchor) => {
                    let fixed_bounds =
                        self.calculate_fixed_bounds(&transform_comp, content_space, anchor);
                    self.computed_bounds.insert(child_id, fixed_bounds);
                    self.compute_component_layout(&child_id, Some(fixed_bounds));
                }
                Position::Absolute(anchor) => {
                    let absolute_bounds = self.calculate_absolute_bounds(&transform_comp, anchor);
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
        for (_, layout_comp, transform_comp) in spacing_data.flex_children {
            let scale_factor = transform_comp.scale_factor;

            if spacing_data.is_row {
                match &transform_comp.size.width {
                    FlexValue::Fixed(w) => {
                        total_used_space += w * scale_factor;
                    }
                    FlexValue::Fill => {
                        total_flex_size += spacing_data.space_per_flex_unit
                            * layout_comp.layout.flex_grow.max(1.0)
                            * scale_factor;
                    }
                    _ => {}
                }
            } else {
                match &transform_comp.size.height {
                    FlexValue::Fixed(h) => {
                        total_used_space += h * scale_factor;
                    }
                    FlexValue::Fill => {
                        total_flex_size += spacing_data.space_per_flex_unit
                            * layout_comp.layout.flex_grow.max(1.0)
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
        child_components: (&LayoutComponent, &TransformComponent),
        is_row: bool,
        content_space: Bounds,
        space_per_flex_unit: f32,
        num_flex_items: usize,
        num_auto_sized: usize,
    ) -> (f32, f32) {
        let (child_layout_comp, child_transform_comp) = child_components;
        // Calculate available space after accounting for margins
        let main_axis_available = if is_row {
            content_space.size.width
                - (child_layout_comp.layout.margin.left + child_layout_comp.layout.margin.right)
        } else {
            content_space.size.height
                - (child_layout_comp.layout.margin.top + child_layout_comp.layout.margin.bottom)
        };

        let cross_axis_available = if is_row {
            content_space.size.height
                - (child_layout_comp.layout.margin.top + child_layout_comp.layout.margin.bottom)
        } else {
            content_space.size.width
                - (child_layout_comp.layout.margin.left + child_layout_comp.layout.margin.right)
        };

        let mut main_size = if is_row {
            match &child_transform_comp.size.width {
                FlexValue::Fixed(w) => *w,
                FlexValue::Fill => {
                    // Use same logic as in the calculation of total_flex_grow
                    let effective_flex_grow = if child_layout_comp.layout.flex_grow == 0.0 {
                        1.0
                    } else {
                        child_layout_comp.layout.flex_grow
                    };
                    space_per_flex_unit * effective_flex_grow
                }
                FlexValue::Fraction(frac) => main_axis_available * frac,
                FlexValue::Auto => {
                    if num_flex_items == 0 && num_auto_sized > 0 {
                        space_per_flex_unit
                    } else {
                        main_axis_available
                    }
                }
                _ => child_transform_comp
                    .size
                    .width
                    .resolve(main_axis_available, self.viewport_size.width),
            }
        } else {
            match &child_transform_comp.size.height {
                FlexValue::Fixed(h) => *h,
                FlexValue::Fill => {
                    // Use same logic as in the calculation of total_flex_grow
                    let effective_flex_grow = if child_layout_comp.layout.flex_grow == 0.0 {
                        1.0
                    } else {
                        child_layout_comp.layout.flex_grow
                    };
                    space_per_flex_unit * effective_flex_grow
                }
                FlexValue::Fraction(frac) => main_axis_available * frac,
                FlexValue::Auto => {
                    if num_flex_items == 0 && num_auto_sized > 0 {
                        space_per_flex_unit
                    } else {
                        main_axis_available
                    }
                }
                _ => child_transform_comp
                    .size
                    .height
                    .resolve(main_axis_available, self.viewport_size.height),
            }
        };

        let mut cross_size = if is_row {
            match &child_transform_comp.size.height {
                FlexValue::Fixed(h) => *h,
                FlexValue::Fraction(frac) => cross_axis_available * frac,
                FlexValue::Fill | FlexValue::Auto => cross_axis_available,
                _ => child_transform_comp
                    .size
                    .height
                    .resolve(cross_axis_available, self.viewport_size.height),
            }
        } else {
            match &child_transform_comp.size.width {
                FlexValue::Fixed(w) => *w,
                FlexValue::Fraction(frac) => cross_axis_available * frac,
                FlexValue::Fill | FlexValue::Auto => cross_axis_available,
                _ => child_transform_comp
                    .size
                    .width
                    .resolve(cross_axis_available, self.viewport_size.width),
            }
        };

        // Apply scale factor to sizes if component requires relayout for scaling
        if child_transform_comp.scale_factor != 1.0 {
            main_size *= child_transform_comp.scale_factor;
            cross_size *= child_transform_comp.scale_factor;
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
}

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
