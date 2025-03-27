use crate::{
    ui::{
        color::Color,
        component::{Component, ComponentMetaData},
        layout::{Anchor, BorderRadius, FlexValue},
    },
    wgpu_ctx::WgpuCtx,
};

use super::{
    background::BackgroundBuilder,
    component_builder::{CommonBuilderProps, ComponentBuilder},
    container::FlexContainerBuilder,
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct SliderData {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub thumb_id: Uuid,
    pub track_background_id: Uuid,
}

// New struct to safely transmit slider update data
#[derive(Debug, Clone, Copy)]
pub struct SliderUpdateData {
    pub track_start_x: f32,
    pub track_width: f32,
    pub thumb_id: Uuid,
    pub track_fill_id: Uuid,
    pub fill_percentage: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct SliderBuilderConfig {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

impl Default for SliderBuilderConfig {
    fn default() -> Self {
        Self {
            value: 50.0,
            min: 0.0,
            max: 100.0,
            step: 10.0,
        }
    }
}

pub struct SliderBuilder {
    common: CommonBuilderProps,
    config: SliderBuilderConfig,
}

impl ComponentBuilder for SliderBuilder {
    fn common_props(&mut self) -> &mut CommonBuilderProps {
        &mut self.common
    }
}

impl SliderBuilder {
    pub fn new() -> Self {
        Self {
            common: CommonBuilderProps::default(),
            config: SliderBuilderConfig::default(),
        }
    }

    pub fn build(self, wgpu_ctx: &mut WgpuCtx) -> Component {
        let mut container_builder = FlexContainerBuilder::new();
        let common_props = self.common.clone();
        let config = self.config;

        // Use fully qualified path to call the trait method
        <FlexContainerBuilder as ComponentBuilder>::apply_common_properties(
            &mut container_builder,
            &common_props,
        );

        let mut container = container_builder.build(wgpu_ctx);
        container.flag_children_extraction();

        let fill_percentage = (config.value - config.min) / (config.max - config.min);

        // Create thumb with improved hit area
        let mut thumb = BackgroundBuilder::with_color(Color::White)
            .with_size(10.0, 10.0)
            .with_border_radius(BorderRadius::all(999.0))
            .with_fixed_position(Anchor::Left)
            .with_z_index(2)
            .build(wgpu_ctx);

        thumb.transform.offset.x = FlexValue::Fraction(fill_percentage);

        // Create track fill
        let track_fill = BackgroundBuilder::with_color(Color::Blue)
            .with_height(FlexValue::Fraction(0.5))
            .with_width(FlexValue::Fraction(fill_percentage))
            .with_border_radius(BorderRadius::all(5.0))
            .with_fixed_position(Anchor::Left)
            .with_z_index(1)
            .build(wgpu_ctx);

        // Create track background
        let track_background = BackgroundBuilder::with_color(Color::DarkGray)
            .with_height(FlexValue::Fraction(0.5))
            .with_fixed_position(Anchor::Left)
            .with_border_radius(BorderRadius::all(5.0))
            .with_z_index(0)
            .build(wgpu_ctx);

        let slider_data = SliderData {
            value: config.value,
            min: config.min,
            max: config.max,
            step: config.step,
            thumb_id: thumb.id,
            track_background_id: track_fill.id,
        };

        container
            .metadata
            .push(ComponentMetaData::SliderData(slider_data));
        
        // Add the children in order (back to front)
        container.add_child(track_background);
        container.add_child(track_fill);
        container.add_child(thumb);

        container
    }
}
