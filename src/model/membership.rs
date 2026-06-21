//! Playlist-membership slice — which of the user's playlists (plus Liked
//! Songs) contain a track.
//!
//! Spotify has no reverse lookup (track → playlists), so the worker scans
//! every editable playlist once, builds a `track_uri → [playlist_id]`
//! index, caches it to disk with a 6h TTL, and updates it incrementally on
//! add/remove. This model is the UI-facing *view*: the picker's playlist
//! list, the current track's membership (drives the heart + checkboxes),
//! and the picker popup state. The heavy index stays in the worker.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;

use frostify_gfx::{Overlay, Signal, TextSignal};
use serde::{Deserialize, Serialize};

/// One editable playlist (picker row + name lookup).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MembershipPlaylist {
    pub id: String,
    pub name: String,
}

/// Disk-persisted membership snapshot — the worker's canonical copy. The UI
/// only ever sees `playlists` + per-track lookups derived from `index`.
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct MembershipSnapshot {
    /// Editable playlists scanned, in library order.
    pub playlists: Vec<MembershipPlaylist>,
    /// `spotify:track:… → [playlist_id]` — which playlists contain a track.
    pub index: std::collections::HashMap<String, Vec<String>>,
}

/// The track the picker popup acts on.
#[derive(Clone, Default)]
pub struct MembershipTarget {
    /// `spotify:track:…` URI (playlist add/remove).
    pub uri: String,
    /// Bare hex id (Liked-Songs save/unsave).
    pub id: String,
    /// Track + artist for the popup header.
    pub name: String,
    pub artist: String,
}

pub struct MembershipModel {
    /// Editable playlists — picker rows + id→name lookup. From the worker's
    /// `MembershipLoaded`.
    pub playlists: RefCell<Vec<MembershipPlaylist>>,
    /// Index loaded/built this session (picker shows a spinner until then).
    pub ready: Cell<bool>,
    /// The current track's playlist ids — drives the picker checkboxes.
    pub current: RefCell<HashSet<String>>,
    /// Current track is in ≥1 playlist; combined with `liked` for the heart
    /// fill.
    pub in_playlist: Signal<bool>,
    /// Heart tooltip: the playlist names (+ "Liked Songs") the current track
    /// belongs to, or empty.
    pub hint: TextSignal,
    /// The picker popup's scrim/fade/dismiss owner (same primitive as the
    /// devices / settings popups).
    pub overlay: Overlay,
    /// The track the open picker acts on.
    pub target: RefCell<MembershipTarget>,
}

impl MembershipModel {
    pub fn new() -> Self {
        Self {
            playlists: RefCell::default(),
            ready: Cell::new(false),
            current: RefCell::default(),
            in_playlist: Signal::new(false),
            hint: TextSignal::new(""),
            overlay: Overlay::new(),
            target: RefCell::default(),
        }
    }

    /// Point the picker at a track (called when the like icon opens it).
    pub fn set_target(&self, target: MembershipTarget) {
        *self.target.borrow_mut() = target;
    }

    /// Apply the loaded/refreshed playlist list (the index landed).
    pub fn set_playlists(&self, playlists: Vec<MembershipPlaylist>) {
        *self.playlists.borrow_mut() = playlists;
        self.ready.set(true);
    }

    /// Replace the current track's membership (from the worker's lookup) and
    /// refresh the derived heart state + tooltip.
    pub fn set_current(&self, ids: Vec<String>, liked: bool) {
        let set: HashSet<String> = ids.into_iter().collect();
        self.in_playlist.set(!set.is_empty());
        *self.current.borrow_mut() = set;
        self.rebuild_hint(liked);
    }

    /// Optimistically flip one playlist's membership for the current track
    /// (the picker checkbox), refreshing the heart + tooltip.
    pub fn toggle_local(&self, playlist_id: &str, add: bool, liked: bool) {
        {
            let mut cur = self.current.borrow_mut();
            if add {
                cur.insert(playlist_id.to_string());
            } else {
                cur.remove(playlist_id);
            }
            self.in_playlist.set(!cur.is_empty());
        }
        self.rebuild_hint(liked);
    }

    /// Whether the current track is in playlist `id` (picker checkbox state).
    pub fn contains(&self, id: &str) -> bool {
        self.current.borrow().contains(id)
    }

    /// Rebuild the heart tooltip from the current membership + liked flag.
    /// "Liked Songs, Chill, Focus" — or empty when in nothing.
    pub fn rebuild_hint(&self, liked: bool) {
        let cur = self.current.borrow();
        let names = self.playlists.borrow();
        let mut parts: Vec<&str> = Vec::new();
        if liked {
            parts.push("Liked Songs");
        }
        for p in names.iter() {
            if cur.contains(&p.id) {
                parts.push(p.name.as_str());
            }
        }
        self.hint.set(parts.join(", ").as_str());
    }
}

impl Default for MembershipModel {
    fn default() -> Self {
        Self::new()
    }
}
