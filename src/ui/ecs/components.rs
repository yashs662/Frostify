use crate::{
    app::AppEvent,
    ui::{
        animation::Animation,
        color::Color,
        ecs::ComponentType,
        ecs::{
            BorderPosition, EcsComponent, EntityId, GradientColorStop, GradientType,
            builders::image::ScaleMode,
        },
        layout::{
            BorderRadius, Bounds, ClipBounds, ComponentOffset, FlexValue, Layout,
            LayoutSize, Position, Size,
        },
    },
};
use frostify_derive::EcsComponent;

#[derive(Debug, Clone, EcsComponent)]
pub struct TransformComponent {
    pub size: LayoutSize,
    pub offset: ComponentOffset,
    pub position_type: Position,
    pub scale_factor: f32,
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
    pub border_position: BorderPosition,
    pub border_radius: BorderRadius,
    pub shadow_color: Color,
    pub shadow_offset: (f32, f32),
    pub shadow_blur: f32,
    pub shadow_opacity: f32,
    pub opacity: f32,
}

impl VisualComponent {
    pub fn is_visible(&self) -> bool {
        self.opacity > 0.0
    }
}

#[derive(Debug, Clone, Copy, EcsComponent)]
pub struct BoundsComponent {
    pub computed_bounds: Bounds,
    pub screen_size: Size,
    pub clip_bounds: Option<ClipBounds>,
    pub clip_self: bool,
    pub fit_to_size: bool,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct InteractionComponent {
    pub is_clickable: bool,
    pub is_draggable: bool,
    pub is_hoverable: bool,
    pub is_hovered: bool,
    pub click_event: Option<AppEvent>,
    pub drag_event: Option<AppEvent>,
    pub is_active: bool,
}

// Animation Component
#[derive(Debug, Clone, EcsComponent)]
pub struct AnimationComponent {
    pub animations: Vec<Animation>,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct IdentityComponent {
    pub debug_name: String,
    pub component_type: ComponentType,
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
    pub tint_intensity: f32,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct TextComponent {
    pub text: String,
    pub font_size: f32,
    pub line_height_multiplier: f32,
    pub color: Color,
}

/// Useful in properly re-fitting the component when screen size changes
#[derive(Debug, Clone, EcsComponent)]
pub struct PreFitSizeComponent {
    pub original_width: FlexValue,
    pub original_height: FlexValue,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct ImageComponent {
    pub scale_mode: ScaleMode,
    pub original_width: u32,
    pub original_height: u32,
}

impl ImageComponent {
    pub fn calculate_fit_to_size(&self, old_bounds: &Bounds) -> Option<(Size, ComponentOffset)> {
        // Here old_bounds is analogous to the container bounds as during layout it fills the parent
        let original_width = self.original_width as f32;
        let original_height = self.original_height as f32;
        let original_aspect = original_width / original_height;
        let container_width = old_bounds.size.width;
        let container_height = old_bounds.size.height;
        let container_aspect = container_width / container_height;

        match self.scale_mode {
            ScaleMode::Stretch => {
                // STRETCH - default, use container dimensions directly
                None
            }
            ScaleMode::Contain => {
                // CONTAIN - scale to fit while preserving aspect ratio
                if original_aspect > container_aspect {
                    // Image is wider than container (relative to height)
                    let new_height = container_width / original_aspect;
                    let y_offset = (container_height - new_height) / 2.0;
                    Some((
                        Size {
                            width: container_width,
                            height: new_height,
                        },
                        ComponentOffset {
                            x: 0.0.into(),
                            y: y_offset.into(),
                        },
                    ))
                } else {
                    // Image is taller than container (relative to width)
                    let new_width = container_height * original_aspect;
                    let x_offset = (container_width - new_width) / 2.0;
                    Some((
                        Size {
                            width: new_width,
                            height: container_height,
                        },
                        ComponentOffset {
                            x: x_offset.into(),
                            y: 0.0.into(),
                        },
                    ))
                }
            }
            ScaleMode::ContainNoCenter => {
                // ContainNoCenter - scale to fit while preserving aspect ratio but not offsetting to center
                if original_aspect > container_aspect {
                    // Image is wider than container (relative to height)
                    let new_height = container_width / original_aspect;
                    Some((
                        Size {
                            width: container_width,
                            height: new_height,
                        },
                        ComponentOffset {
                            x: 0.0.into(),
                            y: 0.0.into(),
                        },
                    ))
                } else {
                    // Image is taller than container (relative to width)
                    let new_width = container_height * original_aspect;
                    Some((
                        Size {
                            width: new_width,
                            height: container_height,
                        },
                        ComponentOffset {
                            x: 0.0.into(),
                            y: 0.0.into(),
                        },
                    ))
                }
            }
            ScaleMode::Cover => {
                // COVER - scale to fill while preserving aspect ratio
                // but keep original container bounds to ensure clipping

                // Calculate scaled dimensions that fully cover the container
                let (scaled_width, scaled_height): (f32, f32);
                let (x_offset, y_offset): (f32, f32);

                if original_aspect < container_aspect {
                    // Image is taller than container (relative to width)
                    scaled_width = container_width;
                    scaled_height = container_width / original_aspect;
                    x_offset = 0.0; // No horizontal offset
                    y_offset = (container_height - scaled_height) / 2.0;
                } else {
                    // Image is wider than container (relative to height)
                    scaled_width = container_height * original_aspect;
                    scaled_height = container_height;
                    x_offset = (container_width - scaled_width) / 2.0;
                    y_offset = 0.0; // No vertical offset
                }

                Some((
                    Size {
                        width: scaled_width,
                        height: scaled_height,
                    },
                    ComponentOffset {
                        x: x_offset.into(),
                        y: y_offset.into(),
                    },
                ))
            }
            ScaleMode::Original => {
                // ORIGINAL - use original dimensions, no scaling
                if original_width < container_width && original_height < container_height {
                    // Center the image in the container
                    let x_offset = (container_width - original_width) / 2.0;
                    let y_offset = (container_height - original_height) / 2.0;

                    Some((
                        Size {
                            width: original_width,
                            height: original_height,
                        },
                        ComponentOffset {
                            x: x_offset.into(),
                            y: y_offset.into(),
                        },
                    ))
                } else {
                    // Image is larger than the container, we will keep the original size and center it
                    let x_offset = (container_width - original_width) / 2.0;
                    let y_offset = (container_height - original_height) / 2.0;

                    Some((
                        Size {
                            width: original_width,
                            height: original_height,
                        },
                        ComponentOffset {
                            x: x_offset.into(),
                            y: y_offset.into(),
                        },
                    ))
                }
            }
        }
    }
}
