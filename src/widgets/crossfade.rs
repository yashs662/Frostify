//! Two-layer album-art crossfade widget.

use frostify_gfx::{Computed, ImageHandle, Len, Scene, Signal};

use crate::widgets::tokens as t;

/// Fully-opaque white tint for the outgoing (under) crossfade layer.
pub const OPAQUE_TINT: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

/// Incoming-layer tint: white with alpha rising 0 → 1 as the crossfade
/// advances, so the new cover fades in *over* the outgoing one.
///
/// The outgoing layer underneath stays fully opaque (a plain `[1,1,1,1]`
/// literal, no bind). Crucially this is NOT a symmetric dual fade: if both
/// layers cross-faded (prev `1-t`, curr `t`) their combined coverage dips
/// to ~75% at the midpoint and the dark glass backdrop bleeds through — a
/// murky mid-transition. Holding the outgoing layer opaque keeps full
/// coverage throughout, so it's a clean A→B dissolve. Painter order
/// (outgoing declared first) guarantees incoming draws on top.
pub fn fade_in_alpha(crossfade_t: &Signal<f32>) -> Computed<[f32; 4]> {
    Computed::new((crossfade_t.clone(),), move |(t,)| {
        [1.0, 1.0, 1.0, t.clamp(0.0, 1.0)]
    })
}

/// Two stacked album-art layers that crossfade on track change, sized to
/// fill the parent box. Reuses the backdrop's `crossfade_t` + prev/curr
/// handles so panel art dissolves in lockstep with the ambient backdrop
/// instead of snapping. Dim placeholder when neither handle resolves. Both
/// layers are `abs(0,0)` so they overlap — the parent must have a definite
/// size for `Fill` to resolve against.
pub fn crossfaded_art(
    c: &mut Scene,
    prev: &Signal<Option<ImageHandle>>,
    curr: &Signal<Option<ImageHandle>>,
    crossfade_t: &Signal<f32>,
    radius: f32,
) {
    // Layer 0: dim placeholder shown until any cover resolves (the image
    // layers above render nothing while their signal is None).
    c.rect(())
        .abs(0.0, 0.0)
        .w(Len::Fill)
        .h(Len::Fill)
        .rgba(t::PLACEHOLDER[0], t::PLACEHOLDER[1], t::PLACEHOLDER[2], 1.0)
        .radius(radius);
    // Layer 1: outgoing cover, opaque. Layer 2: incoming, fading in. Both
    // bound to the shared backdrop signals — swap rebuild-free.
    c.image_bound((), prev.clone())
        .abs(0.0, 0.0)
        .w(Len::Fill)
        .h(Len::Fill)
        .radius(radius)
        .color(OPAQUE_TINT);
    c.image_bound((), curr.clone())
        .abs(0.0, 0.0)
        .w(Len::Fill)
        .h(Len::Fill)
        .radius(radius)
        .color(fade_in_alpha(crossfade_t));
}
