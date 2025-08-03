use std::time::Duration;

use crate::{
    app::AppEvent,
    ui::{
        animation::{
            AnimationConfig, AnimationDirection, AnimationRange, AnimationType, AnimationWhen,
            EasingFunction,
        },
        color::Color,
        ecs::{
            EntityId, NamedRef,
            builders::{
                EntityBuilder, EntityBuilderProps,
                background::{
                    BackgroundBuilder, BackgroundColorConfig, BackgroundGradientConfig,
                    FrostedGlassConfig,
                },
                button::ButtonBuilder,
                container::ContainerBuilder,
                image::{ImageBuilder, ScaleMode},
            },
            components::{InteractionComponent, ModalComponent},
        },
        layout::{
            AlignItems, Anchor, BorderRadius, ComponentOffset, Edges, FlexValue, JustifyContent,
            LayoutContext,
        },
    },
    utils::{deactivate_component_and_children, gather_all_children_with_types},
    wgpu_ctx::WgpuCtx,
};

pub struct ModalBuilder {
    common: EntityBuilderProps,
    backdrop_color: Option<BackgroundColorConfig>,
    backdrop_gradient: Option<BackgroundGradientConfig>,
    backdrop_frosted_glass: Option<FrostedGlassConfig>,
    backdrop_image: Option<String>,
    backdrop_image_scale_mode: Option<ScaleMode>,
    backdrop_animations: Vec<AnimationConfig>,
    close_button_anchor: Anchor,
    close_button_offset: ComponentOffset,
    close_button_size: (FlexValue, FlexValue),
    close_button_visible: bool,
    custom_close_button: Option<EntityId>,
    background_color: Option<BackgroundColorConfig>,
    background_gradient: Option<BackgroundGradientConfig>,
    background_frosted_glass: Option<FrostedGlassConfig>,
    background_image: Option<String>,
    background_image_scale_mode: Option<ScaleMode>,
    modal_width: FlexValue,
    modal_height: FlexValue,
    named_ref: NamedRef,
    dismiss_with_backdrop_click: bool,
}

impl EntityBuilder for ModalBuilder {
    fn common_props(&mut self) -> &mut EntityBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl ModalBuilder {
    pub fn new(named_ref: NamedRef) -> Self {
        Self {
            backdrop_animations: Vec::new(),
            backdrop_color: None,
            backdrop_gradient: None,
            backdrop_frosted_glass: None,
            backdrop_image: None,
            backdrop_image_scale_mode: None,
            common: EntityBuilderProps::default(),
            close_button_anchor: Anchor::TopRight,
            close_button_offset: ComponentOffset::new(-5.0, 5.0),
            close_button_size: (FlexValue::Fixed(30.0), FlexValue::Fixed(30.0)),
            close_button_visible: true,
            custom_close_button: None,
            background_color: None,
            background_gradient: None,
            background_frosted_glass: None,
            background_image: None,
            background_image_scale_mode: None,
            modal_width: FlexValue::Fixed(500.0),
            modal_height: FlexValue::Fixed(300.0),
            named_ref,
            dismiss_with_backdrop_click: true,
        }
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

    pub fn with_backdrop_color(mut self, color: BackgroundColorConfig) -> Self {
        if self.backdrop_gradient.is_some()
            || self.backdrop_frosted_glass.is_some()
            || self.backdrop_image.is_some()
        {
            log::warn!(
                "Backdrop color is set, but other backdrop properties are also defined. This may lead to unexpected behavior."
            );
        }
        self.backdrop_color = Some(color);
        self
    }

    pub fn with_backdrop_gradient(mut self, gradient: BackgroundGradientConfig) -> Self {
        if self.backdrop_color.is_some()
            || self.backdrop_frosted_glass.is_some()
            || self.backdrop_image.is_some()
        {
            log::warn!(
                "Backdrop gradient is set, but other backdrop properties are also defined. This may lead to unexpected behavior."
            );
        }
        self.backdrop_gradient = Some(gradient);
        self
    }

    pub fn with_backdrop_frosted_glass(mut self, config: FrostedGlassConfig) -> Self {
        if self.backdrop_color.is_some()
            || self.backdrop_gradient.is_some()
            || self.backdrop_image.is_some()
        {
            log::warn!(
                "Backdrop frosted glass is set, but other backdrop properties are also defined. This may lead to unexpected behavior."
            );
        }
        self.backdrop_frosted_glass = Some(config);
        self
    }

    pub fn with_backdrop_image<T: Into<String>>(mut self, image: T) -> Self {
        if self.backdrop_color.is_some()
            || self.backdrop_gradient.is_some()
            || self.backdrop_frosted_glass.is_some()
        {
            log::warn!(
                "Backdrop image is set, but other backdrop properties are also defined. This may lead to unexpected behavior."
            );
        }
        self.backdrop_image = Some(image.into());
        self
    }

    pub fn with_backdrop_image_scale_mode(mut self, mode: ScaleMode) -> Self {
        self.backdrop_image_scale_mode = Some(mode);
        self
    }

    pub fn with_backdrop_animation(mut self, animation: AnimationConfig) -> Self {
        self.backdrop_animations.push(animation);
        self
    }

    pub fn with_close_button_anchor(mut self, anchor: Anchor) -> Self {
        self.close_button_anchor = anchor;
        self
    }

    pub fn with_close_button_offset(mut self, offset: ComponentOffset) -> Self {
        self.close_button_offset = offset;
        self
    }

    pub fn with_close_button_size(mut self, width: FlexValue, height: FlexValue) -> Self {
        self.close_button_size = (width, height);
        self
    }

    pub fn with_close_button_visible(mut self, visible: bool) -> Self {
        self.close_button_visible = visible;
        self
    }

    pub fn with_custom_close_button(mut self, button: EntityId) -> Self {
        self.custom_close_button = Some(button);
        self
    }

    pub fn with_background_dismiss_disabled(mut self) -> Self {
        self.dismiss_with_backdrop_click = false;
        self
    }

    pub fn build(self, layout_context: &mut LayoutContext, wgpu_ctx: &mut WgpuCtx) -> EntityId {
        let modal_debug_name = self.common.debug_name.clone().expect(
            "Debug name is required for all components, tried to create a modal without it.",
        );

        let modal_parent_container_id = ContainerBuilder::new()
            .with_debug_name(format!("Modal Parent Container for {modal_debug_name}"))
            .with_align_items(AlignItems::Center)
            .with_justify_content(JustifyContent::Center)
            .with_hidden_overflow()
            .with_spawn_as_inactive()
            .with_absolute_position(Anchor::Center)
            .with_named_ref(self.named_ref)
            .build(
                &mut layout_context.world,
                &mut layout_context.z_index_manager,
            );

        // Analyze all animations to determine entry/exit properties
        let mut has_entry_animation = false;
        let mut has_exit_animation = false;

        let all_animations = self
            .common
            .animations
            .iter()
            .chain(self.backdrop_animations.iter());

        // Check backdrop animations
        for animation in all_animations {
            match animation.when {
                AnimationWhen::Entry => {
                    has_entry_animation = true;
                }
                AnimationWhen::Exit => {
                    has_exit_animation = true;
                }
                _ => {}
            }
        }

        let mut modal_component = ModalComponent {
            renderable_children: vec![],
            non_renderable_children: vec![],
            is_open: false,
            is_opening: false,
            is_closing: false,
            has_entry_animation,
            has_exit_animation,
        };

        let mut current_child_z_index = 0;

        let generic_backdrop_animations = self
            .backdrop_animations
            .iter()
            .filter(|a| a.animation_type.is_generic())
            .cloned()
            .collect::<Vec<_>>();

        let generic_modal_animations = self
            .common
            .animations
            .iter()
            .filter(|a| a.animation_type.is_generic())
            .cloned()
            .collect::<Vec<_>>();

        // Backdrop

        if let Some(backdrop_color_config) = self.backdrop_color {
            let mut backdrop_builder = BackgroundBuilder::with_color(backdrop_color_config)
                .with_debug_name(format!("Backdrop Color for {modal_debug_name}"))
                .with_fixed_position(Anchor::Center)
                .with_z_index(current_child_z_index);

            for animation in &generic_backdrop_animations {
                backdrop_builder = backdrop_builder.with_animation(animation.clone());
            }

            let background_color_animation = self
                .backdrop_animations
                .iter()
                .find(|a| matches!(a.animation_type, AnimationType::Color { .. }));

            if let Some(animation) = background_color_animation {
                backdrop_builder = backdrop_builder.with_animation(animation.clone());
            }

            if self.dismiss_with_backdrop_click {
                backdrop_builder =
                    backdrop_builder.with_click_event(AppEvent::CloseModal(self.named_ref));
            }

            let backdrop_id = backdrop_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            layout_context.add_child_to_parent(modal_parent_container_id, backdrop_id);
            modal_component.renderable_children.push(backdrop_id);
            current_child_z_index += 1;
        }

        if let Some(backdrop_gradient_config) = self.backdrop_gradient {
            let mut backdrop_builder = BackgroundBuilder::with_gradient(backdrop_gradient_config)
                .with_debug_name(format!("Backdrop Gradient for {modal_debug_name}"))
                .with_fixed_position(Anchor::Center)
                .with_z_index(current_child_z_index);

            for animation in &generic_backdrop_animations {
                backdrop_builder = backdrop_builder.with_animation(animation.clone());
            }

            if self.dismiss_with_backdrop_click {
                backdrop_builder =
                    backdrop_builder.with_click_event(AppEvent::CloseModal(self.named_ref));
            }

            let backdrop_id = backdrop_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            layout_context.add_child_to_parent(modal_parent_container_id, backdrop_id);
            modal_component.renderable_children.push(backdrop_id);
            current_child_z_index += 1;
        }

        if let Some(backdrop_image) = self.backdrop_image {
            let mut backdrop_builder = ImageBuilder::new(&backdrop_image)
                .with_debug_name(format!("Backdrop Image for {modal_debug_name}"))
                .with_fixed_position(Anchor::Center)
                .with_z_index(current_child_z_index);

            if let Some(scale_mode) = self.backdrop_image_scale_mode {
                backdrop_builder = backdrop_builder.with_scale_mode(scale_mode);
            }

            for animation in &generic_backdrop_animations {
                backdrop_builder = backdrop_builder.with_animation(animation.clone());
            }

            if self.dismiss_with_backdrop_click {
                backdrop_builder =
                    backdrop_builder.with_click_event(AppEvent::CloseModal(self.named_ref));
            }

            let backdrop_id = backdrop_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            layout_context.add_child_to_parent(modal_parent_container_id, backdrop_id);
            modal_component.renderable_children.push(backdrop_id);
            current_child_z_index += 1;
        }

        if let Some(backdrop_frosted_glass_config) = self.backdrop_frosted_glass {
            let mut backdrop_builder =
                BackgroundBuilder::with_frosted_glass(backdrop_frosted_glass_config)
                    .with_debug_name(format!("Backdrop Frosted Glass for {modal_debug_name}"))
                    .with_fixed_position(Anchor::Center)
                    .with_z_index(current_child_z_index);

            for animation in &generic_backdrop_animations {
                backdrop_builder = backdrop_builder.with_animation(animation.clone());
            }

            let frosted_glass_animation = self
                .backdrop_animations
                .iter()
                .find(|a| matches!(a.animation_type, AnimationType::FrostedGlassTint { .. }));

            if let Some(animation) = frosted_glass_animation {
                backdrop_builder = backdrop_builder.with_animation(animation.clone());
            }

            if self.dismiss_with_backdrop_click {
                backdrop_builder =
                    backdrop_builder.with_click_event(AppEvent::CloseModal(self.named_ref));
            }

            let backdrop_id = backdrop_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            layout_context.add_child_to_parent(modal_parent_container_id, backdrop_id);
            modal_component.renderable_children.push(backdrop_id);
            current_child_z_index += 1;
        }

        // Modal Background

        let mut modal_container_builder = ContainerBuilder::new()
            .with_debug_name(format!("Modal Container for {modal_debug_name}"))
            .with_z_index(current_child_z_index);

        if self.common.position.is_none() {
            modal_container_builder = modal_container_builder.with_fixed_position(Anchor::Center);
        }
        if self.common.width.is_none() && self.common.height.is_none() {
            modal_container_builder =
                modal_container_builder.with_size(self.modal_width, self.modal_height);
        }

        let modal_container_id = modal_container_builder.build(
            &mut layout_context.world,
            &mut layout_context.z_index_manager,
        );

        layout_context.add_child_to_parent(modal_parent_container_id, modal_container_id);
        modal_component
            .non_renderable_children
            .push(modal_container_id);

        if let Some(modal_background_color_config) = self.background_color {
            let mut modal_background_builder =
                BackgroundBuilder::with_color(modal_background_color_config)
                    .with_external_common_props(self.common.clone())
                    .with_debug_name(format!("Modal Background Color for {modal_debug_name}"))
                    .with_fixed_position(Anchor::Center)
                    .with_z_index(current_child_z_index);

            let background_color_animation = self
                .common
                .animations
                .iter()
                .find(|a| matches!(a.animation_type, AnimationType::Color { .. }));

            if let Some(animation) = background_color_animation {
                modal_background_builder =
                    modal_background_builder.with_animation(animation.clone());
            }

            for animation in &generic_modal_animations {
                modal_background_builder =
                    modal_background_builder.with_animation(animation.clone());
            }

            let modal_background_id = modal_background_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            layout_context.add_child_to_parent(modal_container_id, modal_background_id);
            modal_component
                .renderable_children
                .push(modal_background_id);
            current_child_z_index += 1;
        }

        if let Some(modal_background_gradient_config) = self.background_gradient {
            let mut modal_background_builder =
                BackgroundBuilder::with_gradient(modal_background_gradient_config)
                    .with_external_common_props(self.common.clone())
                    .with_debug_name(format!("Modal Background Gradient for {modal_debug_name}"))
                    .with_fixed_position(Anchor::Center)
                    .with_z_index(current_child_z_index);

            for animation in &generic_modal_animations {
                modal_background_builder =
                    modal_background_builder.with_animation(animation.clone());
            }

            let modal_background_id = modal_background_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            layout_context.add_child_to_parent(modal_container_id, modal_background_id);
            modal_component
                .renderable_children
                .push(modal_background_id);
            current_child_z_index += 1;
        }

        if let Some(modal_background_frosted_glass_config) = self.background_frosted_glass {
            let mut modal_background_builder =
                BackgroundBuilder::with_frosted_glass(modal_background_frosted_glass_config)
                    .with_external_common_props(self.common.clone())
                    .with_debug_name(format!(
                        "Modal Background Frosted Glass for {modal_debug_name}"
                    ))
                    .with_fixed_position(Anchor::Center)
                    .with_z_index(current_child_z_index);

            let frosted_glass_animation = self
                .common
                .animations
                .iter()
                .find(|a| matches!(a.animation_type, AnimationType::FrostedGlassTint { .. }));

            if let Some(animation) = frosted_glass_animation {
                modal_background_builder =
                    modal_background_builder.with_animation(animation.clone());
            }

            for animation in &generic_modal_animations {
                modal_background_builder =
                    modal_background_builder.with_animation(animation.clone());
            }

            let modal_background_id = modal_background_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            layout_context.add_child_to_parent(modal_container_id, modal_background_id);
            modal_component
                .renderable_children
                .push(modal_background_id);
            current_child_z_index += 1;
        }

        if let Some(modal_background_image) = self.background_image {
            let mut modal_background_builder = ImageBuilder::new(&modal_background_image)
                .with_external_common_props(self.common.clone())
                .with_debug_name(format!("Modal Background Image for {modal_debug_name}"))
                .with_fixed_position(Anchor::Center)
                .with_z_index(current_child_z_index);

            if let Some(scale_mode) = self.background_image_scale_mode {
                modal_background_builder = modal_background_builder.with_scale_mode(scale_mode);
            }

            for animation in &generic_modal_animations {
                modal_background_builder =
                    modal_background_builder.with_animation(animation.clone());
            }

            let modal_background_id = modal_background_builder.build(
                &mut layout_context.world,
                wgpu_ctx,
                &mut layout_context.z_index_manager,
            );

            layout_context.add_child_to_parent(modal_container_id, modal_background_id);
            modal_component
                .renderable_children
                .push(modal_background_id);
            current_child_z_index += 1;
        }

        // Close Button
        let close_button_id = if let Some(custom_close_button) = self.custom_close_button {
            let custom_close_button_interaction_comp = layout_context
                .world
                .components
                .get_component::<InteractionComponent>(custom_close_button)
                .expect(
                    "Custom close button must have an InteractionComponent to handle close events.",
                );

            if custom_close_button_interaction_comp.is_active {
                deactivate_component_and_children(&mut layout_context.world, custom_close_button);
            }

            custom_close_button
        } else {
            let mut close_button_builder = ButtonBuilder::new()
                .with_debug_name(format!("Close Button for {modal_debug_name}"))
                .with_fixed_position(self.close_button_anchor)
                .with_offset(self.close_button_offset)
                .with_content_padding(Edges::all(5.0))
                .with_size(self.close_button_size.0, self.close_button_size.1)
                .with_border_radius(BorderRadius::all(4.0))
                .with_background_color(BackgroundColorConfig {
                    color: Color::Transparent,
                })
                .with_foreground_image("close.png")
                .with_animation(AnimationConfig {
                    duration: Duration::from_millis(200),
                    easing: EasingFunction::EaseOutExpo,
                    direction: AnimationDirection::Alternate,
                    animation_type: AnimationType::Color {
                        range: AnimationRange::new(Color::Transparent, Color::DarkGray),
                    },
                    when: AnimationWhen::Hover,
                })
                .with_click_event(AppEvent::CloseModal(self.named_ref))
                .with_z_index(current_child_z_index)
                .with_spawn_as_inactive();

            // if we have any entry or exit animations, apply them
            for animation in self.common.animations {
                if matches!(animation.when, AnimationWhen::Entry | AnimationWhen::Exit) {
                    close_button_builder = close_button_builder.with_animation(animation.clone());
                }
            }

            if !self.close_button_visible {
                close_button_builder = close_button_builder.with_spawn_as_inactive();
            }

            close_button_builder.build(layout_context, wgpu_ctx)
        };

        layout_context.add_child_to_parent(modal_container_id, close_button_id);
        modal_component
            .non_renderable_children
            .push(close_button_id);

        // Go through all the children of the close button and add them to the modal component
        let close_button_children =
            gather_all_children_with_types(&layout_context.world, close_button_id);

        for (child_id, component_type) in close_button_children {
            if component_type.is_renderable() {
                modal_component.renderable_children.push(child_id);
            } else {
                modal_component.non_renderable_children.push(child_id);
            }
        }
        layout_context
            .world
            .add_component(modal_parent_container_id, modal_component);

        modal_parent_container_id
    }
}
