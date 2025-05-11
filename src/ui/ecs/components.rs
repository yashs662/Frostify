use frostify_derive::EcsComponent;

use super::{EcsComponent, EntityId, GradientColorStop, GradientType, builders::image::ScaleMode};
use crate::ui::{
    color::Color,
    layout::{BorderRadius, ClipBounds, ComponentOffset, Layout, Position, Size},
};

#[derive(Debug, Clone, EcsComponent)]
pub struct TransformComponent {
    pub size: crate::ui::layout::LayoutSize,
    pub offset: ComponentOffset,
    pub position_type: Position,
    pub z_index: i32,
    pub max_scale_factor: f32,
    pub min_scale_factor: f32,
    pub scale_factor: f32,
    pub scale_anchor: crate::ui::layout::Anchor,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct LayoutComponent {
    pub layout: Layout,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct HierarchyComponent {
    pub parent: Option<EntityId>,
    pub children: Vec<EntityId>,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct VisualComponent {
    pub border_width: f32,
    pub border_color: Color,
    pub border_position: crate::ui::ecs::BorderPosition,
    pub border_radius: BorderRadius,
    pub shadow_color: Color,
    pub shadow_offset: (f32, f32),
    pub shadow_blur: f32,
    pub shadow_opacity: f32,
    pub is_visible: bool,
}

#[derive(Debug, Clone, Copy, EcsComponent)]
pub struct BoundsComponent {
    pub computed_bounds: crate::ui::layout::Bounds,
    pub screen_size: Size,
    pub clip_bounds: Option<ClipBounds>,
    pub clip_self: bool,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct InteractionComponent {
    pub is_clickable: bool,
    pub is_clicked: bool,
    pub is_draggable: bool,
    pub is_hoverable: bool,
    pub is_hovered: bool,
    pub click_event: Option<crate::app::AppEvent>,
    pub drag_event: Option<crate::app::AppEvent>,
}

// Animation Component
#[derive(Debug, Clone, EcsComponent)]
pub struct AnimationComponent {
    pub animations: Vec<crate::ui::animation::Animation>,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct IdentityComponent {
    pub debug_name: String,
    pub component_type: crate::ui::ecs::ComponentType,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct RenderDataComponent {
    pub render_data_buffer: Option<wgpu::Buffer>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub sampler: Option<wgpu::Sampler>,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct ColorComponent {
    pub color: Color,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct GradientComponent {
    pub color_stops: Vec<GradientColorStop>,
    pub gradient_type: GradientType,
    pub angle: f32,
    pub center: Option<(f32, f32)>,
    pub radius: Option<f32>,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct FrostedGlassComponent {
    pub tint_color: Color,
    pub blur_radius: f32,
    pub opacity: f32,
    pub tint_intensity: f32,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct TextComponent {
    pub text: String,
    pub font_size: f32,
    pub line_height_multiplier: f32,
    pub color: Color,
    pub fit_to_size: bool,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct ImageComponent {
    pub image_path: String,
    pub scale_mode: ScaleMode,
    pub original_width: u32,
    pub original_height: u32,
    pub fit_to_size: bool,
}
