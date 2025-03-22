use crate::app::AppEvent;
use crate::ui::color::Color;
use crate::ui::component::{AnimationConfig, BorderPosition, Component};
use crate::ui::layout::{Anchor, BorderRadius, Edges, FlexValue, Position};
use crate::wgpu_ctx::WgpuCtx;
use tokio::sync::mpsc::UnboundedSender;

/// Common properties shared across component builders
#[derive(Default, Clone)]
pub struct CommonBuilderProps {
    pub width: Option<FlexValue>,
    pub height: Option<FlexValue>,
    pub position: Option<Position>,
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
}

/// Trait for component builders that share common properties
pub trait ComponentBuilder: Sized {
    fn common_props(&mut self) -> &mut CommonBuilderProps;

    fn with_size(mut self, width: impl Into<FlexValue>, height: impl Into<FlexValue>) -> Self {
        self.common_props().width = Some(width.into());
        self.common_props().height = Some(height.into());
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

    fn apply_common_props(&mut self, component: &mut Component, wgpu_ctx: &mut WgpuCtx) {
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

        if let Some(animation) = props.animation.clone() {
            component.set_animation(animation, wgpu_ctx);
        }
    }
}
