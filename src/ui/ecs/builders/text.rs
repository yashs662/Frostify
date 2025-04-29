use crate::{
    ui::{
        color::Color,
        ecs::{
            ComponentType, EntityId, World,
            builders::{EntityBuilder, EntityBuilderProps},
            components::{LayoutComponent, TextComponent},
        },
        layout::{Bounds, Layout},
        z_index_manager::ZIndexManager,
    },
    wgpu_ctx::WgpuCtx,
};

use super::add_common_components;

pub struct TextConfig {
    pub text: String,
    pub font_size: f32,
    pub line_height_multiplier: f32,
    pub color: Color,
    pub fit_to_size: bool,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_size: 16.0,
            line_height_multiplier: 1.5,
            color: Color::Black,
            fit_to_size: false,
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

    pub fn with_text(mut self, text: String) -> Self {
        self.config.text = text;
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

    pub fn set_fit_to_size(mut self) -> Self {
        self.config.fit_to_size = true;
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

        add_common_components(
            world,
            z_index_manager,
            entity_id,
            &self.common,
            component_type,
        );

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
        // TODO: Make Fit to size work again
        world.add_component(
            entity_id,
            TextComponent {
                text: self.config.text.clone(),
                font_size: self.config.font_size,
                line_height_multiplier: self.config.line_height_multiplier,
                color: self.config.color,
                fit_to_size: self.config.fit_to_size,
            },
        );

        // Configure
        wgpu_ctx.text_handler.register_text(
            entity_id,
            self.config.text.clone(),
            self.config.font_size,
            self.config.line_height_multiplier,
            Bounds::default(),
            self.config.color,
        );

        entity_id
    }
}
