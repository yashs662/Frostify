use crate::ui::{
    component::{Component, ComponentType},
    layout::{AlignItems, FlexDirection, FlexWrap, JustifyContent},
};
use uuid::Uuid;

use super::component_builder::{CommonBuilderProps, ComponentBuilder};

pub struct FlexContainerBuilder {
    common: CommonBuilderProps,
    direction: FlexDirection,
    wrap: FlexWrap,
    justify_content: JustifyContent,
    align_items: AlignItems,
    parent_id: Option<Uuid>,
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

    pub fn build(mut self) -> Component {
        let container_id = Uuid::new_v4();
        let mut container = Component::new(container_id, ComponentType::Container);

        self.apply_common_props(&mut container);

        container.layout.direction = self.direction;
        container.layout.wrap = self.wrap;
        container.layout.justify_content = self.justify_content;
        container.layout.align_items = self.align_items;

        if let Some(parent_id) = self.parent_id {
            container.set_parent(parent_id);
        }

        container
    }
}
