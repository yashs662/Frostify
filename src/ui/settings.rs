//! Settings panel — the *interior* of the settings modal.
//!
//! The modal shell (scrim, fade, input-blocking, click-to-dismiss) is the
//! engine's reusable [`frostify_gfx::Overlay`]; this module only builds
//! what goes inside it. `home::build` calls `overlay.render(.., |panel|
//! settings::panel(panel, ..))`.
//!
//! The notable bit is [`toggle_switch`] — the animated on/off switch.

use std::rc::Rc;
use std::time::Duration;

use frostify_gfx::{Align, Computed, Curve, Len, Overlay, Scene, Signal};

use crate::api::Profile;
use crate::ui::icon::{Icon, IconSet};
use crate::ui::tokens as t;

/// Timeline key for the toggle knob slide. Distinct namespace from
/// main.rs's `TWEEN_KEY_*` (0x0001_xxxx) so they can't collide.
const TOGGLE_TWEEN_KEY: u32 = 0x0002_0001;

// Animated toggle dimensions (logical px). The knob slides `TRAVEL` px
// between the two pad-inset ends of the track.
const TOGGLE_W: f32 = 44.0;
const TOGGLE_H: f32 = 24.0;
const TOGGLE_KNOB: f32 = 18.0;
const TOGGLE_PAD: f32 = 3.0;
const TOGGLE_TRAVEL: f32 = TOGGLE_W - TOGGLE_KNOB - 2.0 * TOGGLE_PAD;
/// Off-state track colour — a faint white so the switch reads as a
/// recessed pill before it lights up to the accent.
const TOGGLE_OFF: [f32; 4] = [1.0, 1.0, 1.0, 0.18];
/// Track + knob tween — snappy enough to feel responsive, slow enough to
/// read as motion.
const TOGGLE_MS: u64 = 160;

const PANEL_W: f32 = 420.0;
const SIGN_OUT_W: f32 = 116.0;

pub struct SettingsProps<'a> {
    /// The modal widget — so the header ✕ can close it.
    pub overlay: Overlay,
    pub profile: Option<&'a Profile>,
    pub show_canvas: &'a Signal<bool>,
    pub accent: &'a Signal<[f32; 4]>,
    /// Clear the stored token + return to Login.
    pub sign_out: Rc<dyn Fn()>,
    /// Persist after the canvas toggle flips (debounced prefs save).
    pub on_canvas_change: Rc<dyn Fn()>,
}

/// Build the settings panel interior. Called by the `Overlay` with the
/// panel host as `s`; the overlay supplies the scrim, centring, fade and
/// dismissal, so here we only style + fill the panel.
pub fn panel(s: &mut Scene, icons: &IconSet, p: SettingsProps) {
    s.col("settings_panel")
        .w_px(PANEL_W)
        .rgba(t::PANEL[0], t::PANEL[1], t::PANEL[2], 0.98)
        .radius(t::R_XL)
        .border(1.0, t::BORDER)
        .pad(t::SP_6)
        .gap(t::SP_5)
        .child(|panel| {
            header(panel, icons, p.overlay.clone());
            setting_row(
                panel,
                "Show canvas video",
                "Looping artist visual in the now-playing pane",
                p.show_canvas,
                p.accent,
                p.on_canvas_change.clone(),
            );
            panel
                .rect(())
                .w(Len::Fill)
                .h_px(t::SP_PX)
                .rgba(1.0, 1.0, 1.0, 0.06);
            account(panel, p.profile, p.sign_out.clone());
        });
}

fn header(s: &mut Scene, icons: &IconSet, overlay: Overlay) {
    s.row(())
        .w(Len::Fill)
        .align(Align::Center)
        .child(|h| {
            h.text((), "Settings", t::TEXT_XL).color(t::TEXT);
            h.row(())
                .push_end()
                .w_px(t::SP_8)
                .h_px(t::SP_8)
                .center()
                .hover_opacity(0.7)
                .on_click(move |ctx| overlay.close(ctx.timeline, ctx.now))
                .child(|c| {
                    icons.render(c, Icon::Close, t::ICON_MD, t::TEXT_DIM);
                });
        });
}

/// A labelled row with a trailing animated toggle switch.
fn setting_row(
    s: &mut Scene,
    title: &str,
    subtitle: &str,
    state: &Signal<bool>,
    accent: &Signal<[f32; 4]>,
    on_change: Rc<dyn Fn()>,
) {
    s.row(())
        .w(Len::Fill)
        .align(Align::Center)
        .child(|r| {
            r.col(()).gap(t::SP_0_5).child(|c| {
                c.text((), title, t::TEXT_BASE).color(t::TEXT);
                c.text((), subtitle, t::TEXT_XS).color(t::TEXT_DIM);
            });
            r.row(())
                .push_end()
                .align(Align::Center)
                .child(|ctrl| toggle_switch(ctrl, state, accent, on_change));
        });
}

/// The animated on/off switch. A `knob_t` signal (0..=TRAVEL px) is
/// **seeded to the current state** at build so opening the popup shows
/// the right position instantly — no spurious mount animation. Clicking
/// flips the bound `state` and tweens `knob_t` via the timeline; the
/// knob (spacer-width bind) and track colour (`Computed` over `knob_t`)
/// both follow, so the slide + colour fade are one smooth motion with no
/// scene rebuild. The lib bubbles a click on the knob up to this handler.
fn toggle_switch(
    s: &mut Scene,
    state: &Signal<bool>,
    accent: &Signal<[f32; 4]>,
    on_change: Rc<dyn Fn()>,
) {
    let knob_t = Signal::new(if state.get() { TOGGLE_TRAVEL } else { 0.0 });
    let track_col = Computed::new(
        (knob_t.clone(), accent.clone()),
        |(x, acc)| {
            let f = (x / TOGGLE_TRAVEL).clamp(0.0, 1.0);
            lerp4(TOGGLE_OFF, acc, f)
        },
    );
    let st = state.clone();
    let kt = knob_t.clone();
    s.row(())
        .w_px(TOGGLE_W)
        .h_px(TOGGLE_H)
        .radius(t::R_FULL)
        .color(track_col)
        .align(Align::Center)
        .pad_xy(TOGGLE_PAD, 0.0)
        .on_click(move |ctx| {
            let now_on = !st.get();
            st.set(now_on);
            on_change();
            let target = if now_on { TOGGLE_TRAVEL } else { 0.0 };
            ctx.timeline.start(
                TOGGLE_TWEEN_KEY,
                kt.clone(),
                target,
                Curve::EaseInOut,
                Duration::from_millis(TOGGLE_MS),
                ctx.now,
            );
        })
        .child(|tr| {
            // Spacer whose width tracks `knob_t` (0 → TRAVEL), pushing the
            // knob from the left end to the right as the tween advances.
            tr.rect(())
                .width_px_bind(knob_t.clone())
                .h_px(1.0)
                .rgba(0.0, 0.0, 0.0, 0.0);
            tr.rect(())
                .w_px(TOGGLE_KNOB)
                .h_px(TOGGLE_KNOB)
                .radius(t::R_FULL)
                .rgba(1.0, 1.0, 1.0, 1.0);
        });
}

/// Component-wise linear interpolation between two RGBA colours.
fn lerp4(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

fn account(s: &mut Scene, profile: Option<&Profile>, sign_out: Rc<dyn Fn()>) {
    let name = profile
        .map(|p| p.display_name.as_str())
        .filter(|n| !n.is_empty())
        .unwrap_or("Spotify account");
    s.col(())
        .gap(t::SP_2)
        .child(|acc| {
            acc.text((), "Account", t::TEXT_SM).color(t::TEXT_DIM);
            acc.text((), name, t::TEXT_BASE).color(t::TEXT);
            acc.row(())
                .w_px(SIGN_OUT_W)
                .h_px(t::SP_9)
                .radius(t::R_FULL)
                .border(1.0, t::BORDER)
                .center()
                .hover_color(t::BTN_HOVER)
                .on_click(move |_| sign_out())
                .child(|b| {
                    b.text((), "Sign out", t::TEXT_SM).color(t::TEXT);
                });
        });
}
