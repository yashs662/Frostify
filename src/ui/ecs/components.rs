use crate::{
    app::AppEvent,
    ui::{
        animation::Animation,
        color::Color,
        ecs::{
            BorderPosition, ComponentType, EcsComponent, EntityId, GradientColorStop, GradientType,
            builders::image::ScaleMode,
        },
        layout::{
            BorderRadius, Bounds, ClipBounds, ComponentOffset, FlexValue, Layout, LayoutSize,
            Position, Size,
        },
    },
    utils::AppFonts,
};
use cosmic_text::{
    Attrs, Buffer, CacheKeyFlags, Family, FontFeatures, FontSystem, Metrics, Shaping, Stretch,
    Style, SwashCache, Weight,
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
    pub notch: NotchType,
    pub notch_depth: f32,
    pub notch_flat_width: f32,
    pub notch_total_width: f32,
    pub notch_offset: f32,
    pub notch_position: NotchPosition,
}

impl VisualComponent {
    pub fn is_visible(&self) -> bool {
        self.opacity > 0.0
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub enum NotchType {
    #[default]
    None,
    Top,
    Right,
    Bottom,
    Left,
}

impl NotchType {
    pub fn to_u32(self) -> u32 {
        match self {
            NotchType::None => 0,
            NotchType::Top => 1,
            NotchType::Right => 2,
            NotchType::Bottom => 3,
            NotchType::Left => 4,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub enum NotchPosition {
    #[default]
    Center,
    Start,
    End,
}

impl NotchPosition {
    pub fn to_u32(self) -> u32 {
        match self {
            NotchPosition::Start => 0,
            NotchPosition::Center => 1,
            NotchPosition::End => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, EcsComponent)]
pub struct BoundsComponent {
    pub computed_bounds: Bounds,
    pub screen_size: Size<u32>,
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
    pub is_just_activated: bool,
    pub is_just_deactivated: bool,
    pub is_event_bubble_boundary: bool,
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
    pub render_data_buffer: wgpu::Buffer,
    pub sampler: wgpu::Sampler,
    pub bind_group: Option<wgpu::BindGroup>,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub index_buffer: Option<wgpu::Buffer>,
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
pub struct ModalComponent {
    pub renderable_children: Vec<EntityId>,
    pub non_renderable_children: Vec<EntityId>,
    pub is_open: bool,
    pub is_opening: bool,
    pub is_closing: bool,
    pub has_entry_animation: bool,
    pub has_exit_animation: bool,
}

#[derive(Debug, Clone, EcsComponent)]
pub struct TextComponent {
    pub text: String,
    pub font_size: f32,
    pub line_height_multiplier: f32,
    pub color: Color,
    pub font_family: Option<String>, // Add this field
    buffer: Option<Buffer>,
    metrics: Option<Metrics>,
    texture: Option<wgpu::Texture>,
    texture_view: Option<wgpu::TextureView>,
    needs_update: bool,
    cached_bounds: Option<Bounds>,
    bind_group_update_required: bool,
}

impl TextComponent {
    pub fn new(text: String, font_size: f32, line_height_multiplier: f32, color: Color) -> Self {
        Self {
            text,
            font_size,
            line_height_multiplier,
            color,
            font_family: None, // Default to system font
            buffer: None,
            metrics: None,
            texture: None,
            texture_view: None,
            needs_update: true,
            cached_bounds: None,
            bind_group_update_required: false,
        }
    }

    pub fn with_font_family(mut self, font_family: impl Into<String>) -> Self {
        self.font_family = Some(font_family.into());
        self
    }

    fn get_text_attrs(font_family: Option<&str>) -> Attrs<'_> {
        Attrs {
            color_opt: None,
            family: match font_family {
                Some(family) => Family::Name(family),
                None => Family::Monospace,
            },
            stretch: Stretch::Normal,
            style: Style::Normal,
            weight: Weight::BOLD,
            metadata: 0,
            cache_key_flags: CacheKeyFlags::empty(),
            metrics_opt: None,
            letter_spacing_opt: None,
            font_features: FontFeatures { features: vec![] },
        }
    }

    pub fn initialize_rendering(&mut self, font_system: &mut FontSystem) {
        if self.buffer.is_some() {
            panic!("TextComponent buffer already initialized");
        }
        let metrics = Metrics::new(self.font_size, self.font_size * self.line_height_multiplier);
        let mut buffer = Buffer::new(font_system, metrics);
        if self.font_family.is_none() {
            self.font_family = Some(AppFonts::default().as_family_name().to_string());
        }

        let attrs = TextComponent::get_text_attrs(self.font_family.as_deref());
        buffer.set_text(font_system, &self.text, &attrs, Shaping::Advanced);
        self.buffer = Some(buffer);
        self.metrics = Some(metrics);
        self.needs_update = true;
    }

    pub fn update_text(&mut self, new_text: String, font_system: &mut FontSystem) {
        if self.text != new_text {
            self.text = new_text;
            if let Some(ref mut buffer) = self.buffer {
                let attrs = TextComponent::get_text_attrs(self.font_family.as_deref());
                buffer.set_text(font_system, &self.text, &attrs, Shaping::Advanced);
                self.metrics = Some(buffer.metrics());
                self.needs_update = true;
            }
        }
    }

    pub fn update_bounds(&mut self, bounds: Bounds, font_system: &mut FontSystem) {
        let bounds_changed = match self.cached_bounds {
            Some(cached) => {
                cached.position.x != bounds.position.x
                    || cached.position.y != bounds.position.y
                    || cached.size.width != bounds.size.width
                    || cached.size.height != bounds.size.height
            }
            None => true,
        };

        if bounds_changed {
            self.cached_bounds = Some(bounds);
            if let Some(ref mut buffer) = self.buffer {
                buffer.set_size(
                    font_system,
                    Some(bounds.size.width),
                    Some(bounds.size.height),
                );
                self.needs_update = true;
            }
        }
    }

    pub fn measure_text(&self) -> Option<Size<f32>> {
        if let Some(ref buffer) = self.buffer {
            let mut max_width = 0.0f32;
            let mut total_height = 0.0f32;

            for run in buffer.layout_runs() {
                max_width = max_width.max(run.line_w);
                total_height += run.line_height;
            }

            // If no runs, fallback to buffer metrics
            if total_height == 0.0
                && let Some(run) = buffer.layout_runs().next() {
                    total_height = run.line_height;
                }

            if max_width == 0.0 || total_height == 0.0 {
                None
            } else {
                Some(Size {
                    width: max_width + 6.0,      // Add padding for side bearings
                    height: total_height + 12.0, // More padding for ascenders/descenders
                })
            }
        } else {
            None
        }
    }

    pub fn calculate_fit_to_size(
        &self,
        old_bounds: &Bounds,
    ) -> Option<(Size<f32>, ComponentOffset)> {
        if let Some(text_size) = self.measure_text() {
            if text_size != old_bounds.size {
                Some((text_size, ComponentOffset::default()))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn update_texture_if_needed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
    ) {
        if !self.needs_update {
            return;
        }

        let Some(ref mut buffer) = self.buffer else {
            return;
        };

        // Shape the buffer
        buffer.shape_until_scroll(font_system, false);

        // Calculate dimensions
        let (text_width, text_height) = {
            let mut max_width = 0.0f32;
            let mut total_height = 0.0f32;

            for run in buffer.layout_runs() {
                max_width = max_width.max(run.line_w);
                total_height += run.line_height;
            }

            // If no runs, fallback to buffer metrics
            if total_height == 0.0
                && let Some(run) = buffer.layout_runs().next() {
                    total_height = run.line_height;
                }

            // Add padding, especially for descenders
            let width = (max_width.ceil() as u32 + 6).max(1); // Extra padding for side bearings
            let height = (total_height.ceil() as u32 + 12).max(1); // More padding for ascenders/descenders

            (width, height)
        };

        if text_width == 0 || text_height == 0 {
            return;
        }

        // Create texture
        let texture_size = wgpu::Extent3d {
            width: text_width,
            height: text_height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Text Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create image buffer
        let mut image_data = vec![0u8; (text_width * text_height * 4) as usize];

        let start_instant = std::time::Instant::now();

        // Draw text - TODO: Optimize this part
        buffer.draw(
            font_system,
            swash_cache,
            self.color.to_cosmic_color(),
            |x, y, w, h, color| {
                // Fill the rectangle with the given color
                for py in 0..h {
                    for px in 0..w {
                        let buffer_x = x + px as i32 + 3; // TEXT_TEXTURE_HORIZONTAL_PADDING
                        let buffer_y = y + py as i32 + 5; // TEXT_TEXTURE_VERTICAL_PADDING

                        if buffer_x >= 0
                            && buffer_y >= 0
                            && buffer_x < text_width as i32
                            && buffer_y < text_height as i32
                        {
                            let index =
                                ((buffer_y as u32 * text_width + buffer_x as u32) * 4) as usize;
                            if index + 3 < image_data.len() {
                                // Use alpha blending to combine overlapping glyphs
                                let src_alpha = color.a() as f32 / 255.0;
                                let dst_alpha = image_data[index + 3] as f32 / 255.0;

                                if src_alpha > 0.0 {
                                    // Pre-multiply alpha for better blending
                                    let src_r = (color.r() as f32 / 255.0) * src_alpha;
                                    let src_g = (color.g() as f32 / 255.0) * src_alpha;
                                    let src_b = (color.b() as f32 / 255.0) * src_alpha;

                                    let dst_r = (image_data[index] as f32 / 255.0) * dst_alpha;
                                    let dst_g = (image_data[index + 1] as f32 / 255.0) * dst_alpha;
                                    let dst_b = (image_data[index + 2] as f32 / 255.0) * dst_alpha;

                                    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);

                                    if out_alpha > 0.0 {
                                        let inv_out_alpha = 1.0 / out_alpha;
                                        image_data[index] = ((src_r + dst_r * (1.0 - src_alpha))
                                            * inv_out_alpha
                                            * 255.0)
                                            as u8;
                                        image_data[index + 1] = ((src_g
                                            + dst_g * (1.0 - src_alpha))
                                            * inv_out_alpha
                                            * 255.0)
                                            as u8;
                                        image_data[index + 2] = ((src_b
                                            + dst_b * (1.0 - src_alpha))
                                            * inv_out_alpha
                                            * 255.0)
                                            as u8;
                                        image_data[index + 3] = (out_alpha * 255.0) as u8;
                                    }
                                }
                            }
                        }
                    }
                }
            },
        );

        log::trace!(
            "Text rendering took {:?} for {}x{}",
            start_instant.elapsed(),
            text_width,
            text_height
        );

        // Upload to GPU
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(text_width * 4),
                rows_per_image: Some(text_height),
            },
            texture_size,
        );

        // Update component state
        self.texture = Some(texture);
        self.texture_view = Some(texture_view);
        self.needs_update = false;
        self.bind_group_update_required = true;
    }

    pub fn get_texture_view(&self) -> Option<&wgpu::TextureView> {
        self.texture_view.as_ref()
    }

    pub fn bind_group_update_required(&self) -> bool {
        self.bind_group_update_required
    }

    pub fn reset_bind_group_update_required(&mut self) {
        self.bind_group_update_required = false;
    }
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
    pub fn calculate_fit_to_size(
        &self,
        old_bounds: &Bounds,
    ) -> Option<(Size<f32>, ComponentOffset)> {
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
