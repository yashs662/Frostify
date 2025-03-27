use crate::{
    ui::{
        color::Color,
        component::{Component, ComponentMetaData},
        component_update::{CanProvideUpdates, ComponentUpdate},
        layout::{Anchor, BorderRadius, Bounds, FlexValue},
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
    pub track_fill_id: Uuid,
    pub needs_update: bool,
    pub track_bounds: Option<Bounds>,
}

/// A specialized update for sliders that updates both thumb position and track fill
pub struct SliderUpdate {
    pub thumb_id: Uuid,
    pub track_fill_id: Uuid,
    pub fill_percentage: f32,
    pub track_bounds: Bounds,
}

impl ComponentUpdate for SliderUpdate {
    fn apply(&self, component: &mut Component, wgpu_ctx: &mut WgpuCtx) {
        // We need to apply the update based on which component we're dealing with
        if component.id == self.thumb_id {
            // Update thumb position
            component.transform.offset.x = FlexValue::Fraction(self.fill_percentage);

            // Calculate the actual position for immediate visual update
            let mut current_bounds = component.computed_bounds;
            current_bounds.position.x = self.track_bounds.position.x
                + (self.track_bounds.size.width * self.fill_percentage)
                - (current_bounds.size.width / 2.0);
            component.computed_bounds = current_bounds;

            // Update the GPU buffer with new position
            if let Some(buffer) = component.get_render_data_buffer() {
                wgpu_ctx.queue.write_buffer(
                    buffer,
                    0,
                    bytemuck::cast_slice(&[component.get_render_data(current_bounds)]),
                );
            }
        } else if component.id == self.track_fill_id {
            // Update track fill size
            component.transform.size.width = FlexValue::Fraction(self.fill_percentage);

            // Calculate the actual width for immediate visual update
            let mut current_bounds = component.computed_bounds;
            current_bounds.size.width = self.track_bounds.size.width * self.fill_percentage;
            component.computed_bounds = current_bounds;

            // Make sure we're properly passing the updated bounds to get_render_data
            // to ensure the proper size is rendered
            if let Some(buffer) = component.get_render_data_buffer() {
                wgpu_ctx.queue.write_buffer(
                    buffer,
                    0,
                    bytemuck::cast_slice(&[component.get_render_data(current_bounds)]),
                );
            }
        }
    }

    fn target_id(&self) -> Uuid {
        // Return thumb ID as the primary target
        self.thumb_id
    }

    fn additional_target_ids(&self) -> Vec<Uuid> {
        // Return track fill ID as an additional target
        vec![self.track_fill_id]
    }
}

impl CanProvideUpdates for Component {
    fn get_update_data(&self) -> Option<Box<dyn ComponentUpdate>> {
        // Check if this is a slider that needs update
        if self.is_a_slider() {
            if let Some(ComponentMetaData::SliderData(slider_data)) = self
                .metadata
                .iter()
                .find(|m| matches!(m, ComponentMetaData::SliderData(_)))
            {
                if slider_data.needs_update && slider_data.track_bounds.is_some() {
                    // Calculate normalized percentage (0.0 to 1.0)
                    let fill_percentage =
                        (slider_data.value - slider_data.min) / (slider_data.max - slider_data.min);

                    return Some(Box::new(SliderUpdate {
                        thumb_id: slider_data.thumb_id,
                        track_fill_id: slider_data.track_fill_id,
                        fill_percentage,
                        track_bounds: slider_data.track_bounds.unwrap(),
                    }));
                }
            }
        }
        None
    }

    fn has_updates(&self) -> bool {
        if self.is_a_slider() {
            if let Some(ComponentMetaData::SliderData(slider_data)) = self
                .metadata
                .iter()
                .find(|m| matches!(m, ComponentMetaData::SliderData(_)))
            {
                return slider_data.needs_update && slider_data.track_bounds.is_some();
            }
        }
        self.needs_update()
    }

    fn reset_update_state(&mut self) {
        if self.is_a_slider() {
            if let Some(ComponentMetaData::SliderData(slider_data)) = self
                .metadata
                .iter_mut()
                .find(|m| matches!(m, ComponentMetaData::SliderData(_)))
            {
                slider_data.needs_update = false;
            }
        }
        self.clear_update_flag();
    }
}

// New specialized trait for sliders
pub trait SliderBehavior {
    fn set_value(&mut self, value: f32);
    fn reset_drag_state(&mut self);
}

impl SliderBehavior for Component {
    fn set_value(&mut self, value: f32) {
        if let Some(ComponentMetaData::SliderData(data)) = self
            .metadata
            .iter_mut()
            .find(|m| matches!(m, ComponentMetaData::SliderData(_)))
        {
            // Clamp the value to the min/max range
            let clamped_value = value.clamp(data.min, data.max);

            // Apply stepping if step is non-zero
            let new_value = if data.step > 0.0 {
                let steps = ((clamped_value - data.min) / data.step).round();
                data.min + steps * data.step
            } else {
                clamped_value
            };

            // Update the value and mark for update
            if (new_value - data.value).abs() > f32::EPSILON {
                data.value = new_value;
                data.needs_update = true;
            }
        }
    }

    fn reset_drag_state(&mut self) {
        if let Some(ComponentMetaData::SliderData(data)) = self
            .metadata
            .iter_mut()
            .find(|m| matches!(m, ComponentMetaData::SliderData(_)))
        {
            // Simply mark the slider as not being dragged/grabbed
            data.needs_update = false;

            // Force a refresh of the slider to ensure proper visual state
            self.refresh_slider();
        }
    }
}

#[derive(Debug, Clone)]
pub struct SliderBuilderConfig {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    // Visual configuration
    pub thumb_color: Color,
    pub thumb_size: f32,
    pub thumb_border_radius: Option<BorderRadius>,
    pub track_color: Color,
    pub track_fill_color: Color,
    pub track_height: FlexValue,
    pub track_border_radius: Option<BorderRadius>,
}

impl Default for SliderBuilderConfig {
    fn default() -> Self {
        Self {
            value: 50.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            // Default visual appearance
            thumb_color: Color::White,
            thumb_size: 10.0,
            thumb_border_radius: Some(BorderRadius::all(999.0)), // Circle by default
            track_color: Color::DarkGray,
            track_fill_color: Color::Blue,
            track_height: FlexValue::Fraction(0.5),
            track_border_radius: Some(BorderRadius::all(5.0)),
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

    // Configuration methods for slider values
    pub fn with_value(mut self, value: f32) -> Self {
        self.config.value = value;
        self
    }

    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.config.min = min;
        self.config.max = max;
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.config.step = step;
        self
    }

    // Configuration methods for thumb appearance
    pub fn with_thumb_color(mut self, color: Color) -> Self {
        self.config.thumb_color = color;
        self
    }

    pub fn with_thumb_size(mut self, size: f32) -> Self {
        self.config.thumb_size = size;
        self
    }

    pub fn with_thumb_border_radius(mut self, radius: BorderRadius) -> Self {
        self.config.thumb_border_radius = Some(radius);
        self
    }

    // Configuration methods for track appearance
    pub fn with_track_color(mut self, color: Color) -> Self {
        self.config.track_color = color;
        self
    }

    pub fn with_track_fill_color(mut self, color: Color) -> Self {
        self.config.track_fill_color = color;
        self
    }

    pub fn with_track_height(mut self, height: impl Into<FlexValue>) -> Self {
        self.config.track_height = height.into();
        self
    }

    pub fn with_track_border_radius(mut self, radius: BorderRadius) -> Self {
        self.config.track_border_radius = Some(radius);
        self
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

        // Create thumb with customizable appearance
        let mut thumb_builder = BackgroundBuilder::with_color(config.thumb_color)
            .with_size(config.thumb_size, config.thumb_size)
            .with_fixed_position(Anchor::Left)
            .with_z_index(2);

        if let Some(radius) = config.thumb_border_radius {
            thumb_builder = thumb_builder.with_border_radius(radius);
        }

        let mut thumb = thumb_builder.build(wgpu_ctx);
        thumb.transform.offset.x = FlexValue::Fraction(fill_percentage);

        // Create track fill with customizable appearance
        let mut track_fill_builder = BackgroundBuilder::with_color(config.track_fill_color)
            .with_height(config.track_height.clone())
            .with_width(FlexValue::Fraction(fill_percentage))
            .with_fixed_position(Anchor::Left)
            .with_z_index(1);

        if let Some(radius) = config.track_border_radius {
            track_fill_builder = track_fill_builder.with_border_radius(radius);
        }

        let track_fill = track_fill_builder.build(wgpu_ctx);

        // Create track background with customizable appearance
        let mut track_bg_builder = BackgroundBuilder::with_color(config.track_color)
            .with_height(config.track_height)
            .with_fixed_position(Anchor::Left)
            .with_z_index(0);

        if let Some(radius) = config.track_border_radius {
            track_bg_builder = track_bg_builder.with_border_radius(radius);
        }

        let track_background = track_bg_builder.build(wgpu_ctx);

        let slider_data = SliderData {
            value: config.value,
            min: config.min,
            max: config.max,
            step: config.step,
            thumb_id: thumb.id,
            track_fill_id: track_fill.id,
            needs_update: false,
            track_bounds: None,
        };

        container
            .metadata
            .push(ComponentMetaData::SliderData(slider_data));

        container.add_child(track_background);
        container.add_child(track_fill);
        container.add_child(thumb);

        container
    }
}
