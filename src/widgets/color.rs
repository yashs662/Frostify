//! Accent-derived colour utilities.

use frostify_gfx::{Computed, Signal};

/// Foreground colour (icon/text) that contrasts with the live accent: a
/// dark accent gets light text, a light accent gets dark text. Spotify's
/// `color_dark` accent is dark-vibrant so this is usually white, but it
/// keeps contrast correct for any accent (so accent buttons never read as
/// black-on-dark). Reactive — follows the accent crossfade.
pub fn accent_fg(accent: &Signal<[f32; 4]>) -> Computed<[f32; 4]> {
    Computed::new((accent.clone(),), |(a,)| {
        // Perceived luminance (Rec. 601 weights). Below the threshold the
        // accent is dark → light foreground; above → dark foreground.
        let lum = 0.299 * a[0] + 0.587 * a[1] + 0.114 * a[2];
        if lum < 0.6 {
            [1.0, 1.0, 1.0, 1.0]
        } else {
            [0.08, 0.08, 0.08, 1.0]
        }
    })
}
