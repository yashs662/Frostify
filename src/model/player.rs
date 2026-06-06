//! Reactive player-chrome state — the now-playing + player-bar surface.
//!
//! Holds the live track title/artist plus the transport toggles
//! (`is_playing`/`shuffle`/`repeat_on`) and `progress`, all as reactive
//! signals so a cluster push updates the chrome via the lib's binds with
//! **no scene rebuild**: title/artist → text binds, `is_playing` →
//! play/pause image bind, `shuffle`/`repeat_on` → tint colour binds,
//! `progress` (0..=1) → bar fill width.
//!
//! This is the UI-facing reactive mirror; the authoritative
//! [`crate::api::CurrentlyPlaying`] snapshot still lives on the app state.

use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

use frostify_gfx::{Curve, Signal, TextSignal, Timeline};

use crate::api::{CurrentlyPlaying, RepeatMode};

pub struct PlayerModel {
    pub title: TextSignal,
    pub artist: TextSignal,
    pub is_playing: Signal<bool>,
    pub shuffle: Signal<bool>,
    pub repeat_on: Signal<bool>,
    /// Playback progress as a fraction of the track (0.0..=1.0).
    pub progress: Signal<f32>,
    /// Current track duration (ms), as a signal so the seek tooltip's
    /// target-timestamp updates on track change without a rebuild.
    pub duration_ms: Signal<f32>,
    /// Seek-bar interaction state (the scrubbable progress bar):
    /// `seek_preview` is the cursor's fraction along the bar (0..=1) on
    /// hover/drag → drives the fill while dragging + the tooltip position
    /// & label; `bar_hovered`/`seeking` gate the tooltip's visibility and
    /// select fill source. Commit happens on release (see `tick_seek`).
    pub seek_preview: Signal<f32>,
    /// Cursor X along the bar in **logical px** (clamped to the lane). Drives
    /// the tooltip's composite layer offset so it follows the cursor without
    /// dirtying layout (see the seek-bar's `layer_offset_x`).
    pub seek_preview_px: Signal<f32>,
    pub bar_hovered: Signal<bool>,
    pub seeking: Signal<bool>,
    /// Formatted target timestamp ("M:SS") at `seek_preview` — the seek
    /// tooltip label. Updated by [`SeekHandle::set_at`].
    pub seek_label: TextSignal,
    /// Live elapsed ("M:SS", left of the bar) + total duration ("M:SS",
    /// right). `elapsed_label` is updated once per second by
    /// [`Self::tick_clock`]; `total_label` on track change in [`Self::sync`].
    pub elapsed_label: TextSignal,
    pub total_label: TextSignal,
    /// Last whole-second elapsed pushed to `elapsed_label`. The clock tick
    /// runs **every frame for the whole song**, so this guard avoids building
    /// a `String` 60×/s for a label that only changes once a second. (The
    /// seek tooltip needs no such guard — it's interaction-only and
    /// `TextSignal::set` already dedups the re-flatten.)
    last_elapsed_secs: Cell<u32>,
    /// Last observed `seeking` value, for release-edge detection in
    /// [`Self::tick_seek`].
    seek_held_last: Cell<bool>,
    /// Authoritative live player snapshot (the latest cluster push). The
    /// reactive signals above are the UI mirror; this is the source of
    /// truth handlers read for the current track/cover/repeat mode.
    pub snapshot: RefCell<Option<CurrentlyPlaying>>,
}

impl PlayerModel {
    /// Seed from the cold-start snapshot (persisted `last_player`), so the
    /// chrome renders the last-played track immediately instead of a dash.
    pub fn seed(title: &str, artist: &str, progress: f32) -> Self {
        Self {
            title: TextSignal::new(title),
            artist: TextSignal::new(artist),
            is_playing: Signal::new(false),
            shuffle: Signal::new(false),
            repeat_on: Signal::new(false),
            progress: Signal::new(progress),
            duration_ms: Signal::new(0.0),
            seek_preview: Signal::new(0.0),
            seek_preview_px: Signal::new(0.0),
            bar_hovered: Signal::new(false),
            seeking: Signal::new(false),
            seek_label: TextSignal::new("0:00"),
            elapsed_label: TextSignal::new("0:00"),
            total_label: TextSignal::new("0:00"),
            last_elapsed_secs: Cell::new(u32::MAX),
            seek_held_last: Cell::new(false),
            snapshot: RefCell::new(None),
        }
    }

    /// Push a live player snapshot into the reactive chrome. All sets are
    /// dedup'd by the signal layer, so a same-track progress tick only
    /// bumps what changed. Progress snaps to the live position, then (if
    /// playing) tweens to 1.0 over the remaining duration so the bar
    /// advances smoothly between cluster pushes; paused stops the tween so
    /// the bar holds.
    pub fn sync(&self, p: &CurrentlyPlaying, tl: &mut Timeline, now: Instant) {
        self.title.set(p.name.as_str());
        self.artist.set(p.artist.as_str());
        self.is_playing.set(p.is_playing);
        self.shuffle.set(p.shuffle);
        self.repeat_on.set(!matches!(p.repeat, RepeatMode::Off));
        self.duration_ms.set(p.duration_ms as f32);
        self.total_label.set(fmt_ms(p.duration_ms as u32).as_str());

        let live = p.live_progress_ms().min(p.duration_ms);
        let frac = if p.duration_ms > 0 {
            live as f32 / p.duration_ms as f32
        } else {
            0.0
        };
        self.progress.set(frac);
        if p.is_playing && p.duration_ms > 0 {
            let remaining = p.duration_ms.saturating_sub(live);
            tl.animate(&self.progress, 1.0, Curve::Linear, Duration::from_millis(remaining), now);
        } else {
            tl.stop_for(&self.progress);
        }
    }

    /// Nothing playing on any device. Don't wipe the chrome to a dash —
    /// keep the last track visible, just mark stopped and freeze the bar.
    pub fn stopped(&self, tl: &mut Timeline) {
        self.is_playing.set(false);
        tl.stop_for(&self.progress);
    }

    // --- optimistic transport (player-bar intents) --------------------
    //
    // Each flips the optimistic UI signal immediately and returns the
    // domain value the host maps to a worker command — the dealer cluster
    // push corrects the real state shortly after. Returning bool/RepeatMode
    // (not a worker command type) keeps this slice free of `worker`.

    /// Toggle play/pause optimistically; returns whether it **was** playing
    /// (so the host sends Pause if it was, else Play).
    pub fn toggle_play(&self) -> bool {
        let was_playing = self.is_playing.get();
        self.is_playing.set(!was_playing);
        was_playing
    }

    /// Toggle shuffle optimistically; returns the new state.
    pub fn toggle_shuffle(&self) -> bool {
        let next = !self.shuffle.get();
        self.shuffle.set(next);
        next
    }

    /// Advance the repeat mode Off → Context → Track → Off (Spotify's
    /// cycle), driven off the live snapshot's actual mode (not just the
    /// `repeat_on` bool) so the three-state cycle is correct. Updates the
    /// optimistic toggle tint and returns the new mode.
    pub fn cycle_repeat(&self) -> RepeatMode {
        let current = self
            .snapshot
            .borrow()
            .as_ref()
            .map(|p| p.repeat)
            .unwrap_or(RepeatMode::Off);
        let next = match current {
            RepeatMode::Off => RepeatMode::Context,
            RepeatMode::Context => RepeatMode::Track,
            RepeatMode::Track => RepeatMode::Off,
        };
        self.repeat_on.set(!matches!(next, RepeatMode::Off));
        next
    }

    /// Refresh the elapsed-time label from the live `progress` fraction.
    /// Called each frame; re-formats (and dirties the text) only when the
    /// whole-second value changes, so the smoothly-tweening progress doesn't
    /// re-flatten the bar every frame.
    pub fn tick_clock(&self) {
        let secs = (self.progress.get() * self.duration_ms.get() / 1000.0).max(0.0) as u32;
        if secs != self.last_elapsed_secs.get() {
            self.last_elapsed_secs.set(secs);
            self.elapsed_label.set(fmt_ms(secs * 1000).as_str());
        }
    }

    // --- seek bar -----------------------------------------------------

    /// A cloneable write-handle for the bar's `'static` event closures
    /// (which can't borrow the model). Updates the preview fraction +
    /// tooltip label from the cursor's position.
    pub fn seek_handle(&self) -> SeekHandle {
        SeekHandle {
            preview: self.seek_preview.clone(),
            preview_px: self.seek_preview_px.clone(),
            label: self.seek_label.clone(),
            duration_ms: self.duration_ms.clone(),
        }
    }

    /// Run from the frame loop. On the release edge of a seek drag (the
    /// `seeking` signal falling true→false), snap the bar to the previewed
    /// position (so it doesn't jump back to the live tween mid-flight) and
    /// return the absolute target position (ms) for the host to dispatch as
    /// a Web API seek. The dealer cluster push re-syncs shortly after.
    pub fn tick_seek(&self, tl: &mut Timeline) -> Option<u32> {
        let held = self.seeking.get();
        let was_held = self.seek_held_last.replace(held);
        if was_held && !held {
            let frac = self.seek_preview.get().clamp(0.0, 1.0);
            let dur = self.duration_ms.get();
            if dur > 0.0 {
                self.progress.set(frac);
                tl.stop_for(&self.progress);
                return Some((frac * dur) as u32);
            }
        }
        None
    }
}

/// Cloneable seek-bar write handle (see [`PlayerModel::seek_handle`]).
#[derive(Clone)]
pub struct SeekHandle {
    preview: Signal<f32>,
    preview_px: Signal<f32>,
    label: TextSignal,
    duration_ms: Signal<f32>,
}

impl SeekHandle {
    /// Set the preview from the cursor's X relative to the bar's left
    /// (`x_rel`, logical px) given the lane width (`lane_w`, px). Updates the
    /// fraction (0..=1, for the fill), the px offset (for the tooltip's
    /// composite layer position), and the timestamp label. `TextSignal::set`
    /// dedups, so a same-second label is a no-op that doesn't dirty the tree;
    /// the only per-move cost is building the short `M:SS` string, and only
    /// while the user is actively hovering the bar.
    pub fn set_at(&self, x_rel: f32, lane_w: f32) {
        let w = lane_w.max(1.0);
        let frac = (x_rel / w).clamp(0.0, 1.0);
        self.preview.set(frac);
        self.preview_px.set(x_rel.clamp(0.0, w));
        let ms = (frac * self.duration_ms.get()) as u32;
        self.label.set(fmt_ms(ms).as_str());
    }
}

/// Format a millisecond position as `M:SS` for the seek tooltip.
fn fmt_ms(ms: u32) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}
