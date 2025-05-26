use crate::{
    constants::{TEXT_TEXTURE_HORIZONTAL_PADDING, TEXT_TEXTURE_VERTICAL_PADDING},
    ui::{
        color::Color,
        ecs::EntityId,
        layout::{Bounds, Size},
    },
};
use cosmic_text::{
    Attrs, Buffer, CacheKeyFlags, Family, FontFeatures, FontSystem, Metrics, Shaping, Stretch,
    Style, SwashCache, Weight,
};
use std::collections::HashMap;

const TEXT_ATTRS: Attrs = Attrs {
    color_opt: None,
    family: Family::SansSerif,
    stretch: Stretch::Normal,
    style: Style::Normal,
    weight: Weight::BOLD,
    metadata: 0,
    cache_key_flags: CacheKeyFlags::empty(),
    metrics_opt: None,
    letter_spacing_opt: None,
    font_features: FontFeatures { features: vec![] },
};

struct TextRenderData {
    buffer: Buffer,
    metrics: Metrics,
    bounds: Bounds,
    color: Color,
    texture: Option<wgpu::Texture>,
    texture_view: Option<wgpu::TextureView>,
    needs_update: bool,
}

impl TextRenderData {
    pub fn new(buffer: Buffer, metrics: Metrics, bounds: Bounds, color: Color) -> Self {
        Self {
            buffer,
            metrics,
            bounds,
            color,
            texture: None,
            texture_view: None,
            needs_update: true,
        }
    }

    pub fn mark_for_update(&mut self) {
        self.needs_update = true;
    }
}

#[derive(Debug)]
pub struct OptionalTextUpdateData {
    text: Option<String>,
    metrics: Option<Metrics>,
    bounds: Option<Bounds>,
    color: Option<Color>,
}

impl OptionalTextUpdateData {
    pub fn new() -> Self {
        Self {
            text: None,
            metrics: None,
            bounds: None,
            color: None,
        }
    }

    pub fn with_bounds(mut self, bounds: Bounds) -> Self {
        self.bounds = Some(bounds);
        self
    }
}

struct TextRenderBuffers {
    buffers: HashMap<EntityId, TextRenderData>,
}

impl TextRenderBuffers {
    fn new() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }

    fn get(&self, id: EntityId) -> Option<&TextRenderData> {
        self.buffers.get(&id)
    }

    fn register(
        &mut self,
        id: EntityId,
        buffer: Buffer,
        metrics: Metrics,
        bounds: Bounds,
        color: Color,
    ) {
        self.buffers
            .insert(id, TextRenderData::new(buffer, metrics, bounds, color));
    }

    fn create_buffer(
        font_system: &mut FontSystem,
        font_metrics: Metrics,
        size: Size<f32>,
        text: &str,
    ) -> Buffer {
        let mut buffer = Buffer::new(font_system, font_metrics);
        buffer.set_size(font_system, Some(size.width), Some(size.height));
        buffer.set_text(
            font_system,
            text,
            &TEXT_ATTRS,
            Shaping::Advanced,
        );
        buffer
    }
}

pub struct TextHandler {
    font_system: FontSystem,
    swash_cache: SwashCache,
    buffers: TextRenderBuffers,
    sampler: wgpu::Sampler,
}

impl TextHandler {
    pub fn new(
        device: &wgpu::Device,
        _config: &wgpu::SurfaceConfiguration,
        _queue: &wgpu::Queue,
    ) -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let buffers = TextRenderBuffers::new();

        // Create a sampler for text textures
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            font_system,
            swash_cache,
            buffers,
            sampler,
        }
    }

    pub fn register_text(
        &mut self,
        id: EntityId,
        text: String,
        font_size: f32,
        line_height_multiplier: f32,
        bounds: Bounds,
        color: Color,
    ) {
        let font_metrics = Metrics::new(font_size, font_size * line_height_multiplier);
        let buffer = TextRenderBuffers::create_buffer(
            &mut self.font_system,
            font_metrics,
            bounds.size,
            &text,
        );
        self.buffers
            .register(id, buffer, font_metrics, bounds, color);
    }

    pub fn update_viewport_size(
        &mut self,
        _config: &wgpu::SurfaceConfiguration,
        _queue: &wgpu::Queue,
    ) {
        // No viewport management needed with cosmic-text approach
    }

    pub fn update_texture_if_needed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: EntityId,
    ) {
        // First check if update is needed without borrowing
        let needs_update = self
            .buffers
            .buffers
            .get(&id)
            .map(|data| data.needs_update)
            .unwrap_or(false);

        if !needs_update {
            return;
        }

        log::debug!("Updating texture for text entity: {}", id);

        // Shape the buffer and calculate dimensions
        if let Some(text_data) = self.buffers.buffers.get_mut(&id) {
            text_data
                .buffer
                .shape_until_scroll(&mut self.font_system, false);
        }

        // Calculate dimensions
        let (text_width, text_height, color) =
            if let Some(text_data) = self.buffers.buffers.get(&id) {
                let (width, height) = self.calculate_text_dimensions(&text_data.buffer);
                (width, height, text_data.color)
            } else {
                return;
            };

        if text_width == 0 || text_height == 0 {
            log::warn!(
                "Text dimensions are zero for entity {}: {}x{}",
                id,
                text_width,
                text_height
            );
            return;
        }

        log::debug!(
            "Creating texture for entity {} with dimensions: {}x{}",
            id,
            text_width,
            text_height
        );

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

        // Draw text - need mutable access to buffer
        if let Some(text_data) = self.buffers.buffers.get_mut(&id) {
            text_data.buffer.draw(
                &mut self.font_system,
                &mut self.swash_cache,
                color.to_cosmic_color(),
                |x, y, w, h, color| {
                    // Fill the rectangle with the given color
                    for py in 0..h {
                        for px in 0..w {
                            let buffer_x = x + px as i32 + TEXT_TEXTURE_HORIZONTAL_PADDING;
                            let buffer_y = y + py as i32 + TEXT_TEXTURE_VERTICAL_PADDING;

                            // More permissive bounds checking - allow drawing anywhere in the texture
                            if buffer_x >= 0
                                && buffer_y >= 0
                                && buffer_x < text_width as i32
                                && buffer_y < text_height as i32
                            {
                                let index =
                                    ((buffer_y as u32 * text_width + buffer_x as u32) * 4) as usize;
                                if index + 3 < image_data.len() {
                                    // Use alpha blending to combine overlapping glyphs instead of overwriting
                                    let src_alpha = color.a() as f32 / 255.0;
                                    let dst_alpha = image_data[index + 3] as f32 / 255.0;

                                    if src_alpha > 0.0 {
                                        // Alpha blending: result = src * src_alpha + dst * (1 - src_alpha)
                                        let inv_alpha = 1.0 - src_alpha;

                                        image_data[index] = ((color.r() as f32 * src_alpha
                                            + image_data[index] as f32 * inv_alpha)
                                            .min(255.0))
                                            as u8;
                                        image_data[index + 1] = ((color.g() as f32 * src_alpha
                                            + image_data[index + 1] as f32 * inv_alpha)
                                            .min(255.0))
                                            as u8;
                                        image_data[index + 2] = ((color.b() as f32 * src_alpha
                                            + image_data[index + 2] as f32 * inv_alpha)
                                            .min(255.0))
                                            as u8;
                                        image_data[index + 3] =
                                            ((src_alpha + dst_alpha * (1.0 - src_alpha)) * 255.0)
                                                .min(255.0)
                                                as u8;
                                    }
                                }
                            }
                        }
                    }
                },
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

            // Update texture data
            text_data.texture = Some(texture);
            text_data.texture_view = Some(texture_view);
            text_data.needs_update = false;

            log::debug!(
                "Successfully created and uploaded texture for entity {}",
                id
            );
        }
    }

    /// Get the texture view for a text component (used by the main rendering system)
    pub fn get_texture_view(&self, id: EntityId) -> Option<&wgpu::TextureView> {
        let result = self.buffers.buffers.get(&id)?.texture_view.as_ref();
        if result.is_none() {
            log::debug!("No texture view found for text entity {}", id);
        }
        result
    }

    /// Get the sampler for text textures
    pub fn get_sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    fn calculate_text_dimensions(&self, buffer: &Buffer) -> (u32, u32) {
        let mut max_width = 0.0f32;
        let mut total_height = 0.0f32;

        for run in buffer.layout_runs() {
            max_width = max_width.max(run.line_w);
            total_height += run.line_height;
        }

        // If no runs, fallback to buffer metrics
        if total_height == 0.0 {
            if let Some(run) = buffer.layout_runs().next() {
                total_height = run.line_height;
            }
        }

        // Add more padding, especially for descenders
        let width = (max_width.ceil() as u32 + 6).max(1); // Extra padding for side bearings
        let height = (total_height.ceil() as u32 + 12).max(1); // More padding for ascenders/descenders

        log::debug!(
            "Text dimensions calculated: {}x{} (max_width: {}, total_height: {})",
            width,
            height,
            max_width,
            total_height
        );

        (width, height)
    }

    pub fn update(&mut self, update_data: (EntityId, OptionalTextUpdateData)) {
        let (key, value) = update_data;
        if let Some(data) = self.buffers.buffers.get_mut(&key) {
            let mut needs_reshape = false;

            if let Some(updated_text) = value.text {
                data.buffer.set_text(
                    &mut self.font_system,
                    &updated_text,
                    &TEXT_ATTRS,
                    Shaping::Advanced,
                );
                needs_reshape = true;
            }

            if let Some(updated_metrics) = value.metrics {
                data.metrics = updated_metrics;
                data.buffer
                    .set_metrics(&mut self.font_system, updated_metrics);
                needs_reshape = true;
            }

            if let Some(updated_bounds) = value.bounds {
                data.bounds = updated_bounds;
                data.buffer.set_size(
                    &mut self.font_system,
                    Some(updated_bounds.size.width),
                    Some(updated_bounds.size.height),
                );
                needs_reshape = true;
            }

            if let Some(updated_color) = value.color {
                data.color = updated_color;
                needs_reshape = true;
            }

            if needs_reshape {
                data.mark_for_update();
            }
        }
    }

    pub fn measure_text(&self, id: EntityId) -> Option<Size<f32>> {
        if let Some(data) = self.buffers.get(id) {
            let buffer = &data.buffer;
            let mut max_width = 0.0f32;
            let mut total_height = 0.0f32;

            for run in buffer.layout_runs() {
                max_width = max_width.max(run.line_w);
                total_height += run.line_height;
            }

            // If no runs, fallback to buffer metrics
            if total_height == 0.0 {
                if let Some(run) = buffer.layout_runs().next() {
                    total_height = run.line_height;
                }
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
}
