use crate::{
    ui::{
        animation::AnimationWhen,
        color::Color,
        component::{
            BackgroundColorConfig, BackgroundGradientConfig, Component, ComponentConfig,
            ComponentType, FrostedGlassConfig, GradientColorStop, GradientType, ImageConfig,
            TextConfig,
        },
        components::{
            component_builder::{CommonBuilderProps, ComponentBuilder},
            container::FlexContainerBuilder,
            label::LabelBuilder,
        },
        layout::{Anchor, Edges, Position},
    },
    wgpu_ctx::WgpuCtx,
};
use log::error;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ButtonSubComponent {
    Color(Color),
    Gradient {
        color_stops: Vec<GradientColorStop>,
        gradient_type: GradientType,
        angle: f32,
        center: Option<(f32, f32)>,
        radius: Option<f32>,
    },
    FrostedGlass {
        tint_color: Color,
        blur_radius: f32,
        opacity: f32,
        tint_intensity: f32,
    },
    Image(ImageConfig),
    Text(TextConfig),
}

#[derive(Debug, Clone, Default)]
pub struct ButtonConfig {
    pub sub_components: Vec<ButtonSubComponent>,
    pub content_padding: Option<Edges>,
}

pub struct ButtonBuilder {
    common: CommonBuilderProps,
    config: ButtonConfig,
}

impl ComponentBuilder for ButtonBuilder {
    fn common_props(&mut self) -> &mut CommonBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl ButtonBuilder {
    pub fn new() -> Self {
        Self {
            common: CommonBuilderProps::default(),
            config: ButtonConfig::default(),
        }
    }

    pub fn with_sub_component(mut self, sub_component: ButtonSubComponent) -> Self {
        self.config.sub_components.push(sub_component);
        self
    }

    pub fn with_content_padding(mut self, padding: Edges) -> Self {
        self.config.content_padding = Some(padding);
        self
    }

    pub fn build(self, wgpu_ctx: &mut WgpuCtx) -> Component {
        let mut container_builder = FlexContainerBuilder::new();
        let common_props = self.common.clone();
        let config = self.config.clone();

        // Use fully qualified path to call the trait method
        <FlexContainerBuilder as ComponentBuilder>::apply_common_properties(
            &mut container_builder,
            &common_props,
        );

        let mut container = container_builder.build(wgpu_ctx);
        container.flag_children_extraction();

        let border_radius = common_props.border_radius;
        let animation_config = common_props.animation;

        // Create a content container if content padding is specified
        let mut content_container = if self.config.content_padding.is_some() {
            Some(
                FlexContainerBuilder::new()
                    .with_padding(self.config.content_padding.unwrap())
                    .with_debug_name("Button Content Container")
                    .with_position(Position::Fixed(Anchor::Center))
                    .with_z_index(2)
                    .build(wgpu_ctx),
            )
        } else {
            None
        };

        // Create background if specified
        for sub_component in config.sub_components {
            match sub_component {
                ButtonSubComponent::Color(color) => {
                    let mut bg = Component::new(Uuid::new_v4(), ComponentType::BackgroundColor);
                    bg.transform.position_type = Position::Fixed(Anchor::Center);
                    bg.set_debug_name("Button Background Color");
                    bg.set_z_index(-2);

                    if let Some(radius) = border_radius {
                        bg.transform.border_radius = radius;
                    }

                    bg.configure(
                        ComponentConfig::BackgroundColor(BackgroundColorConfig { color }),
                        wgpu_ctx,
                    );

                    if let Some(anim_config) = &animation_config {
                        bg.set_animation(anim_config.clone(), wgpu_ctx);
                    }

                    container.add_child_to_front(bg);
                }
                ButtonSubComponent::Gradient {
                    color_stops,
                    gradient_type,
                    angle,
                    center,
                    radius,
                } => {
                    let mut bg = self.configure_gradient_background(
                        wgpu_ctx,
                        color_stops,
                        gradient_type,
                        angle,
                        center,
                        radius,
                    );
                    if let Some(animation) = &animation_config {
                        bg.set_animation(animation.clone(), wgpu_ctx);
                    }
                    container.add_child(bg);
                }
                ButtonSubComponent::FrostedGlass {
                    tint_color,
                    blur_radius,
                    opacity,
                    tint_intensity,
                } => {
                    let mut bg = self.configure_frosted_glass(
                        wgpu_ctx,
                        tint_color,
                        blur_radius,
                        opacity,
                        tint_intensity,
                    );
                    if let Some(animation) = &animation_config {
                        bg.set_animation(animation.clone(), wgpu_ctx);
                    }
                    container.add_child(bg);
                }
                ButtonSubComponent::Image(img_config) => {
                    let img = self.configure_image_background(wgpu_ctx, img_config);
                    if let Some(ref mut content) = content_container {
                        content.add_child(img);
                    } else {
                        container.add_child(img);
                    }
                }
                ButtonSubComponent::Text(text_config) => {
                    let mut label = LabelBuilder::new(text_config.text)
                        .with_debug_name("Button Label")
                        .with_fixed_position(Anchor::Center)
                        .with_color(text_config.color)
                        .with_font_size(text_config.font_size)
                        .with_line_height(text_config.line_height)
                        .with_z_index(2) // Match image z-index
                        .build(wgpu_ctx);

                    if container.fit_to_size {
                        label.set_fit_to_size(true);
                    }

                    if let Some(ref mut content) = content_container {
                        content.add_child(label);
                    } else {
                        container.add_child(label);
                    }
                }
            }
        }

        // Add the content container if it exists and has children
        if let Some(content) = content_container {
            if content.has_children() {
                container.add_child(content);
            }
        }

        // Make the container itself hoverable if it has hover animations
        if let Some(anim_config) = animation_config {
            if matches!(anim_config.when, AnimationWhen::Hover) {
                container.set_z_index(-1);

                if let Some(radius) = border_radius {
                    container.transform.border_radius = radius;
                }

                container.set_animation(anim_config, wgpu_ctx);
            }
        }

        if (container.get_click_event().is_some() || container.get_drag_event().is_some())
            && container.get_event_sender().is_none()
        {
            let identifier = if let Some(debug_name) = container.debug_name.as_ref() {
                format!("{} ({})", debug_name, container.id)
            } else {
                container.id.to_string()
            };
            error!(
                "Button {} has click/drag event but no event sender, this will cause the event to not be propagated",
                identifier
            );
        }

        container
    }

    fn configure_frosted_glass(
        &self,
        wgpu_ctx: &mut WgpuCtx<'_>,
        tint_color: Color,
        blur_radius: f32,
        opacity: f32,
        tint_intensity: f32,
    ) -> Component {
        let bg_id = Uuid::new_v4();
        let mut bg = Component::new(bg_id, ComponentType::FrostedGlass);
        bg.transform.position_type = Position::Fixed(Anchor::Center);
        bg.set_debug_name("Button Frosted Glass Background");
        bg.set_z_index(0);
        if let Some(border_radius) = self.common.border_radius {
            bg.set_border_radius(border_radius);
        }
        if let Some(border_width) = self.common.border_width {
            bg.border_width = border_width;
        }
        if let Some(border_color) = self.common.border_color {
            bg.border_color = border_color;
        }
        if let Some(border_position) = self.common.border_position {
            bg.set_border_position(border_position);
        }
        bg.configure(
            ComponentConfig::FrostedGlass(FrostedGlassConfig {
                tint_color,
                blur_radius,
                opacity,
                tint_intensity,
            }),
            wgpu_ctx,
        );
        bg
    }

    fn configure_image_background(
        &self,
        wgpu_ctx: &mut WgpuCtx<'_>,
        img_config: ImageConfig,
    ) -> Component {
        let bg_id = Uuid::new_v4();
        let mut bg = Component::new(bg_id, ComponentType::Image);
        bg.transform.position_type = Position::Fixed(Anchor::Center);
        bg.set_debug_name("Button Image");
        bg.set_z_index(1); // Ensure image is in front
        if let Some(border_radius) = self.common.border_radius {
            bg.set_border_radius(border_radius);
        }
        if let Some(border_width) = self.common.border_width {
            bg.border_width = border_width;
        }
        if let Some(border_color) = self.common.border_color {
            bg.border_color = border_color;
        }
        if let Some(border_position) = self.common.border_position {
            bg.set_border_position(border_position);
        }
        bg.configure(ComponentConfig::Image(img_config), wgpu_ctx);
        bg
    }

    fn configure_gradient_background(
        &self,
        wgpu_ctx: &mut WgpuCtx<'_>,
        color_stops: Vec<GradientColorStop>,
        gradient_type: GradientType,
        angle: f32,
        center: Option<(f32, f32)>,
        radius: Option<f32>,
    ) -> Component {
        let bg_id = Uuid::new_v4();
        let mut bg = Component::new(bg_id, ComponentType::BackgroundGradient);
        bg.transform.position_type = Position::Fixed(Anchor::Center);
        bg.set_debug_name("Button Gradient Background");
        bg.set_z_index(0);
        if let Some(border_radius) = self.common.border_radius {
            bg.set_border_radius(border_radius);
        }
        if let Some(border_width) = self.common.border_width {
            bg.border_width = border_width;
        }
        if let Some(border_color) = self.common.border_color {
            bg.border_color = border_color;
        }
        if let Some(border_position) = self.common.border_position {
            bg.set_border_position(border_position);
        }
        bg.configure(
            ComponentConfig::BackgroundGradient(BackgroundGradientConfig {
                color_stops,
                gradient_type,
                angle,
                center,
                radius,
            }),
            wgpu_ctx,
        );
        bg
    }
}
