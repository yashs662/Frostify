use std::time::Duration;

use crate::{
    app::AppEvent,
    ui::{
        Configurable, Positionable, Renderable,
        color::Color,
        components::core::{
            background_color::BackgroundColorComponent,
            background_gradient::BackgroundGradientComponent, image::ImageComponent,
            text::TextComponent,
        },
        layout::{
            Bounds, ComponentOffset, ComponentSize, ComponentTransform, Layout, Position, Size,
        },
    },
    wgpu_ctx::{AppPipelines, WgpuCtx},
};
use colorgrad::Gradient;
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use super::{
    components::{
        core::{frosted_glass::FrostedGlassComponent, image::ImageMetadata},
        image::ScaleMode,
    },
    layout::{BorderRadius, ComponentPosition, FlexValue},
};

/// Defines the position of the border relative to the component edges
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[allow(dead_code)]
pub enum BorderPosition {
    /// Border drawn inside the component's bounds
    Inside,
    /// Border straddles the component's edges
    Center,
    /// Border drawn outside the component's bounds
    #[default]
    Outside,
}

#[derive(Debug, Clone)]
pub enum ComponentHoverEffects {
    BackgroundColor(Color, Duration), // Color and duration in seconds
}

#[derive(Debug, Clone)]
pub struct Component {
    pub id: Uuid,
    pub debug_name: Option<String>,
    pub component_type: ComponentType,
    pub transform: ComponentTransform,
    pub layout: Layout,
    pub children_ids: Vec<Uuid>,
    parent_id: Option<Uuid>,
    pub metadata: Vec<ComponentMetaData>,
    pub config: Option<ComponentConfig>,
    screen_size: ComponentSize,
    requires_children_extraction: bool,
    is_clickable: bool,
    is_draggable: bool,
    pub border_width: f32,
    pub border_color: Color,
    pub border_position: BorderPosition,
    pub fit_to_size: bool,
    pub computed_bounds: Bounds,
    pub hover_effects: Option<ComponentHoverEffects>,
    animation_state: f32,
    animation_going_forward: bool,
    clean_config_copy: Option<ComponentConfig>,
    is_hovered: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    Container,
    Text,
    Image,
    BackgroundColor,
    BackgroundGradient,
    FrostedGlass,
}

#[derive(Debug, Clone)]
pub enum ComponentMetaData {
    ClickEvent(AppEvent),
    BindGroup(wgpu::BindGroup),
    RenderDataBuffer(wgpu::Buffer),
    EventSender(UnboundedSender<AppEvent>),
    DragEvent(AppEvent),
    ChildComponents(Vec<Component>),
    ImageMetadata(ImageMetadata),
    Sampler(wgpu::Sampler),
    CanBeResizedTo(ComponentSize),
}

#[derive(Debug, Clone)]
pub enum ComponentConfig {
    BackgroundColor(BackgroundColorConfig),
    BackgroundGradient(BackgroundGradientConfig),
    Text(TextConfig),
    Image(ImageConfig),
    FrostedGlass(FrostedGlassConfig),
}

#[derive(Debug, Clone)]
pub struct BackgroundGradientConfig {
    pub color_stops: Vec<GradientColorStop>,
    pub gradient_type: GradientType,
    pub angle: f32,                 // Angle in degrees (for linear gradients)
    pub center: Option<(f32, f32)>, // Center position for radial gradients (0.0-1.0 range)
    pub radius: Option<f32>,        // Radius for radial gradients (0.0-1.0 range)
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientType {
    Linear,
    Radial,
    // Can be extended with Conic later
}

#[derive(Debug, Clone)]
pub struct GradientColorStop {
    pub color: Color,
    pub position: f32, // 0.0 to 1.0 representing the position along the gradient line
}

#[derive(Debug, Clone)]
pub struct ImageConfig {
    pub file_name: String,
    pub scale_mode: ScaleMode,
}

#[derive(Debug, Clone)]
pub struct BackgroundColorConfig {
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct TextConfig {
    pub text: String,
    pub font_size: f32,
    pub line_height: f32,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct FrostedGlassConfig {
    pub tint_color: Color,
    pub blur_radius: f32, // Blur intensity (0-10)
    pub opacity: f32,     // Overall opacity (0.0-1.0)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ComponentBufferData {
    pub color: [f32; 4],
    pub position: [f32; 2],      // Position in pixels (top-left corner)
    pub size: [f32; 2],          // Size in pixels (width, height)
    pub border_radius: [f32; 4], // Corner radii in pixels (top-left, top-right, bottom-left, bottom-right)
    pub screen_size: [f32; 2],   // Viewport dimensions in pixels
    pub use_texture: u32,        // Flag: 0 for color, 1 for texture, 2 for frosted glass
    pub blur_radius: f32,        // Blur amount for frosted glass (0-10)
    pub opacity: f32,            // Overall opacity for frosted glass (0.0-1.0)
    pub _padding: [f32; 3],
    pub border_color: [f32; 4], // Border color
    pub border_width: f32,      // Border thickness in pixels
    pub border_position: u32,   // Border position: 0=inside, 1=center, 2=outside
    pub _padding2: [f32; 2],
}

impl ComponentConfig {
    pub fn get_text_config(self) -> Option<TextConfig> {
        match self {
            Self::Text(config) => Some(config),
            _ => None,
        }
    }

    pub fn get_image_config(self) -> Option<ImageConfig> {
        match self {
            Self::Image(config) => Some(config),
            _ => None,
        }
    }

    pub fn get_gradient_config(self) -> Option<BackgroundGradientConfig> {
        match self {
            Self::BackgroundGradient(config) => Some(config),
            _ => None,
        }
    }
}

impl Component {
    pub fn new(id: Uuid, component_type: ComponentType) -> Self {
        Self {
            id,
            debug_name: None,
            component_type,
            transform: ComponentTransform {
                size: Size::fill(),
                offset: ComponentOffset { x: 0.0, y: 0.0 },
                position_type: Position::Flex,
                z_index: 0,
                border_radius: BorderRadius::zero(),
            },
            layout: Layout::new(),
            children_ids: Vec::new(),
            parent_id: None,
            metadata: Vec::new(),
            config: None,
            screen_size: ComponentSize::default(),
            requires_children_extraction: false,
            is_clickable: false,
            is_draggable: false,
            border_width: 0.0,
            border_color: Color::Transparent,
            border_position: BorderPosition::default(),
            fit_to_size: false,
            computed_bounds: Bounds::default(),
            hover_effects: None,
            animation_state: 0.0,
            animation_going_forward: true,
            clean_config_copy: None,
            is_hovered: false,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.layout.opacity > 0.0
    }

    pub fn is_hit(&self, position: ComponentPosition) -> bool {
        let bounds = self.computed_bounds;
        let x = position.x;
        let y = position.y;
        x >= bounds.position.x
            && x <= bounds.position.x + bounds.size.width
            && y >= bounds.position.y
            && y <= bounds.position.y + bounds.size.height
    }

    pub fn get_parent_id(&self) -> Option<Uuid> {
        self.parent_id
    }

    pub fn requires_to_be_drawn(&self) -> bool {
        !matches!(
            self.component_type,
            ComponentType::Text | ComponentType::Container
        )
    }

    pub fn flag_children_extraction(&mut self) {
        self.requires_children_extraction = true;
    }

    pub fn requires_children_extraction(&self) -> bool {
        self.requires_children_extraction
    }

    pub fn set_debug_name(&mut self, name: impl Into<String>) {
        self.debug_name = Some(name.into());
    }

    pub fn set_border_radius(&mut self, radius: BorderRadius) {
        self.transform.border_radius = radius;
    }

    pub fn set_fit_to_size(&mut self, fit_to_size: bool) {
        self.fit_to_size = fit_to_size;
    }

    pub fn get_all_children(&self) -> Vec<Uuid> {
        self.children_ids.clone()
    }

    pub fn set_z_index(&mut self, z_index: i32) {
        self.transform.z_index = z_index;
    }

    pub fn is_hoverable(&self) -> bool {
        self.hover_effects.is_some()
    }

    pub fn set_hover_effects(&mut self, effects: ComponentHoverEffects) {
        self.hover_effects = Some(effects);
        self.clean_config_copy = self.config.clone();
    }

    pub fn set_hover_state(&mut self, is_hovered: bool) {
        self.is_hovered = is_hovered;
    }

    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    pub fn draw(&mut self, render_pass: &mut wgpu::RenderPass, app_pipelines: &mut AppPipelines) {
        if self.config.is_some() {
            match self.component_type {
                ComponentType::BackgroundColor => {
                    BackgroundColorComponent::draw(self, render_pass, app_pipelines);
                }
                ComponentType::BackgroundGradient => {
                    BackgroundGradientComponent::draw(self, render_pass, app_pipelines);
                }
                ComponentType::Text => {
                    TextComponent::draw(self, render_pass, app_pipelines);
                }
                ComponentType::Image => {
                    ImageComponent::draw(self, render_pass, app_pipelines);
                }
                ComponentType::FrostedGlass => {
                    FrostedGlassComponent::draw(self, render_pass, app_pipelines);
                }
                ComponentType::Container => {
                    // Containers are not drawn directly
                }
            }
        }
    }

    pub fn configure(&mut self, config: ComponentConfig, wgpu_ctx: &mut WgpuCtx) {
        self.config = Some(config.clone());

        match config {
            ComponentConfig::BackgroundColor(_) => {
                for metadata in BackgroundColorComponent::configure(self, config, wgpu_ctx) {
                    self.metadata.push(metadata);
                }
            }
            ComponentConfig::BackgroundGradient(_) => {
                for metadata in BackgroundGradientComponent::configure(self, config, wgpu_ctx) {
                    self.metadata.push(metadata);
                }
            }
            ComponentConfig::Text(_) => {
                for metadata in TextComponent::configure(self, config, wgpu_ctx) {
                    self.metadata.push(metadata);
                }
            }
            ComponentConfig::Image(_) => {
                for metadata in ImageComponent::configure(self, config, wgpu_ctx) {
                    self.metadata.push(metadata);
                }
            }
            ComponentConfig::FrostedGlass(_) => {
                for metadata in FrostedGlassComponent::configure(self, config, wgpu_ctx) {
                    self.metadata.push(metadata);
                }
            }
        }
    }

    pub fn set_position(
        &mut self,
        wgpu_ctx: &mut WgpuCtx,
        bounds: Bounds,
        screen_size: ComponentSize,
    ) {
        self.screen_size = screen_size;
        self.computed_bounds = bounds;
        if let Some(config) = &self.config {
            match config {
                ComponentConfig::BackgroundColor(_) => {
                    BackgroundColorComponent::set_position(self, wgpu_ctx, bounds);
                }
                ComponentConfig::BackgroundGradient(_) => {
                    BackgroundGradientComponent::set_position(self, wgpu_ctx, bounds);
                }
                ComponentConfig::Image(_) => {
                    ImageComponent::set_position(self, wgpu_ctx, bounds);
                }
                ComponentConfig::Text(_) => {
                    TextComponent::set_position(self, wgpu_ctx, bounds);
                }
                ComponentConfig::FrostedGlass(_) => {
                    FrostedGlassComponent::set_position(self, wgpu_ctx, bounds);
                }
            }
        }
    }

    pub fn needs_update(&self) -> bool {
        (self.is_hovered && self.animation_state < 1.0)
            || (!self.is_hovered && self.animation_state > 0.0)
    }

    pub fn update(&mut self, wgpu_ctx: &mut WgpuCtx, frame_time: f32) {
        let need_to_update = self.needs_update();
        if !need_to_update {
            return;
        }

        let mut should_be_updated = false;

        let base_color =
            if let Some(ComponentConfig::BackgroundColor(BackgroundColorConfig { color })) =
                self.clean_config_copy
            {
                color
            } else {
                Color::Transparent
            };

        if let Some(ComponentHoverEffects::BackgroundColor(color, duration)) = &self.hover_effects {
            // frame time is in milliseconds, convert to seconds for proper delta calculation
            let delta = frame_time / duration.as_millis() as f32;

            if self.is_hovered {
                self.animation_state += delta;
                if self.animation_state >= 1.0 {
                    self.animation_state = 1.0;
                }
            } else {
                self.animation_state -= delta;
                if self.animation_state <= 0.0 {
                    self.animation_state = 0.0;
                }
            }

            // Clamp the animation state between 0.0 and 1.0
            self.animation_state = self.animation_state.clamp(0.0, 1.0);

            // Calculate the color based on the animation state
            let colors = if self.animation_going_forward {
                vec![base_color.to_colorgrad_color(), color.to_colorgrad_color()]
            } else {
                vec![color.to_colorgrad_color(), base_color.to_colorgrad_color()]
            };

            let gradient = colorgrad::GradientBuilder::new()
                .colors(&colors)
                .domain(&[0.0, 1.0])
                .build::<colorgrad::LinearGradient>()
                .unwrap();

            let color = gradient.at(self.animation_state);
            let color = Color::Custom([color.r, color.g, color.b, color.a]);

            self.config = Some(ComponentConfig::BackgroundColor(BackgroundColorConfig {
                color,
            }));
            should_be_updated = true;
        }

        if should_be_updated {
            if let Some(buffer) = self.get_render_data_buffer() {
                wgpu_ctx.queue.write_buffer(
                    buffer,
                    0,
                    bytemuck::cast_slice(&[self.get_render_data(self.computed_bounds)]),
                );
            }
        }
    }

    pub fn add_child(&mut self, child: Component) {
        let mut child = child;
        child.set_parent(self.id);
        self.children_ids.push(child.id);
        if let Some(ComponentMetaData::ChildComponents(existing_children)) = self
            .metadata
            .iter_mut()
            .find(|m| matches!(m, ComponentMetaData::ChildComponents(_)))
        {
            existing_children.push(child);
        } else {
            self.metadata
                .push(ComponentMetaData::ChildComponents(vec![child]));
        }
        self.flag_children_extraction();
    }

    pub fn set_parent(&mut self, parent_id: Uuid) {
        self.parent_id = Some(parent_id);
    }

    pub fn resize_to_metadata(&mut self) {
        if let Some(size) = self.can_be_resized_to_metadata() {
            self.transform.size.width = FlexValue::Fixed(size.width);
            self.transform.size.height = FlexValue::Fixed(size.height);
        }
    }

    pub fn is_frosted_component(&self) -> bool {
        matches!(self.component_type, ComponentType::FrostedGlass)
    }

    pub fn is_text_component(&self) -> bool {
        matches!(self.component_type, ComponentType::Text)
    }

    pub fn remove_resize_metadata(&mut self) {
        self.metadata
            .retain(|m| !matches!(m, ComponentMetaData::CanBeResizedTo(_)));
    }

    fn get_metadata<T>(&self, matcher: fn(&ComponentMetaData) -> Option<&T>) -> Option<&T> {
        self.metadata.iter().find_map(matcher)
    }

    pub fn can_be_resized_to_metadata(&self) -> Option<ComponentSize> {
        self.metadata.iter().find_map(|m| match m {
            ComponentMetaData::CanBeResizedTo(size) => Some(*size),
            _ => None,
        })
    }

    pub fn get_image_metadata(&self) -> Option<&ImageMetadata> {
        self.metadata.iter().find_map(|m| match m {
            ComponentMetaData::ImageMetadata(metadata) => Some(metadata),
            _ => None,
        })
    }

    pub fn get_render_data_buffer(&self) -> Option<&wgpu::Buffer> {
        self.get_metadata(|m| match m {
            ComponentMetaData::RenderDataBuffer(buf) => Some(buf),
            _ => None,
        })
    }

    pub fn get_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.get_metadata(|m| match m {
            ComponentMetaData::BindGroup(group) => Some(group),
            _ => None,
        })
    }

    pub fn get_event_sender(&self) -> Option<&UnboundedSender<AppEvent>> {
        self.metadata.iter().find_map(|m| match m {
            ComponentMetaData::EventSender(sender) => Some(sender),
            _ => None,
        })
    }

    pub fn get_click_event(&self) -> Option<&AppEvent> {
        self.metadata.iter().find_map(|m| match m {
            ComponentMetaData::ClickEvent(event) => Some(event),
            _ => None,
        })
    }

    pub fn get_children_from_metadata(&self) -> Option<&Vec<Component>> {
        self.metadata.iter().find_map(|m| match m {
            ComponentMetaData::ChildComponents(children) => Some(children),
            _ => None,
        })
    }

    pub fn set_click_event(&mut self, event: AppEvent) {
        self.metadata.push(ComponentMetaData::ClickEvent(event));
        if self.get_event_sender().is_some() {
            self.is_clickable = true;
        }
    }

    pub fn is_clickable(&self) -> bool {
        self.is_clickable
    }

    pub fn set_drag_event(&mut self, event: AppEvent) {
        self.metadata.push(ComponentMetaData::DragEvent(event));
        if self.get_event_sender().is_some() {
            self.is_draggable = true;
        }
    }

    pub fn set_event_sender(&mut self, sender: UnboundedSender<AppEvent>) {
        self.metadata.push(ComponentMetaData::EventSender(sender));
        if self.get_click_event().is_some() {
            self.is_clickable = true;
        }
        if self.get_drag_event().is_some() {
            self.is_draggable = true;
        }
    }

    pub fn get_drag_event(&self) -> Option<&AppEvent> {
        self.metadata.iter().find_map(|m| match m {
            ComponentMetaData::DragEvent(event) => Some(event),
            _ => None,
        })
    }

    pub fn is_draggable(&self) -> bool {
        self.is_draggable
    }

    pub fn get_render_data(&self, bounds: Bounds) -> ComponentBufferData {
        let default_color = [1.0, 0.0, 1.0, 1.0];
        let position = [bounds.position.x, bounds.position.y];
        let size = [bounds.size.width, bounds.size.height];

        // Get color and frosted glass parameters if available
        let (color, blur_radius, opacity) = match &self.config {
            Some(ComponentConfig::BackgroundColor(BackgroundColorConfig { color })) => {
                (color.value(), 0.0, 1.0)
            }
            Some(ComponentConfig::FrostedGlass(FrostedGlassConfig {
                tint_color,
                blur_radius,
                opacity,
            })) => (tint_color.value(), *blur_radius, *opacity),
            _ => (default_color, 0.0, 1.0),
        };

        let use_texture = match &self.config {
            Some(ComponentConfig::BackgroundGradient(_)) | Some(ComponentConfig::Image(_)) => 1,
            Some(ComponentConfig::FrostedGlass(_)) => 2,
            _ => 0,
        };

        let border_radius = self.transform.border_radius.values();

        // Convert border position enum to u32 for shader
        let border_position_value = match self.border_position {
            BorderPosition::Inside => 0u32,
            BorderPosition::Center => 1u32,
            BorderPosition::Outside => 2u32,
        };

        ComponentBufferData {
            color,
            position,
            size,
            border_radius,
            screen_size: [self.screen_size.width, self.screen_size.height],
            use_texture,
            blur_radius,
            opacity,
            _padding: [0.0; 3],
            border_color: self.border_color.value(),
            border_width: self.border_width,
            border_position: border_position_value,
            _padding2: [0.0; 2],
        }
    }

    pub fn get_sampler(&self) -> Option<&wgpu::Sampler> {
        self.metadata.iter().find_map(|m| match m {
            ComponentMetaData::Sampler(sampler) => Some(sampler),
            _ => None,
        })
    }

    pub fn update_bind_group(&mut self, new_bind_group: wgpu::BindGroup) {
        for metadata in &mut self.metadata {
            if let ComponentMetaData::BindGroup(_) = metadata {
                *metadata = ComponentMetaData::BindGroup(new_bind_group);
                return;
            }
        }
        // If we didn't find an existing bind group, add a new one
        self.metadata
            .push(ComponentMetaData::BindGroup(new_bind_group));
    }

    pub fn set_border_position(&mut self, position: BorderPosition) {
        self.border_position = position;
    }
}
