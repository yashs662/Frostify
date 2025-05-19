use crate::ui::{color::Color, layout::Anchor};
use std::{f32::consts::PI, time::Duration};

#[derive(Debug, Clone)]
pub struct AnimationConfig {
    pub duration: Duration,
    pub direction: AnimationDirection,
    pub easing: EasingFunction,
    pub animation_type: AnimationType,
    pub when: AnimationWhen,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationDirection {
    Forward,
    Backward,
    Alternate,
    AlternateReverse,
}

impl AnimationDirection {
    pub fn is_flippable(&self) -> bool {
        matches!(
            self,
            AnimationDirection::Alternate | AnimationDirection::AlternateReverse
        )
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationWhen {
    Hover,
    Forever,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimationRange<T> {
    pub from: T,
    pub to: T,
}

impl<T> AnimationRange<T> {
    pub fn new(from: T, to: T) -> Self {
        Self { from, to }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationType {
    Color {
        range: AnimationRange<Color>,
    },
    FrostedGlassTint {
        range: AnimationRange<Color>,
    },
    Scale {
        range: AnimationRange<f32>,
        // TODO: Scale anchor does nothing as of now, scaled position is
        // calculated based on how the parent is placing the scaled object

        // TODO: Scale animation interferes with scrollable containers making them allocate
        // more space than required as the scale is also taken into account even though
        // the object is not scaled at the moment
        anchor: Anchor,
    },
    Opacity {
        range: AnimationRange<f32>,
    },
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
    /// Only used for Forever animations
    /// to determine if the animation is going forward or backward,
    /// when the animation can reverse directions
    pub is_going_forward: bool,
}

impl Animation {
    pub fn new(config: AnimationConfig) -> Self {
        let mut is_going_forward = false;
        if config.direction != AnimationDirection::Backward {
            is_going_forward = true;
        }
        Self {
            config,
            progress: 0.0,
            is_going_forward,
        }
    }
}
