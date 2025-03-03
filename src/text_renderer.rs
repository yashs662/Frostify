use crate::{
    color::Color,
    ui::layout::{Bounds, ComponentSize},
};
use glyphon::Metrics;
use std::collections::HashMap;
use uuid::Uuid;

struct TextRenderData {
    buffer: glyphon::Buffer,
    metrics: Metrics,
    bounds: Bounds,
    color: Color,
}

impl TextRenderData {
    pub fn new(buffer: glyphon::Buffer, metrics: Metrics, bounds: Bounds, color: Color) -> Self {
        Self {
            buffer,
            metrics,
            bounds,
            color,
        }
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
    buffers: HashMap<Uuid, TextRenderData>,
}

impl TextRenderBuffers {
    fn new() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }

    fn get(&self, id: Uuid) -> Option<&TextRenderData> {
        self.buffers.get(&id)
    }

    fn register(
        &mut self,
        id: Uuid,
        buffer: glyphon::Buffer,
        metrics: Metrics,
        bounds: Bounds,
        color: Color,
    ) {
        self.buffers
            .insert(id, TextRenderData::new(buffer, metrics, bounds, color));
    }

    fn create_buffer(
        font_system: &mut glyphon::FontSystem,
        font_metrics: glyphon::Metrics,
        size: ComponentSize,
        text: &str,
    ) -> glyphon::Buffer {
        let mut buffer = glyphon::Buffer::new(font_system, font_metrics);
        buffer.set_size(font_system, Some(size.width), Some(size.height));
        buffer.set_text(
            font_system,
            text,
            glyphon::Attrs::new().family(glyphon::Family::SansSerif),
            glyphon::Shaping::Advanced,
        );
        buffer
    }
}

pub struct TextHandler {
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
    viewport: glyphon::Viewport,
    text_renderer: glyphon::TextRenderer,
    atlas: glyphon::TextAtlas,
    buffers: TextRenderBuffers,
}

impl TextHandler {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
    ) -> Self {
        let font_system = glyphon::FontSystem::new();
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(device);
        let mut viewport = glyphon::Viewport::new(device, &cache);
        let mut atlas = glyphon::TextAtlas::new(device, queue, &cache, config.format);

        let text_renderer =
            glyphon::TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        viewport.update(
            queue,
            glyphon::Resolution {
                width: config.width,
                height: config.height,
            },
        );

        let buffers = TextRenderBuffers::new();

        Self {
            font_system,
            swash_cache,
            viewport,
            text_renderer,
            atlas,
            buffers,
        }
    }

    pub fn register_text(
        &mut self,
        id: Uuid,
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
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
    ) {
        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: config.width,
                height: config.height,
            },
        );
    }

    pub fn render<'a>(
        &'a mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        let text_areas = self
            .buffers
            .buffers
            .values()
            .map(|text_render_data| glyphon::TextArea {
                buffer: &text_render_data.buffer,
                left: text_render_data.bounds.position.x,
                top: text_render_data.bounds.position.y,
                scale: 1.0,
                bounds: glyphon::TextBounds::default(),
                default_color: text_render_data.color.to_glyphon_color(),
                custom_glyphs: &[],
            });

        self.text_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();

        self.text_renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }

    pub fn trim_atlas(&mut self) {
        self.atlas.trim();
    }

    pub fn update(&mut self, update_data: (Uuid, OptionalTextUpdateData)) {
        let (key, value) = update_data;
        if let Some(data) = self.buffers.buffers.get_mut(&key) {
            if let Some(updated_text) = value.text {
                data.buffer.set_text(
                    &mut self.font_system,
                    &updated_text,
                    glyphon::Attrs::new().family(glyphon::Family::SansSerif),
                    glyphon::Shaping::Advanced,
                );
            } else if let Some(updated_metrics) = value.metrics {
                data.metrics = updated_metrics;
                data.buffer
                    .set_metrics(&mut self.font_system, updated_metrics);
            } else if let Some(updated_bounds) = value.bounds {
                data.bounds = updated_bounds;
                data.buffer.set_size(
                    &mut self.font_system,
                    Some(updated_bounds.size.width),
                    Some(updated_bounds.size.height),
                );
            } else if let Some(updated_color) = value.color {
                data.color = updated_color;
            }
        }
    }

    pub fn measure_text(&self, id: Uuid) -> Option<ComponentSize> {
        if let Some(data) = self.buffers.get(id) {
            let buffer = &data.buffer;
            let (width, total_lines) = buffer
                .layout_runs()
                .fold((0.0, 0usize), |(width, total_lines), run| {
                    (run.line_w.max(width), total_lines + 1)
                });

            Some(ComponentSize {
                width,
                height: (total_lines as f32 * buffer.metrics().line_height),
            })
        } else {
            None
        }
    }
}
