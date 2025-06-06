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
        layout::{
            AlignItems, Anchor, ComponentOffset, Edges, JustifyContent, LayoutContext, Overflow,
        },
    },
    wgpu_ctx::WgpuCtx,
};

pub struct ModalBuilder {
    common: EntityBuilderProps,
    animations: Vec<AnimationConfig>,
    background_color: Option<BackgroundColorConfig>,
    background_gradient: Option<BackgroundGradientConfig>,
    background_frosted_glass: Option<FrostedGlassConfig>,
    background_image: Option<String>,
    background_image_scale_mode: Option<ScaleMode>,
    close_button_anchor: Anchor,
    close_button_offset: ComponentOffset,
    close_button_visible: bool,
    custom_close_button: Option<EntityId>,
}

impl EntityBuilder for ModalBuilder {
    fn common_props(&mut self) -> &mut EntityBuilderProps {
        &mut self.common
    }
}

impl ModalBuilder {
    pub fn new() -> Self {
        Self {
            common: EntityBuilderProps::default(),
            animations: Vec::new(),
            background_color: None,
            background_gradient: None,
            background_frosted_glass: None,
            background_image: None,
            background_image_scale_mode: None,
            close_button_anchor: Anchor::TopRight,
            close_button_offset: ComponentOffset::new(-5.0, -5.0),
            close_button_visible: true,
            custom_close_button: None,
        }
    }

    pub fn with_background_color(mut self, color: BackgroundColorConfig) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn with_background_gradient(mut self, gradient: BackgroundGradientConfig) -> Self {
        self.background_gradient = Some(gradient);
        self
    }

    pub fn with_background_frosted_glass(mut self, config: FrostedGlassConfig) -> Self {
        self.background_frosted_glass = Some(config);
        self
    }

    pub fn with_background_image<T: Into<String>>(mut self, image: T) -> Self {
        self.background_image = Some(image.into());
        self
    }

    pub fn with_background_image_scale_mode(mut self, mode: ScaleMode) -> Self {
        self.background_image_scale_mode = Some(mode);
        self
    }

    pub fn with_animation(mut self, animation: AnimationConfig) -> Self {
        self.animations.push(animation);
        self
    }

    pub fn build(self, layout_context: &mut LayoutContext, wgpu_ctx: &mut WgpuCtx) -> EntityId {
        let modal_debug_name = self.common.debug_name.clone().expect(
            "Debug name is required for all components, tried to create a modal without it.",
        );

        let modal_main_container_id = ContainerBuilder::new()
            .with_external_common_props(self.common.clone())
            .with_debug_name(format!("Modal Main Container for {}", modal_debug_name))
            .with_align_items(AlignItems::Center)
            .with_justify_content(JustifyContent::Center)
            .with_overflow(Overflow::Hidden)
            .build(
                &mut layout_context.world,
                &mut layout_context.z_index_manager,
            );

        modal_main_container_id
    }
}
