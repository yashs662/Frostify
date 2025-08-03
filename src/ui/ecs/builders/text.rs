use wgpu::util::DeviceExt;

use crate::{
    ui::{
        color::Color,
        ecs::{
            ComponentType, EntityId, World,
            builders::{EntityBuilder, EntityBuilderProps, add_common_components},
            components::{LayoutComponent, RenderDataComponent, TextComponent},
            resources::TextRenderingResource,
        },
        layout::Layout,
        z_index_manager::ZIndexManager,
    },
    utils::create_entity_buffer_data,
    wgpu_ctx::WgpuCtx,
};

pub struct TextConfig {
    pub text: String,
    pub font_size: f32,
    pub line_height_multiplier: f32,
    pub color: Color,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_size: 16.0,
            line_height_multiplier: 1.5,
            color: Color::Black,
        }
    }
}

pub struct TextBuilder {
    common: EntityBuilderProps,
    config: TextConfig,
}

impl EntityBuilder for TextBuilder {
    fn common_props(&mut self) -> &mut EntityBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl TextBuilder {
    pub fn new() -> Self {
        Self {
            common: EntityBuilderProps::default(),
            config: TextConfig::default(),
        }
    }

    pub fn with_text<S: Into<String>>(mut self, text: S) -> Self {
        self.config.text = text.into();
        self
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.config.font_size = font_size;
        self
    }

    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.config.line_height_multiplier = line_height;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.config.color = color;
        self
    }

    pub fn build(
        self,
        world: &mut World,
        wgpu_ctx: &mut WgpuCtx,
        z_index_manager: &mut ZIndexManager,
    ) -> EntityId {
        let component_type = ComponentType::Text;

        let entity_id = world.create_entity(
            self.common
                .debug_name.clone()
                .expect("Debug name is required for all components, tried to create a text component without it."),
            component_type,
        );

        add_common_components(world, z_index_manager, entity_id, &self.common);

        // Add layout component
        let mut layout = Layout::new();
        if let Some(margin) = self.common.margin {
            layout.margin = margin;
        }
        if let Some(padding) = self.common.padding {
            layout.padding = padding;
        }
        world.add_component(entity_id, LayoutComponent { layout });

        // Add text component
        let mut text_component = TextComponent::new(
            self.config.text.clone(),
            self.config.font_size,
            self.config.line_height_multiplier,
            self.config.color,
        );

        // Initialize the text component's rendering state
        if let Some(text_resource) = world.resources.get_resource_mut::<TextRenderingResource>() {
            text_component.initialize_rendering(&mut text_resource.font_system);
        }

        world.add_component(entity_id, text_component);

        let render_data_buffer =
            wgpu_ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(format!("{entity_id} Render Data Buffer").as_str()),
                    contents: bytemuck::cast_slice(&[create_entity_buffer_data(
                        &world.components,
                        entity_id,
                    )]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let sampler = wgpu_ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        world.add_component(
            entity_id,
            RenderDataComponent {
                render_data_buffer,
                sampler,
                bind_group: None,    // Will be created later during layout sync
                vertex_buffer: None, // Will be generated during layout sync
                index_buffer: None,  // Will be generated during layout sync
            },
        );

        entity_id
    }
}
