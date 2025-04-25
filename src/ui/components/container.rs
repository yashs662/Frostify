use crate::{
    ui::{
        component::{Component, ComponentType},
        components::component_builder::{CommonBuilderProps, ComponentBuilder},
        layout::{
            AlignItems, FlexDirection, FlexWrap, JustifyContent, Overflow, ScrollOrientation,
        },
    },
    wgpu_ctx::WgpuCtx,
};
use uuid::Uuid;

pub struct FlexContainerBuilder {
    common: CommonBuilderProps,
    direction: FlexDirection,
    wrap: FlexWrap,
    justify_content: JustifyContent,
    align_items: AlignItems,
    parent_id: Option<Uuid>,
    is_scrollable: bool,
    scroll_orientation: ScrollOrientation,
    overflow_x: Overflow,
    overflow_y: Overflow,
}

impl ComponentBuilder for FlexContainerBuilder {
    fn common_props(&mut self) -> &mut CommonBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl FlexContainerBuilder {
    pub fn new() -> Self {
        Self {
            common: CommonBuilderProps::default(),
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            parent_id: None,
            is_scrollable: false,
            scroll_orientation: ScrollOrientation::Vertical,
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
        }
    }

    pub fn with_direction(mut self, direction: FlexDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_wrap(mut self, wrap: FlexWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn with_justify_content(mut self, justify: JustifyContent) -> Self {
        self.justify_content = justify;
        self
    }

    pub fn with_align_items(mut self, align: AlignItems) -> Self {
        self.align_items = align;
        self
    }

    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_scroll(mut self, orientation: ScrollOrientation) -> Self {
        self.is_scrollable = true;
        self.scroll_orientation = orientation;
        self
    }

    pub fn with_vertical_scroll(mut self) -> Self {
        self.is_scrollable = true;
        self.scroll_orientation = ScrollOrientation::Vertical;
        // Set direction to column for vertical scrolling
        self.direction = FlexDirection::Column;
        // Ensure overflow is set to Hidden, not just Scroll - Hidden is what performs the clipping
        self.overflow_y = Overflow::Hidden;
        // Don't clip the container itself, only its children
        self.common_props().clip_self = Some(false);
        self
    }

    pub fn with_horizontal_scroll(mut self) -> Self {
        self.is_scrollable = true;
        self.scroll_orientation = ScrollOrientation::Horizontal;
        // Set direction to row for horizontal scrolling
        self.direction = FlexDirection::Row;
        // Set horizontal overflow to scroll
        self.overflow_x = Overflow::Scroll;
        // Don't clip the container itself, only its children
        self.common_props().clip_self = Some(false);
        self
    }

    pub fn scrollable(mut self, orientation: ScrollOrientation) -> Self {
        self.is_scrollable = true;
        self.scroll_orientation = orientation;

        // Set appropriate direction based on orientation
        match orientation {
            ScrollOrientation::Vertical => self.direction = FlexDirection::Column,
            ScrollOrientation::Horizontal => self.direction = FlexDirection::Row,
        }

        // Don't clip the container itself, only its children
        self.common_props().clip_self = Some(false);
        self
    }

    pub fn with_overflow(mut self, overflow: Overflow) -> Self {
        self.overflow_x = overflow;
        self.overflow_y = overflow;
        self
    }

    pub fn with_overflow_x(mut self, overflow: Overflow) -> Self {
        self.overflow_x = overflow;
        self
    }

    pub fn with_overflow_y(mut self, overflow: Overflow) -> Self {
        self.overflow_y = overflow;
        self
    }

    pub fn with_hidden_overflow(mut self) -> Self {
        self.overflow_x = Overflow::Hidden;
        self.overflow_y = Overflow::Hidden;
        self
    }

    pub fn build(mut self, wgpu_ctx: &mut WgpuCtx) -> Component {
        let container_id = Uuid::new_v4();
        let mut container = Component::new(container_id, ComponentType::Container);

        self.apply_common_props_for_leaf(&mut container, wgpu_ctx);

        container.layout.direction = self.direction;
        container.layout.wrap = self.wrap;
        container.layout.justify_content = self.justify_content;
        container.layout.align_items = self.align_items;

        // Apply scrolling properties
        container.layout.is_scrollable = self.is_scrollable;
        container.layout.scroll_orientation = self.scroll_orientation;

        // Apply overflow settings
        container.layout.overflow_x = self.overflow_x;
        container.layout.overflow_y = self.overflow_y;

        // If this is a scrollable container, set clip_self to false by default
        // (unless explicitly set otherwise)
        if container.layout.is_scrollable && self.common_props().clip_self.is_none() {
            container.clip_self = false;
        }

        if let Some(parent_id) = self.parent_id {
            container.set_parent(parent_id);
        }

        container
    }
}
