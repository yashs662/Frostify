//! User preferences — persisted across sessions as JSON in the OS
//! config directory.
//!
//! Schema is versioned (`version` field) so future migrations can detect
//! and adapt older files. Every field carries `#[serde(default)]` so
//! adding a new field is forward-compatible: an old preferences file
//! missing the field deserializes cleanly, the new field picks up its
//! Default value, and the next save writes the upgraded shape.
//!
//! Loading is fail-soft: any error (missing file, malformed JSON,
//! permission denied) yields [`UserPreferences::default`]. Saving is
//! best-effort — a write failure is logged but does not propagate.
//!
//! Scope today: panel sizes, window geometry, audio prefs. Extend by
//! adding a field-with-`#[serde(default)]` to [`UserPreferences`] or
//! one of its child structs; no migration needed for additive changes.

use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Bump on any **incompatible** schema change (renamed fields, removed
/// fields with semantic load-bearers, changed types). Additive
/// changes don't need a bump — `#[serde(default)]` covers them.
pub const SCHEMA_VERSION: u32 = 1;

/// Top-level preferences. Every nested field defaults so partial /
/// older JSON files load cleanly.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserPreferences {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub panels: PanelPrefs,
    #[serde(default)]
    pub window: WindowPrefs,
    #[serde(default)]
    pub audio: AudioPrefs,
    /// Snapshot of the last track that was playing when the app exited.
    /// Restored on next launch as the "what was I listening to?" hint —
    /// populates the player chrome before any live cluster push lands so
    /// the UI isn't blank during the seconds between session-connect and
    /// the first dealer state. Overwritten the moment a real cluster
    /// update arrives.
    #[serde(default)]
    pub last_player: Option<StoredPlayer>,
    /// Show the looping Canvas video in the now-playing pane when a track
    /// has one. The playback pipeline isn't built yet; this persists the
    /// user's choice so it's honoured the moment canvas support lands.
    #[serde(default = "default_show_canvas")]
    pub show_canvas: bool,
    /// User-chosen cache directory (parent of `frostify/art` + `json`).
    /// `None` = the OS cache dir. Lets the user relocate the on-disk cache
    /// (album art, Canvas videos, API JSON) to another drive/folder.
    #[serde(default)]
    pub cache_dir: Option<String>,
}

fn default_version() -> u32 {
    SCHEMA_VERSION
}

fn default_show_canvas() -> bool {
    true
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            version: SCHEMA_VERSION,
            panels: PanelPrefs::default(),
            window: WindowPrefs::default(),
            audio: AudioPrefs::default(),
            last_player: None,
            show_canvas: default_show_canvas(),
            cache_dir: None,
        }
    }
}

/// Minimal snapshot of the live `CurrentlyPlaying` — just the fields
/// the player chrome reads. `is_playing` is intentionally **not**
/// persisted: the app can't keep playing while closed, and a stored
/// `true` would make the cold-start UI lie about playback state.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredPlayer {
    pub track_id: String,
    pub name: String,
    pub artist: String,
    pub album_image_url: Option<String>,
    pub progress_ms: u64,
    pub duration_ms: u64,
}

/// Sidebar + now-playing pane widths in **logical** pixels. `0`
/// represents a fully collapsed (hidden) panel — the splitter can
/// re-open it.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PanelPrefs {
    #[serde(default = "default_sidebar_w")]
    pub sidebar_w: f32,
    #[serde(default = "default_now_playing_w")]
    pub now_playing_w: f32,
}

fn default_sidebar_w() -> f32 {
    320.0
}
fn default_now_playing_w() -> f32 {
    340.0
}

impl Default for PanelPrefs {
    fn default() -> Self {
        Self {
            sidebar_w: default_sidebar_w(),
            now_playing_w: default_now_playing_w(),
        }
    }
}

/// Snap a stored panel width into a known-good state. Defends against:
/// - hand-edited / corrupted JSON values outside `[min, max]`
/// - schema additions where `min`/`max` moved past an existing save
/// - off-by-one drift from float round-trips
///
/// Below the midpoint between `collapsed` and `min`, snap **down** to
/// `collapsed` (preserving the user's intent to hide the panel). Above
/// it, clamp to `[min, max]`. A panel without a collapsed state can pass
/// `collapsed = min` to disable the snap entirely.
pub fn clamp_panel_width(w: f32, min: f32, max: f32, collapsed: f32) -> f32 {
    let midpoint = (collapsed + min) * 0.5;
    if w < midpoint {
        collapsed
    } else {
        w.clamp(min, max)
    }
}

/// Last known window geometry — used to restore size + position on
/// launch. All fields optional; missing → winit picks a default.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct WindowPrefs {
    /// Logical-px inner size. `None` → fall back to the hardcoded
    /// default in `main.rs`.
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Outer position in screen-space px. `None` → OS picks.
    pub x: Option<i32>,
    pub y: Option<i32>,
    #[serde(default)]
    pub maximized: bool,
}

/// Playback / audio preferences. Hooked into the player layer (libre-
/// spot mixer + crossfade timeline) — values applied at session start.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AudioPrefs {
    /// Master volume, 0.0..=1.0. Applied to the librespot soft mixer.
    #[serde(default = "default_volume")]
    pub volume: f32,
    /// Crossfade between consecutive tracks, in ms. `0` disables.
    #[serde(default)]
    pub crossfade_ms: u32,
    /// Bitrate tier — librespot accepts 96 / 160 / 320 kbps.
    #[serde(default)]
    pub quality: AudioQuality,
}

fn default_volume() -> f32 {
    0.8
}

impl Default for AudioPrefs {
    fn default() -> Self {
        Self {
            volume: default_volume(),
            crossfade_ms: 0,
            quality: AudioQuality::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AudioQuality {
    Low,
    #[default]
    Normal,
    High,
}

impl UserPreferences {
    /// Read + parse the JSON file. Returns [`Self::default`] on any
    /// failure (missing file, malformed JSON, permission denied) so a
    /// fresh install / corrupted state always boots cleanly.
    pub fn load() -> Self {
        let Some(path) = preferences_path() else {
            return Self::default();
        };
        match fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<Self>(&text) {
                Ok(prefs) => {
                    log::info!("loaded user prefs from {}", path.display());
                    prefs
                }
                Err(e) => {
                    log::warn!("malformed prefs at {}: {e} — using defaults", path.display());
                    Self::default()
                }
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(e) => {
                log::warn!("failed to read prefs at {}: {e} — using defaults", path.display());
                Self::default()
            }
        }
    }

    /// Pretty-print to the on-disk JSON file. Creates the parent dir
    /// if missing. Best-effort — caller logs but does not propagate.
    pub fn save(&self) -> io::Result<()> {
        let Some(path) = preferences_path() else {
            return Err(io::Error::other("no config dir"));
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::other(format!("serialize: {e}")))?;
        fs::write(&path, json)?;
        Ok(())
    }
}

/// `<config_dir>/frostify/preferences.json`. `None` if the OS doesn't
/// expose a config dir (extremely rare; e.g. some headless containers).
pub fn preferences_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("frostify").join("preferences.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trip_through_json() {
        let prefs = UserPreferences::default();
        let json = serde_json::to_string_pretty(&prefs).unwrap();
        let back: UserPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, SCHEMA_VERSION);
        assert_eq!(back.panels.sidebar_w, 320.0);
        assert_eq!(back.audio.quality, AudioQuality::Normal);
    }

    #[test]
    fn missing_fields_use_defaults() {
        // Old-shape file with only one nested field — additive forward
        // compat. Every other field falls back to its Default.
        let json = r#"{"panels": {"sidebar_w": 280.0}}"#;
        let prefs: UserPreferences = serde_json::from_str(json).unwrap();
        assert_eq!(prefs.panels.sidebar_w, 280.0);
        assert_eq!(prefs.panels.now_playing_w, 340.0, "default kicks in");
        assert_eq!(prefs.audio.volume, 0.8);
        assert_eq!(prefs.version, SCHEMA_VERSION);
    }

    #[test]
    fn empty_object_yields_full_defaults() {
        let prefs: UserPreferences = serde_json::from_str("{}").unwrap();
        assert_eq!(prefs.panels.sidebar_w, 320.0);
        assert_eq!(prefs.window.width, None);
        assert!(!prefs.window.maximized);
    }
}
