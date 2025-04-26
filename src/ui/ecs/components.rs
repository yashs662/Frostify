use crate::ui::{
    color::Color,
    layout::{
        BorderRadius, ComponentOffset, ComponentPosition, ComponentSize, 
        FlexValue, Layout, Position,
    },
};
use std::any::Any;
use super::{EntityId, EcsComponent};

// Transform Component
#[derive(Debug, Clone)]
pub struct TransformComponent {
    pub size: crate::ui::layout::Size,
    pub offset: ComponentOffset,
    pub position_type: Position,
    pub z_index: i32,
    pub border_radius: BorderRadius,
    pub max_scale_factor: f32,
    pub min_scale_factor: f32,
    pub scale_factor: f32,
    pub scale_anchor: crate::ui::layout::Anchor,
}

// Layout Component
#[derive(Debug, Clone)]
pub struct LayoutComponent {
    pub layout: Layout,
}

// Hierarchy Component
#[derive(Debug, Clone)]
pub struct HierarchyComponent {
    pub parent: Option<EntityId>,
    pub children: Vec<EntityId>,
}

// Visual Component
#[derive(Debug, Clone)]
pub struct VisualComponent {
    pub component_type: crate::ui::component::ComponentType,
    pub border_width: f32,
    pub border_color: Color,
    pub border_position: crate::ui::component::BorderPosition,
    pub shadow_color: Color,
    pub shadow_offset: (f32, f32),
    pub shadow_blur: f32,
    pub shadow_opacity: f32,
    pub is_visible: bool,
}

// Bounds Component
#[derive(Debug, Clone, Copy)]
pub struct BoundsComponent {
    pub computed_bounds: crate::ui::layout::Bounds,
    pub screen_size: ComponentSize,
    pub clip_bounds: Option<(crate::ui::layout::Bounds, bool, bool)>,
    pub clip_self: bool,
}

// Interaction Component
#[derive(Debug, Clone)]
pub struct InteractionComponent {
    pub is_clickable: bool,
    pub is_draggable: bool,
    pub is_hoverable: bool,
    pub is_hovered: bool,
    pub click_event: Option<crate::app::AppEvent>,
    pub drag_event: Option<crate::app::AppEvent>,
}

// Animation Component
#[derive(Debug, Clone)]
pub struct AnimationComponent {
    pub animations: Vec<crate::ui::animation::Animation>,
    pub needs_update: bool,
}

// Identity Component
#[derive(Debug, Clone)]
pub struct IdentityComponent {
    pub id: EntityId,
    pub debug_name: Option<String>,
    pub component_type: crate::ui::component::ComponentType,
}

// Implement EcsComponent for all components
impl EcsComponent for TransformComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl EcsComponent for LayoutComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl EcsComponent for HierarchyComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl EcsComponent for VisualComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl EcsComponent for BoundsComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl EcsComponent for InteractionComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl EcsComponent for AnimationComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl EcsComponent for IdentityComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// Special component types for specific behaviors

// Slider Component
#[derive(Debug, Clone)]
pub struct SliderComponent {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub thumb_id: EntityId,
    pub track_fill_id: EntityId,
    pub track_bounds: Option<crate::ui::layout::Bounds>,
    pub needs_update: bool,
    pub is_dragging: bool,
}

impl EcsComponent for SliderComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

// Render Data Component
#[derive(Debug, Clone)]
pub struct RenderDataComponent {
    pub render_data_buffer: Option<wgpu::Buffer>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub sampler: Option<wgpu::Sampler>,
}

impl EcsComponent for RenderDataComponent {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}