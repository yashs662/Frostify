use crate::{
    ui::{
        animation::{AnimationConfig, AnimationType},
        color::Color,
        ecs::{
            EntityId,
            builders::{
                EntityBuilder, EntityBuilderProps,
                background::{
                    BackgroundBuilder, BackgroundColorConfig, BackgroundGradientConfig,
                    FrostedGlassConfig,
                },
                container::ContainerBuilder,
                image::{ImageBuilder, ScaleMode},
                text::{TextBuilder, TextConfig},
            },
        },
        layout::{AlignItems, Anchor, Edges, JustifyContent, LayoutContext},
    },
    wgpu_ctx::WgpuCtx,
};

pub struct ButtonBuilder {
    common: EntityBuilderProps,
    background_color: Option<BackgroundColorConfig>,
    background_gradient: Option<BackgroundGradientConfig>,
    background_frosted_glass: Option<FrostedGlassConfig>,
    background_image: Option<String>,
    background_image_scale_mode: Option<ScaleMode>,
    foreground_image: Option<String>,
    foreground_image_scale_mode: Option<ScaleMode>,
    foreground_text: Option<TextConfig>,
    animations: Vec<AnimationConfig>,
    content_padding: Option<Edges>,
}

impl EntityBuilder for ButtonBuilder {
    fn common_props(&mut self) -> &mut EntityBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl ButtonBuilder {
    pub fn new() -> Self {
        Self {
            common: EntityBuilderProps::default(),
            background_color: None,
            background_gradient: None,
            background_frosted_glass: None,
            background_image: None,
            background_image_scale_mode: None,
            foreground_image: None,
            foreground_image_scale_mode: None,
            foreground_text: None,
            animations: Vec::new(),
            content_padding: None,
        }
    }

    pub fn with_text<S: Into<String>>(mut self, text: S) -> Self {
        let text = text.into();
        if let Some(text_config) = &mut self.foreground_text {
            text_config.text = text;
        } else {
            let default_text_config = TextConfig {
                text,
                ..Default::default()
            };
            self.foreground_text = Some(default_text_config);
        }
        self
    }

    pub fn with_text_color(mut self, color: Color) -> Self {
        if let Some(text_config) = &mut self.foreground_text {
            text_config.color = color;
        } else {
            let default_text_config = TextConfig {
                color,
                ..Default::default()
            };
            self.foreground_text = Some(default_text_config);
        }
        self
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        if let Some(text_config) = &mut self.foreground_text {
            text_config.font_size = font_size;
        } else {
            let default_text_config = TextConfig {
                font_size,
                ..Default::default()
            };
            self.foreground_text = Some(default_text_config);
        }
        self
    }

    pub fn with_line_height(mut self, line_height: f32) -> Self {
        if let Some(text_config) = &mut self.foreground_text {
            text_config.line_height_multiplier = line_height;
        } else {
            let default_text_config = TextConfig {
                line_height_multiplier: line_height,
                ..Default::default()
            };
            self.foreground_text = Some(default_text_config);
        }
        self
    }

    pub fn with_content_padding(mut self, padding: Edges) -> Self {
        self.content_padding = Some(padding);
        self
    }

    pub fn with_background_color(mut self, color: BackgroundColorConfig) -> Self {
        if self.background_gradient.is_some()
            || self.background_frosted_glass.is_some()
            || self.background_image.is_some()
        {
            log::warn!(
                "Background color is set, but other background properties are also defined. This may lead to unexpected behavior."
            );
        }
        self.background_color = Some(color);
        self
    }

    pub fn with_background_gradient(mut self, gradient: BackgroundGradientConfig) -> Self {
        if self.background_color.is_some()
            || self.background_frosted_glass.is_some()
            || self.background_image.is_some()
        {
            log::warn!(
                "Background gradient is set, but other background properties are also defined. This may lead to unexpected behavior."
            );
        }
        self.background_gradient = Some(gradient);
        self
    }

    pub fn with_background_frosted_glass(mut self, config: FrostedGlassConfig) -> Self {
        if self.background_color.is_some()
            || self.background_gradient.is_some()
            || self.background_image.is_some()
        {
            log::warn!(
                "Background frosted glass is set, but other background properties are also defined. This may lead to unexpected behavior."
            );
        }
        self.background_frosted_glass = Some(config);
        self
    }

    pub fn with_background_image<T: Into<String>>(mut self, image: T) -> Self {
        if self.background_color.is_some()
            || self.background_gradient.is_some()
            || self.background_frosted_glass.is_some()
        {
            log::warn!(
                "Background image is set, but other background properties are also defined. This may lead to unexpected behavior."
            );
        }
        self.background_image = Some(image.into());
        self
    }

    pub fn with_background_image_scale_mode(mut self, mode: ScaleMode) -> Self {
        self.background_image_scale_mode = Some(mode);
        self
    }

    pub fn with_foreground_image<T: Into<String>>(mut self, image: T) -> Self {
        self.foreground_image = Some(image.into());
        self
    }

    pub fn with_foreground_image_scale_mode(mut self, mode: ScaleMode) -> Self {
        self.foreground_image_scale_mode = Some(mode);
        self
    }

    pub fn with_animation(mut self, animation: AnimationConfig) -> Self {
        self.animations.push(animation);
        self
    }

    pub fn build(self, layout_context: &mut LayoutContext, wgpu_ctx: &mut WgpuCtx) -> EntityId {
        let button_debug_name = self.common.debug_name.clone().expect(
            "Debug name is required for all components, tried to create a button without it.",
        );
        let button_container_id = ContainerBuilder::new()
            .with_external_common_props(self.common.clone())
            .with_debug_name(format!("Button Container for {}", button_debug_name))
            .with_align_items(AlignItems::Center)
            .with_justify_content(JustifyContent::Center)
            .with_hidden_overflow()
            .build(
                &mut layout_context.world,
                &mut layout_context.z_index_manager,
            );

        let mut current_child_z_index = 1;
        let generic_animations = self
            .animations
            .iter()
            .filter(|a| a.animation_type.is_generic())
            .cloned()
            .collect::<Vec<_>>();

        let content_container = if let Some(padding) = self.content_padding {
            let content_container_id = ContainerBuilder::new()
                .with_debug_name(format!(
                    "Button Content Container for {}",
                    button_debug_name
                ))
                .with_fixed_position(Anchor::Center)
                .with_align_items(AlignItems::Center)
                .with_justify_content(JustifyContent::Center)
                .with_padding(padding)
                .with_z_index(10)
                .build(
                    &mut layout_context.world,
                    &mut layout_context.z_index_manager,
                );

            layout_context.add_child_to_parent(button_container_id, content_container_id);

            Some(content_container_id)
        } else {
            None
        };

        if let Some(background_color_config) = self.background_color {
            let background_color_animation = self
                .animations
                .iter()
                .find(|a| matches!(a.animation_type, AnimationType::Color { .. }));

            let mut background_color_builder =
                BackgroundBuilder::with_color(background_color_config)
                    .with_fixed_position(Anchor::Center)
                    .with_debug_name(format!("Button Background Color for {}", button_debug_name))
                    .with_z_index(current_child_z_index);

            if let Some(animation) = background_color_animation {
                background_color_builder =
                    background_color_builder.with_animation(animation.clone());
            }
            for animation in &generic_animations {
                background_color_builder =
                    background_color_builder.with_animation(animation.clone());
            }

            let background_color_id = background_color_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            layout_context.add_child_to_parent(button_container_id, background_color_id);
            current_child_z_index += 1;
        }

        if let Some(background_gradient_config) = self.background_gradient {
            let mut background_gradient_builder =
                BackgroundBuilder::with_gradient(background_gradient_config)
                    .with_fixed_position(Anchor::Center)
                    .with_debug_name(format!(
                        "Button Background Gradient for {}",
                        button_debug_name
                    ))
                    .with_z_index(current_child_z_index);

            for animation in &generic_animations {
                background_gradient_builder =
                    background_gradient_builder.with_animation(animation.clone());
            }

            let background_gradient_id = background_gradient_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            layout_context.add_child_to_parent(button_container_id, background_gradient_id);
            current_child_z_index += 1;
        }

        if let Some(background_image) = self.background_image {
            let mut background_image_builder = ImageBuilder::new(&background_image)
                .with_fixed_position(Anchor::Center)
                .with_scale_mode(self.background_image_scale_mode.unwrap_or_default())
                .with_debug_name(format!("Button Background Image for {}", button_debug_name))
                .with_z_index(current_child_z_index);

            for animation in &generic_animations {
                background_image_builder =
                    background_image_builder.with_animation(animation.clone());
            }

            let background_image_id = background_image_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            layout_context.add_child_to_parent(button_container_id, background_image_id);
            current_child_z_index += 1;
        }

        if let Some(frosted_glass_config) = self.background_frosted_glass {
            let mut background_frosted_glass_builder =
                BackgroundBuilder::with_frosted_glass(frosted_glass_config)
                    .with_debug_name(format!(
                        "Button Background Frosted Glass for {}",
                        button_debug_name
                    ))
                    .with_fixed_position(Anchor::Center)
                    .with_z_index(current_child_z_index);

            for animation in &generic_animations {
                background_frosted_glass_builder =
                    background_frosted_glass_builder.with_animation(animation.clone());
            }

            let background_frosted_glass_id = background_frosted_glass_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            layout_context.add_child_to_parent(button_container_id, background_frosted_glass_id);
            current_child_z_index += 1;
        }

        if let Some(foreground_image) = self.foreground_image {
            let mut foreground_image_builder = ImageBuilder::new(&foreground_image)
                .with_debug_name(format!("Button Foreground Image for {}", button_debug_name))
                .with_fixed_position(Anchor::Center)
                .with_scale_mode(self.foreground_image_scale_mode.unwrap_or_default())
                .with_z_index(current_child_z_index);

            for animation in &generic_animations {
                foreground_image_builder =
                    foreground_image_builder.with_animation(animation.clone());
            }

            let foreground_image_id = foreground_image_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            if let Some(content_container_id) = content_container {
                layout_context.add_child_to_parent(content_container_id, foreground_image_id);
            } else {
                layout_context.add_child_to_parent(button_container_id, foreground_image_id);
            }
            current_child_z_index += 1;
        }

        if let Some(text_config) = &self.foreground_text {
            let mut text_builder = TextBuilder::new()
                .with_debug_name(format!("Button Foreground Text for {}", button_debug_name))
                .with_fixed_position(Anchor::Center)
                .with_text(text_config.text.clone())
                .with_font_size(text_config.font_size)
                .with_line_height(text_config.line_height_multiplier)
                .with_color(text_config.color)
                .set_fit_to_size()
                .with_z_index(current_child_z_index);

            for animation in &generic_animations {
                text_builder = text_builder.with_animation(animation.clone());
            }

            let text_id = text_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            if let Some(content_container_id) = content_container {
                layout_context.add_child_to_parent(content_container_id, text_id);
            } else {
                layout_context.add_child_to_parent(button_container_id, text_id);
            }
        }

        button_container_id
    }
}
