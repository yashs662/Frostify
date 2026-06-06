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
use crate::disk_cache::{self, CacheUsage};
use crate::model::{BackdropModel, CanvasModel, SettingsModel};
use crate::widgets::component::Component;
use crate::widgets::icon::{Icon, IconSet};
use crate::widgets::tokens as t;

/// Colour of the album-art segment in the cache usage bar.
const CACHE_ART_COL: [f32; 4] = [0.36, 0.7, 0.95, 1.0];
/// Colour of the Canvas-video segment in the cache usage bar.
const CACHE_CANVAS_COL: [f32; 4] = [0.78, 0.5, 0.95, 1.0];
/// Colour of the API-JSON segment in the cache usage bar.
const CACHE_JSON_COL: [f32; 4] = [0.55, 0.82, 0.55, 1.0];

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

/// The settings modal — a [`Component`]. Reads its toggle/accent/cache
/// slices off the models directly; owns the [`Overlay`] render wrapper
/// (the overlay supplies the scrim, centring, fade and dismissal, so the
/// body here only styles + fills the panel). Costs nothing when closed.
pub struct SettingsPanel<'a> {
    pub settings: &'a SettingsModel,
    pub canvas: &'a CanvasModel,
    pub backdrop: &'a BackdropModel,
    pub profile: Option<&'a Profile>,
    pub icons: &'a Rc<IconSet>,
    /// Clear the stored token + return to Login.
    pub sign_out: Rc<dyn Fn()>,
    /// Persist after the canvas toggle flips (debounced prefs save).
    pub on_canvas_change: Rc<dyn Fn()>,
    /// Delete all cached files.
    pub on_clear_cache: Rc<dyn Fn()>,
    /// Open a folder picker to relocate the cache.
    pub on_change_cache_dir: Rc<dyn Fn()>,
}

impl Component for SettingsPanel<'_> {
    fn view(&self, s: &mut Scene) {
        let icons = self.icons;
        // Measured on settings-open (a dir walk), not per build.
        let cache_usage = self.settings.cache_usage.get();
        let cache_path = disk_cache::root_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        self.settings.overlay.render(s, t::SCRIM, |host| {
            host.col("settings_panel")
                .w_px(PANEL_W)
                .rgba(t::PANEL[0], t::PANEL[1], t::PANEL[2], 0.98)
                .radius(t::R_XL)
                .border(1.0, t::BORDER)
                .pad(t::SP_6)
                .gap(t::SP_5)
                .child(|panel| {
                    header(panel, icons, self.settings.overlay.clone());
                    setting_row(
                        panel,
                        "Show canvas video",
                        "Looping artist visual in the now-playing pane",
                        &self.canvas.show,
                        &self.backdrop.accent,
                        self.on_canvas_change.clone(),
                    );
                    panel.rect(()).w(Len::Fill).h_px(t::SP_PX).rgba(1.0, 1.0, 1.0, 0.06);
                    cache_section(
                        panel,
                        cache_usage,
                        &cache_path,
                        self.on_clear_cache.clone(),
                        self.on_change_cache_dir.clone(),
                    );
                    panel.rect(()).w(Len::Fill).h_px(t::SP_PX).rgba(1.0, 1.0, 1.0, 0.06);
                    account(panel, self.profile, self.sign_out.clone());
                });
        });
    }
}

/// Human-readable byte size (e.g. `1.2 GB`, `340 MB`, `12 KB`).
fn fmt_bytes(b: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let f = b as f64;
    if f >= GB {
        format!("{:.2} GB", f / GB)
    } else if f >= MB {
        format!("{:.1} MB", f / MB)
    } else if f >= KB {
        format!("{:.0} KB", f / KB)
    } else {
        format!("{b} B")
    }
}

/// Cache management: a usage-breakdown bar (album-art/Canvas vs API JSON),
/// the on-disk location with a relocate button, and a clear-cache button.
fn cache_section(
    s: &mut Scene,
    usage: CacheUsage,
    path: &str,
    on_clear: Rc<dyn Fn()>,
    on_change_dir: Rc<dyn Fn()>,
) {
    let total = usage.total();
    let frac = |b: u64| if total > 0 { b as f32 / total as f32 } else { 0.0 };
    // Non-zero segments in draw order, for the proportional bar. End caps
    // are rounded by rounding the first segment's left + last's right.
    let segments: Vec<(f32, [f32; 4])> = [
        (frac(usage.art), CACHE_ART_COL),
        (frac(usage.canvas), CACHE_CANVAS_COL),
        (frac(usage.json), CACHE_JSON_COL),
    ]
    .into_iter()
    .filter(|(f, _)| *f > 0.0)
    .collect();
    let total_label = fmt_bytes(total);
    let art_label = format!("Album art  {}", fmt_bytes(usage.art));
    let canvas_label = format!("Canvas  {}", fmt_bytes(usage.canvas));
    let json_label = format!("Metadata  {}", fmt_bytes(usage.json));
    let path = path.to_string();
    s.col(()).w(Len::Fill).gap(t::SP_2).child(move |c| {
        c.row(())
            .w(Len::Fill)
            .align(Align::Center)
            .child(|h| {
                h.text((), "Storage", t::TEXT_SM).color(t::TEXT_DIM);
                h.row(())
                    .push_end()
                    .child(|e| {
                        e.text((), &total_label, t::TEXT_SM).color(t::TEXT);
                    });
            });
        // Proportional usage bar. Coloured segments fill it by each
        // category's share; the rounded track clips them (rounded overflow
        // clipping), so the whole bar reads as a clean pill with rounded
        // caps regardless of how thin the end segment is.
        c.row(())
            .w(Len::Fill)
            .h_px(t::SP_2)
            .radius(t::R_FULL)
            .rgba(1.0, 1.0, 1.0, 0.08)
            .overflow(frostify_gfx::Overflow::Hidden, frostify_gfx::Overflow::Hidden)
            .child(move |bar| {
                for (f, col) in &segments {
                    bar.rect(()).w(Len::Pct(*f)).h(Len::Fill).color(*col);
                }
            });
        // Legend.
        c.row(())
            .w(Len::Fill)
            .gap(t::SP_4)
            .child(move |lg| {
                legend_dot(lg, CACHE_ART_COL, &art_label);
                legend_dot(lg, CACHE_CANVAS_COL, &canvas_label);
                legend_dot(lg, CACHE_JSON_COL, &json_label);
            });
        // Location + relocate.
        c.row(())
            .w(Len::Fill)
            .align(Align::Center)
            .gap(t::SP_2)
            .child(move |loc| {
                loc.col(())
                    .child(|p| {
                        p.text((), "Location", t::TEXT_XS).color(t::TEXT_DIM);
                        p.text((), &path, t::TEXT_XS).color(t::TEXT).max_width_px(240.0);
                    });
                loc.row(())
                    .push_end()
                    .h_px(t::SP_8)
                    .pad_xy(t::SP_3, 0.0)
                    .radius(t::R_FULL)
                    .border(1.0, t::BORDER)
                    .center()
                    .hover_color(t::BTN_HOVER)
                    .on_click(move |_| on_change_dir())
                    .child(|b| {
                        b.text((), "Change\u{2026}", t::TEXT_SM).color(t::TEXT);
                    });
            });
        // Clear — full width to match the section.
        c.row(())
            .w(Len::Fill)
            .h_px(t::SP_9)
            .radius(t::R_FULL)
            .border(1.0, t::BORDER)
            .center()
            .hover_color(t::BTN_HOVER)
            .on_click(move |_| on_clear())
            .child(|b| {
                b.text((), "Clear cache", t::TEXT_SM).color(t::TEXT);
            });
    });
}

/// A small coloured dot + label, for the cache-bar legend.
fn legend_dot(s: &mut Scene, color: [f32; 4], label: &str) {
    s.row(())
        .align(Align::Center)
        .gap(t::SP_1)
        .child(|d| {
            d.rect(()).w_px(t::SP_2).h_px(t::SP_2).radius(t::R_FULL).color(color);
            d.text((), label, t::TEXT_XS).color(t::TEXT_DIM);
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
            ctx.timeline.animate(
                &kt,
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
