use crate::{
    app::AppEvent,
    ui::{
        animation::{Animation, AnimationConfig, AnimationWhen},
        color::Color,
        ecs::{
            BorderPosition, EntityId, NamedRef, World,
            components::{
                AnimationComponent, BoundsComponent, HierarchyComponent, InteractionComponent,
                NotchPosition, NotchType, PreFitSizeComponent, TransformComponent, VisualComponent,
            },
            resources::NamedRefsResource,
        },
        layout::{
            Anchor, BorderRadius, ComponentOffset, Edges, FlexValue, LayoutSize, Position, Size,
        },
        z_index_manager::ZIndexManager,
    },
};

pub mod background;
pub mod button;
pub mod container;
pub mod image;
pub mod modal;
pub mod text;

/// Common properties shared across component builders
#[derive(Default, Clone, Debug)]
pub struct EntityBuilderProps {
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
    pub click_event: Option<AppEvent>,
    pub drag_event: Option<AppEvent>,
    pub animations: Vec<AnimationConfig>,
    pub shadow_color: Option<Color>,
    pub shadow_offset: Option<(f32, f32)>,
    pub shadow_blur: Option<f32>,
    pub shadow_opacity: Option<f32>,
    pub clip_self: Option<bool>, // Whether component should be clipped by its parent
    pub as_inactive: bool,       // Whether component should be inactive on creation
    pub named_ref: Option<NamedRef>,
    pub event_bubble_boundary: bool,
    pub notch: NotchType,
    pub notch_depth: f32,
    pub notch_flat_width: f32,
    pub notch_total_width: f32,
    pub notch_offset: f32,
    pub notch_position: NotchPosition,
}

/// Trait for component builders that share common properties
#[allow(dead_code)]
pub trait EntityBuilder: Sized {
    fn common_props(&mut self) -> &mut EntityBuilderProps;

    fn with_external_common_props(mut self, external_common: EntityBuilderProps) -> Self {
        let curr_common = self.common_props();
        *curr_common = external_common;
        self
    }

    fn with_size(mut self, width: impl Into<FlexValue>, height: impl Into<FlexValue>) -> Self {
        self.common_props().width = Some(width.into());
        self.common_props().height = Some(height.into());
        self
    }

    fn with_spawn_as_inactive(mut self) -> Self {
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

    fn with_absolute_position(mut self, anchor: Anchor) -> Self {
        self.common_props().position = Some(Position::Absolute(anchor));
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

    fn with_click_event(mut self, event: AppEvent) -> Self {
        self.common_props().click_event = Some(event);
        self
    }

    fn with_drag_event(mut self, event: AppEvent) -> Self {
        self.common_props().drag_event = Some(event);
        self
    }

    fn with_animation(mut self, animation: AnimationConfig) -> Self {
        self.common_props().animations.push(animation);
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

    fn with_named_ref(mut self, named_ref: NamedRef) -> Self {
        self.common_props().named_ref = Some(named_ref);
        self
    }

    fn with_event_bubble_boundary(mut self, is_blocking: bool) -> Self {
        self.common_props().event_bubble_boundary = is_blocking;
        self
    }

    fn with_notch(
        mut self,
        notch: NotchType,
        notch_position: NotchPosition,
        depth: f32,
        flat_width: f32,
        total_width: f32,
        offset: f32,
    ) -> Self {
        self.common_props().notch = notch;
        self.common_props().notch_depth = depth;
        self.common_props().notch_flat_width = flat_width;
        self.common_props().notch_total_width = total_width;
        self.common_props().notch_offset = offset;
        self.common_props().notch_position = notch_position;
        self
    }
}

/// Adds Animation, Transform, Hierarchy, and Visual components to the entity
pub fn add_common_components(
    world: &mut World,
    z_index_manager: &mut ZIndexManager,
    entity_id: EntityId,
    props: &EntityBuilderProps,
) {
    // Ensure NamedEntity if set is not already in the world
    if let Some(named_ref) = props.named_ref {
        let named_refs_resource = world
            .resources
            .get_resource_mut::<NamedRefsResource>()
            .expect("NamedRefsResource should be initialized before adding components");

        named_refs_resource.set_entity_id(named_ref, entity_id);
    }

    let mut has_entry_anim = false;

    // Add animation component if configured
    for animation_config in &props.animations {
        if animation_config.when == AnimationWhen::Entry {
            has_entry_anim = true;
        }

        let animation = Animation::new(animation_config.clone());
        // check if animation component already exists
        if let Some(animation_comp) = world
            .components
            .get_component_mut::<AnimationComponent>(entity_id)
        {
            animation_comp.animations.push(animation);
        } else {
            // Create a new animation component if it doesn't exist
            world.add_component(
                entity_id,
                AnimationComponent {
                    animations: vec![animation],
                },
            );
        }
    }

    if let Some(custom_z_index) = props.z_index {
        z_index_manager.set_adjustment(entity_id, custom_z_index);
    }

    // Add transform component
    world.add_component(
        entity_id,
        TransformComponent {
            size: LayoutSize {
                width: props.width.clone().unwrap_or(FlexValue::Fill),
                height: props.height.clone().unwrap_or(FlexValue::Fill),
                min_width: FlexValue::default(),
                min_height: FlexValue::default(),
                max_width: FlexValue::default(),
                max_height: FlexValue::default(),
            },
            offset: props.offset.clone().unwrap_or_default(),
            position_type: props.position.unwrap_or_default(),
            scale_factor: 1.0,
        },
    );

    // Add hierarchy component
    world.add_component(
        entity_id,
        HierarchyComponent {
            parent: None,
            children: Vec::new(),
        },
    );

    // Add visual component
    world.add_component(
        entity_id,
        VisualComponent {
            border_width: props.border_width.unwrap_or(0.0),
            border_color: props.border_color.unwrap_or(Color::Transparent),
            border_position: props.border_position.unwrap_or_default(),
            border_radius: props.border_radius.unwrap_or_default(),
            shadow_color: props.shadow_color.unwrap_or(Color::Transparent),
            shadow_offset: props.shadow_offset.unwrap_or((0.0, 0.0)),
            shadow_blur: props.shadow_blur.unwrap_or(0.0),
            shadow_opacity: props.shadow_opacity.unwrap_or(1.0),
            opacity: 1.0,
            notch: props.notch,
            notch_depth: props.notch_depth,
            notch_flat_width: props.notch_flat_width,
            notch_total_width: props.notch_total_width,
            notch_offset: props.notch_offset,
            notch_position: props.notch_position,
        },
    );

    // Add bounds component
    world.add_component(
        entity_id,
        BoundsComponent {
            computed_bounds: Default::default(),
            screen_size: Size::default(),
            clip_bounds: None,
            clip_self: props.clip_self.unwrap_or(true),
            fit_to_size: props.fit_to_size,
        },
    );

    let is_just_activated = if props.as_inactive {
        false
    } else {
        has_entry_anim
    };

    // Add interaction component
    world.add_component(
        entity_id,
        InteractionComponent {
            is_clickable: props.click_event.is_some(),
            is_draggable: props.drag_event.is_some(),
            is_hoverable: props
                .animations
                .iter()
                .any(|a| a.when == AnimationWhen::Hover),
            is_hovered: false,
            click_event: props.click_event,
            drag_event: props.drag_event,
            is_active: !props.as_inactive,
            is_just_activated,
            is_just_deactivated: false,
            is_event_bubble_boundary: props.event_bubble_boundary,
        },
    );

    // PreFitBoundsComponent
    if props.fit_to_size {
        world.add_component(
            entity_id,
            PreFitSizeComponent {
                original_width: props.width.clone().unwrap_or(FlexValue::Fill),
                original_height: props.height.clone().unwrap_or(FlexValue::Fill),
            },
        );
    }
}
