use crate::{
    ui::{
        color::Color,
        component::{
            BackgroundColorConfig, Component, ComponentConfig, ComponentMetaData, ComponentType,
        },
        layout::{Anchor, Position},
    },
    wgpu_ctx::WgpuCtx,
};
use log::{debug, error};
use std::{f32::consts::PI, time::Duration};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AnimationConfig {
    pub duration: Duration,
    pub direction: AnimationDirection,
    pub easing: EasingFunction,
    pub animation_type: AnimationType,
    pub when: AnimationWhen,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AnimationDirection {
    Forward,
    Backward,
    Alternate,
    AlternateReverse,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AnimationWhen {
    Hover,
    OnClick,
    Forever,
}

#[derive(Debug, Clone)]
pub enum AnimationType {
    Color { from: Color, to: Color },
    FrostedGlassTint { from: Color, to: Color },
    // Scale { from: f32, to: f32 },
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingFunction {
    Linear,
    EaseInSine,
    EaseInQuad,
    EaseInCubic,
    EaseInQuart,
    EaseInQuint,
    EaseInExpo,
    EaseInCirc,
    EaseInBack,
    EaseInElastic,
    EaseInBounce,
    EaseOutSine,
    EaseOutQuad,
    EaseOutCubic,
    EaseOutQuart,
    EaseOutQuint,
    EaseOutExpo,
    EaseOutCirc,
    EaseOutBack,
    EaseOutBounce,
    EaseOutElastic,
    EaseInOutSine,
    EaseInOutQuad,
    EaseInOutCubic,
    EaseInOutQuart,
    EaseInOutQuint,
    EaseInOutExpo,
    EaseInOutCirc,
    EaseInOutBack,
    EaseInOutBounce,
    EaseInOutElastic,
}

impl EasingFunction {
    pub fn compute(&self, t: f32) -> f32 {
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseInSine => 1.0 - (t * PI / 2.0).cos(),
            EasingFunction::EaseInQuad => t * t,
            EasingFunction::EaseInCubic => t * t * t,
            EasingFunction::EaseInQuart => t * t * t * t,
            EasingFunction::EaseInQuint => t * t * t * t * t,
            EasingFunction::EaseInExpo => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0f32.powf(10.0 * t - 10.0)
                }
            }
            EasingFunction::EaseInCirc => 1.0 - (1.0 - t * t).sqrt(),
            EasingFunction::EaseInBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            EasingFunction::EaseInElastic => {
                let c4 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -2.0f32.powf(10.0 * t - 10.0) * (t * 10.0 - 10.75).sin() * c4
                }
            }
            EasingFunction::EaseInBounce => {
                let t = 1.0 - t;
                let n1 = 7.5625;
                let d1 = 2.75;
                let computed = if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    n1 * (t - 1.5 / d1) * (t - 1.5 / d1) + 0.75
                } else if t < 2.5 / d1 {
                    n1 * (t - 2.25 / d1) * (t - 2.25 / d1) + 0.9375
                } else {
                    n1 * (t - 2.625 / d1) * (t - 2.625 / d1) + 0.984375
                };
                1.0 - computed
            }
            EasingFunction::EaseOutSine => ((t * PI) / 2.0).sin(),
            EasingFunction::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            EasingFunction::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            EasingFunction::EaseOutQuint => 1.0 - (1.0 - t).powi(5),
            EasingFunction::EaseOutExpo => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - 2.0f32.powf(-10.0 * t)
                }
            }
            EasingFunction::EaseOutCirc => (1.0 - ((t - 1.0) * (t - 1.0))).sqrt(),
            EasingFunction::EaseOutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            EasingFunction::EaseOutBounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    n1 * (t - 1.5 / d1) * (t - 1.5 / d1) + 0.75
                } else if t < 2.5 / d1 {
                    n1 * (t - 2.25 / d1) * (t - 2.25 / d1) + 0.9375
                } else {
                    n1 * (t - 2.625 / d1) * (t - 2.625 / d1) + 0.984375
                }
            }
            EasingFunction::EaseOutElastic => {
                let c4 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    2.0f32.powf(-10.0 * t) * (t * 10.0 - 0.75).sin() * c4 + 1.0
                }
            }
            EasingFunction::EaseInOutSine => -((t * PI).cos() - 1.0) / 2.0,
            EasingFunction::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            EasingFunction::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            EasingFunction::EaseInOutQuart => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }
            EasingFunction::EaseInOutQuint => {
                if t < 0.5 {
                    16.0 * t * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
                }
            }
            EasingFunction::EaseInOutExpo => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    2.0f32.powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - 2.0f32.powf(-20.0 * t + 10.0)) / 2.0
                }
            }
            EasingFunction::EaseInOutCirc => {
                if t < 0.5 {
                    (1.0 - (1.0 - 4.0 * t * t).sqrt()) / 2.0
                } else {
                    ((4.0 * t - 3.0) * (4.0 * t - 1.0)).sqrt() / 2.0 + 0.5
                }
            }
            EasingFunction::EaseInOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if t < 0.5 {
                    (c2 + 1.0) * t * t * t - c2 * t * t
                } else {
                    (c2 + 1.0) * (t - 1.0).powi(3) + c2 * (t - 1.0).powi(2) + 1.0
                }
            }
            EasingFunction::EaseInOutBounce => {
                if t < 0.5 {
                    (1.0 - EasingFunction::EaseOutBounce.compute(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + EasingFunction::EaseOutBounce.compute(2.0 * t - 1.0)) / 2.0
                }
            }
            EasingFunction::EaseInOutElastic => {
                let c5 = (2.0 * PI) / 4.5;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    -(2.0f32.powf(20.0 * t - 10.0) * (t * 20.0 - 11.125).sin() * c5) / 2.0
                } else {
                    (2.0f32.powf(-20.0 * t + 10.0) * (t * 20.0 - 11.125).sin() * c5) / 2.0 + 1.0
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub config: AnimationConfig,
    pub progress: f32,
}

impl Animation {
    pub fn new(config: AnimationConfig) -> Self {
        Self {
            config,
            progress: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32, forward: bool) -> f32 {
        // Calculate the delta based on frame time and duration
        let delta = delta_time / self.config.duration.as_secs_f32();

        // Update progress based on animation type
        match self.config.direction {
            AnimationDirection::Forward => {
                if forward {
                    // Animate in slowly
                    self.progress = (self.progress + delta).min(1.0);
                } else {
                    // Instant out
                    self.progress = 0.0;
                }
            }
            AnimationDirection::Backward => {
                if forward {
                    // Instant in
                    self.progress = 1.0;
                } else {
                    // Animate out slowly
                    self.progress = (self.progress - delta).max(0.0);
                }
            }
            AnimationDirection::Alternate => {
                self.progress = if forward {
                    (self.progress + delta).min(1.0)
                } else {
                    (self.progress - delta).max(0.0)
                };
            }
            AnimationDirection::AlternateReverse => {
                self.progress = if forward {
                    (self.progress - delta).max(0.0)
                } else {
                    (self.progress + delta).min(1.0)
                };
            }
        }

        self.config.easing.compute(self.progress)
    }

    pub fn configure_component(&self, component: &mut Component, wgpu_ctx: &mut WgpuCtx) {
        match self.config.animation_type {
            AnimationType::Color { from, to: _ } => {
                // Find existing background color component or create new one
                let bg_id = if let Some((id, _)) = component
                    .children_ids
                    .iter()
                    .find(|(_, t)| matches!(t, ComponentType::BackgroundColor))
                {
                    *id
                } else {
                    debug!(
                        "Component Doesn't have a background color component, creating one for animated color"
                    );
                    let bg_id = Uuid::new_v4();
                    let mut bg = Component::new(bg_id, ComponentType::BackgroundColor);
                    bg.transform.position_type = Position::Fixed(Anchor::Center);
                    bg.set_debug_name("Animated Color Background");
                    bg.set_z_index(0);
                    bg.configure(
                        ComponentConfig::BackgroundColor(BackgroundColorConfig { color: from }),
                        wgpu_ctx,
                    );
                    component.add_child_to_front(bg);
                    bg_id
                };

                // Add animation to the background component
                if let Some(ComponentMetaData::ChildComponents(children)) = component
                    .metadata
                    .iter_mut()
                    .find(|m| matches!(m, ComponentMetaData::ChildComponents(_)))
                {
                    if let Some(bg_component) = children.iter_mut().find(|c| c.id == bg_id) {
                        bg_component.animations.push(self.clone());
                    }
                }
            }
            AnimationType::FrostedGlassTint { from: _, to: _ } => {
                if !component
                    .children_ids
                    .iter()
                    .any(|(_, t)| matches!(t, ComponentType::FrostedGlass))
                {
                    error!(
                        "added frosted glass tint animation but the component doesn't have a frosted glass tint component"
                    );
                }
            }
        }
    }
}
