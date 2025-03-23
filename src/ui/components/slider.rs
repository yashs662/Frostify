use uuid::Uuid;

use crate::{
    ui::{
        color::Color,
        component::{
            BackgroundColorConfig, BorderPosition, Component, ComponentConfig, ComponentMetaData,
            ComponentType,
        },
        components::component_builder::CommonBuilderProps,
        layout::{AlignItems, Anchor, BorderRadius, FlexValue, JustifyContent, Position},
    },
    wgpu_ctx::WgpuCtx,
};

use super::{component_builder::ComponentBuilder, container::FlexContainerBuilder};

#[derive(Debug, Clone)]
pub struct SliderData {
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub value: f32,
    pub track_id: Uuid,
    pub thumb_id: Uuid,
}

pub struct SliderBuilder {
    common: CommonBuilderProps,
    config: SliderConfig,
}

#[derive(Debug, Clone)]
pub struct SliderConfig {
    min: f32,
    max: f32,
    step: f32,
    value: f32,
    thumb_color: Color,
    thumb_border_radius: Option<BorderRadius>,
    thumb_size: Option<f32>,
    thumb_border_color: Option<Color>,
    thumb_border_width: Option<f32>,
    thumb_border_position: Option<BorderPosition>,
    track_color: Color,
    track_fractional_height: Option<f32>,
    track_border_color: Option<Color>,
    track_border_width: Option<f32>,
    track_border_position: Option<BorderPosition>,
    fill_color: Color,
    fill_border_color: Option<Color>,
    fill_border_width: Option<f32>,
    fill_border_position: Option<BorderPosition>,
}

impl Default for SliderConfig {
    fn default() -> Self {
        SliderConfig {
            min: 0.0,
            max: 100.0,
            step: 1.0,
            value: 50.0,
            thumb_color: Color::White,
            thumb_border_radius: None,
            thumb_size: None,
            thumb_border_color: None,
            thumb_border_width: None,
            thumb_border_position: None,
            track_color: Color::DarkGray,
            track_fractional_height: None,
            track_border_color: None,
            track_border_width: None,
            track_border_position: None,
            fill_color: Color::Blue,
            fill_border_color: None,
            fill_border_width: None,
            fill_border_position: None,
        }
    }
}

impl ComponentBuilder for SliderBuilder {
    fn common_props(&mut self) -> &mut CommonBuilderProps {
        &mut self.common
    }
}

impl SliderBuilder {
    pub fn new() -> Self {
        SliderBuilder {
            common: CommonBuilderProps::default(),
            config: SliderConfig::default(),
        }
    }

    pub fn with_min(mut self, min: f32) -> Self {
        self.config.min = min;
        self
    }

    pub fn with_max(mut self, max: f32) -> Self {
        self.config.max = max;
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.config.step = step;
        self
    }

    pub fn with_value(mut self, value: f32) -> Self {
        self.config.value = value;
        self
    }

    pub fn with_thumb_color(mut self, color: Color) -> Self {
        self.config.thumb_color = color;
        self
    }

    pub fn with_thumb_border_radius(mut self, radius: BorderRadius) -> Self {
        self.config.thumb_border_radius = Some(radius);
        self
    }

    pub fn with_thumb_size(mut self, size: f32) -> Self {
        self.config.thumb_size = Some(size);
        self
    }

    pub fn with_thumb_border_color(mut self, color: Color) -> Self {
        self.config.thumb_border_color = Some(color);
        self
    }

    pub fn with_thumb_border_width(mut self, width: f32) -> Self {
        self.config.thumb_border_width = Some(width);
        self
    }

    pub fn with_thumb_border_position(mut self, position: BorderPosition) -> Self {
        self.config.thumb_border_position = Some(position);
        self
    }

    pub fn with_track_color(mut self, color: Color) -> Self {
        self.config.track_color = color;
        self
    }

    pub fn with_track_fractional_height(mut self, height: f32) -> Self {
        self.config.track_fractional_height = Some(height);
        self
    }

    pub fn with_track_border_color(mut self, color: Color) -> Self {
        self.config.track_border_color = Some(color);
        self
    }

    pub fn with_track_border_width(mut self, width: f32) -> Self {
        self.config.track_border_width = Some(width);
        self
    }

    pub fn with_track_border_position(mut self, position: BorderPosition) -> Self {
        self.config.track_border_position = Some(position);
        self
    }

    pub fn with_fill_color(mut self, color: Color) -> Self {
        self.config.fill_color = color;
        self
    }

    pub fn build(self, wgpu_ctx: &mut WgpuCtx) -> Component {
        let mut container_builder = FlexContainerBuilder::new();
        let common_props = self.common.clone();
        let config = self.config.clone();

        <SliderBuilder as ComponentBuilder>::apply_common_properties(
            &mut container_builder,
            &common_props,
        );

        let mut container = container_builder
            .with_debug_name("Slider Container")
            .build(wgpu_ctx);

        container.flag_children_extraction();
        container.set_fit_to_size(true);

        let mut track_bg = Component::new(Uuid::new_v4(), ComponentType::BackgroundColor);
        track_bg.transform.position_type = Position::Fixed(Anchor::Left);
        track_bg.set_debug_name("Slider Track Background");
        track_bg.set_z_index(0);
        track_bg.set_border_radius(container.transform.border_radius);

        if let Some(color) = config.track_border_color {
            track_bg.border_color = color;
        }
        if let Some(width) = config.track_border_width {
            track_bg.border_width = width;
        }
        if let Some(position) = config.track_border_position {
            track_bg.set_border_position(position);
        }
        if let Some(height) = config.track_fractional_height {
            track_bg.transform.size.height = FlexValue::Fraction(height);
        }

        track_bg.configure(
            ComponentConfig::BackgroundColor(BackgroundColorConfig {
                color: config.track_color,
            }),
            wgpu_ctx,
        );

        // Calculate normalized value (0.0 to 1.0) for positioning
        let normalized_value = if config.max > config.min {
            (config.value - config.min) / (config.max - config.min)
        } else {
            0.0 // Avoid division by zero
        };

        // Create the track fill
        let track_fill_id = Uuid::new_v4();
        let mut track_fill = Component::new(track_fill_id, ComponentType::BackgroundColor);
        track_fill.transform.position_type = Position::Fixed(Anchor::Left);
        track_fill.set_debug_name("Slider Track Fill");
        track_fill.set_z_index(1);
        track_fill.set_border_radius(container.transform.border_radius);

        if let Some(color) = config.fill_border_color {
            track_fill.border_color = color;
        }
        if let Some(width) = config.fill_border_width {
            track_fill.border_width = width;
        }
        if let Some(position) = config.fill_border_position {
            track_fill.set_border_position(position);
        }
        if let Some(height) = config.track_fractional_height {
            track_fill.transform.size.height = FlexValue::Fraction(height);
        }

        // Set fill width based on normalized value - make sure it's in range
        let clamped_normalized = normalized_value.clamp(0.0, 1.0);
        track_fill.transform.size.width = FlexValue::Fraction(clamped_normalized);

        track_fill.configure(
            ComponentConfig::BackgroundColor(BackgroundColorConfig {
                color: config.fill_color,
            }),
            wgpu_ctx,
        );

        // Create the thumb
        let thumb_id = Uuid::new_v4();
        let mut thumb = Component::new(thumb_id, ComponentType::BackgroundColor);
        thumb.transform.position_type = Position::Fixed(Anchor::Left);
        thumb.set_debug_name("Slider Thumb");
        thumb.set_z_index(2);

        if let Some(radius) = config.thumb_border_radius {
            thumb.set_border_radius(radius);
        }
        if let Some(color) = config.thumb_border_color {
            thumb.border_color = color;
        }
        if let Some(width) = config.thumb_border_width {
            thumb.border_width = width;
        }
        if let Some(position) = config.thumb_border_position {
            thumb.set_border_position(position);
        }

        // Get or set the thumb size
        let thumb_size = if let Some(size) = config.thumb_size {
            size
        } else {
            10.0
        };

        thumb.transform.size.width = FlexValue::Fixed(thumb_size);
        thumb.transform.size.height = FlexValue::Fixed(thumb_size);

        thumb.configure(
            ComponentConfig::BackgroundColor(BackgroundColorConfig {
                color: config.thumb_color,
            }),
            wgpu_ctx,
        );

        container.add_child(track_bg);
        container.add_child(track_fill);
        container.add_child(thumb);

        // Store slider data in the container
        container
            .metadata
            .push(ComponentMetaData::SliderData(SliderData {
                min: config.min,
                max: config.max,
                step: config.step,
                value: config.value,
                track_id: track_fill_id,
                thumb_id,
            }));

        container
    }
}
