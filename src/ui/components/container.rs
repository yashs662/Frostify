use crate::{
    app::AppEvent,
    ui::{
        components::core::component::{Component, ComponentType},
        layout::{AlignItems, Edges, FlexDirection, FlexValue, FlexWrap, JustifyContent, Layout},
    },
};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FlexContainerConfig {
    pub width: FlexValue,
    pub height: FlexValue,
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub padding: Option<Edges>,
    pub margin: Option<Edges>,
    pub debug_name: Option<String>,
    pub parent_id: Option<Uuid>,
    pub z_index: Option<i32>,
    pub click_event: Option<AppEvent>,
    pub drag_event: Option<AppEvent>,
    pub event_sender: Option<UnboundedSender<AppEvent>>,
}

impl Default for FlexContainerConfig {
    fn default() -> Self {
        Self {
            width: FlexValue::Fill,
            height: FlexValue::Fill,
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            padding: None,
            margin: None,
            debug_name: None,
            parent_id: None,
            z_index: None,
            click_event: None,
            drag_event: None,
            event_sender: None,
        }
    }
}

pub struct FlexContainerBuilder {
    config: FlexContainerConfig,
}

#[allow(dead_code)]
impl FlexContainerBuilder {
    pub fn new() -> Self {
        Self {
            config: FlexContainerConfig::default(),
        }
    }

    pub fn with_width(mut self, width: FlexValue) -> Self {
        self.config.width = width;
        self
    }

    pub fn with_height(mut self, height: FlexValue) -> Self {
        self.config.height = height;
        self
    }

    pub fn with_size(mut self, width: FlexValue, height: FlexValue) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    pub fn with_direction(mut self, direction: FlexDirection) -> Self {
        self.config.direction = direction;
        self
    }

    pub fn with_wrap(mut self, wrap: FlexWrap) -> Self {
        self.config.wrap = wrap;
        self
    }

    pub fn with_justify_content(mut self, justify: JustifyContent) -> Self {
        self.config.justify_content = justify;
        self
    }

    pub fn with_align_items(mut self, align: AlignItems) -> Self {
        self.config.align_items = align;
        self
    }

    pub fn with_padding(mut self, padding: Edges) -> Self {
        self.config.padding = Some(padding);
        self
    }

    pub fn with_margin(mut self, margin: Edges) -> Self {
        self.config.margin = Some(margin);
        self
    }

    pub fn with_debug_name(mut self, name: impl Into<String>) -> Self {
        self.config.debug_name = Some(name.into());
        self
    }

    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.config.parent_id = Some(parent_id);
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.config.z_index = Some(z_index);
        self
    }

    pub fn with_click_event(mut self, event: AppEvent) -> Self {
        self.config.click_event = Some(event);
        self
    }

    pub fn with_drag_event(mut self, event: AppEvent) -> Self {
        self.config.drag_event = Some(event);
        self
    }

    pub fn with_event_sender(mut self, event_sender: UnboundedSender<AppEvent>) -> Self {
        self.config.event_sender = Some(event_sender);
        self
    }

    pub fn build(self) -> Component {
        create_flex_container(self.config)
    }
}

fn create_flex_container(config: FlexContainerConfig) -> Component {
    let container_id = Uuid::new_v4();
    let mut container = Component::new(container_id, ComponentType::Container);

    // Set size
    container.transform.size.width = config.width;
    container.transform.size.height = config.height;

    // Set layout properties
    container.layout = Layout::new();
    container.layout.direction = config.direction;
    container.layout.wrap = config.wrap;
    container.layout.justify_content = config.justify_content;
    container.layout.align_items = config.align_items;

    // Set optional properties
    if let Some(padding) = config.padding {
        container.layout.padding = padding;
    }
    if let Some(margin) = config.margin {
        container.layout.margin = margin;
    }
    if let Some(name) = config.debug_name {
        container.set_debug_name(name);
    }
    if let Some(parent_id) = config.parent_id {
        container.set_parent(parent_id);
    }
    if let Some(z_index) = config.z_index {
        container.set_z_index(z_index);
    }
    if let Some(event) = config.click_event {
        container.set_click_event(event);
    }
    if let Some(event) = config.drag_event {
        container.set_drag_event(event);
    }
    if let Some(event_sender) = config.event_sender {
        container.set_event_sender(event_sender);
    }

    container
}
