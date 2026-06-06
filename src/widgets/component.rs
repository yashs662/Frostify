//! The `Component` trait — a self-rendering UI region.
//!
//! A component holds references to its own model slice(s) and builds
//! itself by reading them. [`Component::view`] is the unit a future
//! hot-reload (subsecond) patches, so it is a plain method that takes
//! its inputs and emits scene nodes — no struct-layout assumptions, no
//! hidden global state. State lives on the host models (Rc-backed
//! signals), so the component instance is a cheap per-build view over
//! them and survives scene rebuilds.
//!
//! This replaces the former pattern of free `fn region(s, &HomeView)`
//! builders fed a wide prop bundle: each component now reads the slices
//! it needs directly off the model.

use frostify_gfx::Scene;

pub trait Component {
    /// Build this component's subtree into `s`, binding to its model
    /// slice(s). Reactive — sets up signal binds + event handlers; it
    /// does not need the per-frame [`crate::cx::Cx`] (that flows to the
    /// handlers it wires, at event/frame time).
    fn view(&self, s: &mut Scene);
}
