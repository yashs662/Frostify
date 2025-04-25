use crate::{
    app::AppEvent,
    ui::{
        animation::AnimationConfig,
        color::Color,
        component::{BorderPosition, Component},
        layout::{Anchor, BorderRadius, ComponentOffset, Edges, FlexValue, Position},
    },
    wgpu_ctx::WgpuCtx,
};
use tokio::sync::mpsc::UnboundedSender;

/// Common properties shared across component builders
#[derive(Default, Clone)]
pub struct CommonBuilderProps {
    pub width: Option<FlexValue>,
    pub height: Option<FlexValue>,
    pub position: Option<Position>,
    pub offset: Option<ComponentOffset>,
    pub margin: Option<Edges>,
    pub padding: Option<Edges>,
    pub z_index: Option<i32>,
    pub debug_name: Option<String>,
    pub border_width: Option<f32>,
    pub border_color: Option<Color>,
    pub border_position: Option<BorderPosition>,
    pub border_radius: Option<BorderRadius>,
    pub fit_to_size: bool,
    pub event_sender: Option<UnboundedSender<AppEvent>>,
    pub click_event: Option<AppEvent>,
    pub drag_event: Option<AppEvent>,
    pub animation: Option<AnimationConfig>,
    pub shadow_color: Option<Color>,
    pub shadow_offset: Option<(f32, f32)>,
    pub shadow_blur: Option<f32>,
    pub shadow_opacity: Option<f32>,
    pub clip_self: Option<bool>, // Whether component should be clipped by its parent
    pub as_inactive: bool,       // Whether component should be inactive on creation
}

/// Trait for component builders that share common properties
#[allow(dead_code)]
pub trait ComponentBuilder: Sized {
    fn common_props(&mut self) -> &mut CommonBuilderProps;

    fn with_size(mut self, width: impl Into<FlexValue>, height: impl Into<FlexValue>) -> Self {
        self.common_props().width = Some(width.into());
        self.common_props().height = Some(height.into());
        self
    }

    fn with_inactive(mut self) -> Self {
        self.common_props().as_inactive = true;
        self
    }

    fn with_width(mut self, width: impl Into<FlexValue>) -> Self {
        self.common_props().width = Some(width.into());
        self
    }

    fn with_height(mut self, height: impl Into<FlexValue>) -> Self {
        self.common_props().height = Some(height.into());
        self
    }

    fn with_position(mut self, position: Position) -> Self {
        self.common_props().position = Some(position);
        self
    }

    fn with_offset(mut self, offset: ComponentOffset) -> Self {
        self.common_props().offset = Some(offset);
        self
    }

    fn with_fixed_position(mut self, anchor: Anchor) -> Self {
        self.common_props().position = Some(Position::Fixed(anchor));
        self
    }

    fn with_margin(mut self, margin: Edges) -> Self {
        self.common_props().margin = Some(margin);
        self
    }

    fn with_padding(mut self, padding: Edges) -> Self {
        self.common_props().padding = Some(padding);
        self
    }

    fn with_z_index(mut self, z_index: i32) -> Self {
        self.common_props().z_index = Some(z_index);
        self
    }

    fn with_debug_name(mut self, name: impl Into<String>) -> Self {
        self.common_props().debug_name = Some(name.into());
        self
    }

    fn with_border(mut self, width: f32, color: Color) -> Self {
        self.common_props().border_width = Some(width);
        self.common_props().border_color = Some(color);
        self
    }

    fn with_border_full(mut self, width: f32, color: Color, position: BorderPosition) -> Self {
        self.common_props().border_width = Some(width);
        self.common_props().border_color = Some(color);
        self.common_props().border_position = Some(position);
        self
    }

    fn with_border_position(mut self, position: BorderPosition) -> Self {
        self.common_props().border_position = Some(position);
        self
    }

    fn with_border_radius(mut self, radius: BorderRadius) -> Self {
        self.common_props().border_radius = Some(radius);
        self
    }

    fn with_uniform_border_radius(mut self, radius: f32) -> Self {
        self.common_props().border_radius = Some(BorderRadius::all(radius));
        self
    }

    fn set_fit_to_size(mut self) -> Self {
        self.common_props().fit_to_size = true;
        self
    }

    fn with_event_sender(mut self, sender: UnboundedSender<AppEvent>) -> Self {
        self.common_props().event_sender = Some(sender);
        self
    }

    fn with_click_event(mut self, event: AppEvent) -> Self {
        self.common_props().click_event = Some(event);
        self
    }

    fn with_drag_event(mut self, event: AppEvent) -> Self {
        self.common_props().drag_event = Some(event);
        self
    }

    fn with_animation(mut self, animation: AnimationConfig) -> Self {
        self.common_props().animation = Some(animation);
        self
    }

    fn with_shadow(mut self, color: Color, offset: (f32, f32), blur: f32, opacity: f32) -> Self {
        self.common_props().shadow_color = Some(color);
        self.common_props().shadow_offset = Some(offset);
        self.common_props().shadow_blur = Some(blur);
        self.common_props().shadow_opacity = Some(opacity);
        self
    }

    fn with_clipping(mut self, clip_self: bool) -> Self {
        self.common_props().clip_self = Some(clip_self);
        self
    }

    fn allow_overflow(mut self) -> Self {
        self.common_props().clip_self = Some(false);
        self
    }

    /// This is only to be used for leaf components like image, background, etc.
    fn apply_common_props_for_leaf(&mut self, component: &mut Component, wgpu_ctx: &mut WgpuCtx) {
        let props = self.common_props();

        if let Some(debug_name) = props.debug_name.clone() {
            component.set_debug_name(debug_name);
        }

        if let Some(width) = &props.width {
            component.transform.size.width = width.clone();
        }

        if let Some(height) = &props.height {
            component.transform.size.height = height.clone();
        }

        if let Some(position) = props.position {
            component.transform.position_type = position;
        }

        if let Some(offset) = &props.offset {
            component.transform.offset = offset.clone();
        }

        if let Some(z_index) = props.z_index {
            component.set_z_index(z_index);
        }

        if let Some(margin) = props.margin {
            component.layout.margin = margin;
        }

        if let Some(padding) = props.padding {
            component.layout.padding = padding;
        }

        if let Some(border_width) = props.border_width {
            component.border_width = border_width;
        }

        if let Some(border_color) = props.border_color {
            component.border_color = border_color;
        }

        if let Some(border_position) = props.border_position {
            component.set_border_position(border_position);
        }

        if let Some(border_radius) = props.border_radius {
            component.set_border_radius(border_radius);
        }

        if props.fit_to_size {
            component.set_fit_to_size(true);
        }

        if let Some(event_sender) = props.event_sender.clone() {
            component.set_event_sender(event_sender);
        }

        if let Some(click_event) = props.click_event.clone() {
            component.set_click_event(click_event);
        }

        if let Some(drag_event) = props.drag_event.clone() {
            component.set_drag_event(drag_event);
        }

        if let Some(shadow_color) = props.shadow_color {
            component.shadow_color = shadow_color;
        }

        if let Some(shadow_offset) = props.shadow_offset {
            component.shadow_offset = shadow_offset;
        }

        if let Some(shadow_blur) = props.shadow_blur {
            component.shadow_blur = shadow_blur;
        }

        if let Some(shadow_opacity) = props.shadow_opacity {
            component.shadow_opacity = shadow_opacity;
        }

        if let Some(clip_self) = props.clip_self {
            component.clip_self = clip_self;
        }

        if props.as_inactive {
            component.set_as_inactive();
        }

        if let Some(animation) = props.animation.clone() {
            component.set_animation(animation, wgpu_ctx);
        }
    }

    // This function is used to apply common properties to component builders
    fn apply_common_properties<T: ComponentBuilder>(
        container_builder: &mut T,
        common_props: &CommonBuilderProps,
    ) {
        if let Some(width) = &common_props.width {
            container_builder.common_props().width = Some(width.clone());
        }
        if let Some(height) = &common_props.height {
            container_builder.common_props().height = Some(height.clone());
        }
        if let Some(name) = &common_props.debug_name {
            container_builder.common_props().debug_name = Some(name.clone());
        }
        if let Some(event) = &common_props.click_event {
            container_builder.common_props().click_event = Some(event.clone());
        }
        if let Some(event_sender) = &common_props.event_sender {
            container_builder.common_props().event_sender = Some(event_sender.clone());
        }
        if let Some(z_index) = common_props.z_index {
            container_builder.common_props().z_index = Some(z_index);
        }
        if let Some(margin) = common_props.margin {
            container_builder.common_props().margin = Some(margin);
        }
        if common_props.fit_to_size {
            container_builder.common_props().fit_to_size = true;
        }
        if let Some(position) = common_props.position {
            container_builder.common_props().position = Some(position);
        }
        if let Some(offset) = &common_props.offset {
            container_builder.common_props().offset = Some(offset.clone());
        }
        if let Some(padding) = common_props.padding {
            container_builder.common_props().padding = Some(padding);
        }
        if let Some(border_radius) = common_props.border_radius {
            container_builder.common_props().border_radius = Some(border_radius);
        }
        if let Some(clip_self) = common_props.clip_self {
            container_builder.common_props().clip_self = Some(clip_self);
        }
    }
}
