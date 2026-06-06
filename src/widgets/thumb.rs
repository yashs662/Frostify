//! Fixed-size square cover thumbnail.

use frostify_gfx::{ImageHandle, Len, Scene, Signal};

use crate::widgets::tokens as t;

/// Renders the resolved cover when the signal carries `Some(handle)`
/// (overlaid on a dim placeholder so the pre-resolve frame doesn't pop).
/// `None` (no signal or unresolved) = placeholder only.
pub fn thumb(s: &mut Scene, art: Option<Signal<Option<ImageHandle>>>, size: f32, radius: f32) {
    s.col(()).w_px(size).h_px(size).child(|b| {
        // Placeholder backdrop — always present so the tile keeps a visible
        // thumb shape while art is loading.
        b.rect(())
            .abs(0.0, 0.0)
            .w(Len::Fill)
            .h(Len::Fill)
            .rgba(t::PLACEHOLDER[0], t::PLACEHOLDER[1], t::PLACEHOLDER[2], 1.0)
            .radius(radius);
        if let Some(sig) = art {
            b.image_bound((), sig)
                .abs(0.0, 0.0)
                .w(Len::Fill)
                .h(Len::Fill)
                .radius(radius);
        }
    });
}
