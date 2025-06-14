use crate::ui::{
    ecs::{
        ComponentType, EntityId, World,
        builders::{EntityBuilder, EntityBuilderProps, add_common_components},
        components::LayoutComponent,
    },
    layout::{
        AlignItems, FlexDirection, FlexWrap, JustifyContent, Layout, Overflow, ScrollOrientation,
    },
    z_index_manager::ZIndexManager,
};

pub struct ContainerBuilder {
    common: EntityBuilderProps,
    direction: FlexDirection,
    wrap: FlexWrap,
    justify_content: JustifyContent,
    align_items: AlignItems,
    is_scrollable: bool,
    scroll_orientation: ScrollOrientation,
    overflow_x: Overflow,
    overflow_y: Overflow,
}

impl EntityBuilder for ContainerBuilder {
    fn common_props(&mut self) -> &mut EntityBuilderProps {
        &mut self.common
    }
}

#[allow(dead_code)]
impl ContainerBuilder {
    pub fn new() -> Self {
        Self {
            common: EntityBuilderProps::default(),
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
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

    pub fn with_vertical_scroll(mut self) -> Self {
        self.is_scrollable = true;
        self.scroll_orientation = ScrollOrientation::Vertical;
        self.direction = FlexDirection::Column;
        self.overflow_y = Overflow::Hidden;
        self.common_props().clip_self = Some(false);
        self
    }

    pub fn with_horizontal_scroll(mut self) -> Self {
        self.is_scrollable = true;
        self.scroll_orientation = ScrollOrientation::Horizontal;
        self.direction = FlexDirection::Row;
        self.overflow_x = Overflow::Scroll;
        self.common_props().clip_self = Some(false);
        self
    }

    pub fn with_overflow_both(mut self, overflow: Overflow) -> Self {
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

    pub fn build(self, world: &mut World, z_index_manager: &mut ZIndexManager) -> EntityId {
        let entity_id = world.create_entity(self.common.debug_name.clone().expect(
            "Debug name is required for all components, tried to create a container without it.",
        ), ComponentType::Container);

        add_common_components(world, z_index_manager, entity_id, &self.common);

        // Create layout and set container-specific properties
        let mut layout = Layout::new();
        layout.direction = self.direction;
        layout.wrap = self.wrap;
        layout.justify_content = self.justify_content;
        layout.align_items = self.align_items;

        // Apply scrolling properties
        layout.is_scrollable = self.is_scrollable;
        layout.scroll_orientation = self.scroll_orientation;

        // Apply overflow settings
        layout.overflow_x = self.overflow_x;
        layout.overflow_y = self.overflow_y;

        // Apply margin and padding if set
        if let Some(margin) = self.common.margin {
            layout.margin = margin;
        }
        if let Some(padding) = self.common.padding {
            layout.padding = padding;
        }

        // Add layout component
        world.add_component(entity_id, LayoutComponent { layout });

        // Return the entity ID
        entity_id
    }
}
