//! Settings-modal slice.
//!
//! Owns the modal [`Overlay`] (self-contained scrim/fade/input-blocking),
//! the last-measured on-disk cache usage shown in the storage bar, and
//! the cross-thread handoff slot for the (blocking) folder-picker dialog.

use std::cell::Cell;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use frostify_gfx::{Overlay, WakeHandle};

use crate::disk_cache;

pub struct SettingsModel {
    /// The settings modal. Owns its fade opacity + timeline key, blocks
    /// input beneath it, costs nothing when closed.
    pub overlay: Overlay,
    /// Last-measured on-disk cache usage (art vs JSON), shown in the
    /// storage bar. Recomputed on open / clear / relocate.
    pub cache_usage: Cell<disk_cache::CacheUsage>,
    /// Folder picked by the off-thread (blocking) cache-relocation dialog,
    /// awaiting pickup on the UI thread in the frame loop.
    pub pending_cache_dir: Arc<Mutex<Option<PathBuf>>>,
}

impl SettingsModel {
    pub fn new() -> Self {
        Self {
            overlay: Overlay::new(),
            cache_usage: Cell::new(disk_cache::CacheUsage::default()),
            pending_cache_dir: Arc::new(Mutex::new(None)),
        }
    }

    /// Re-measure on-disk cache usage into the slice (settings open /
    /// cache cleared / cache relocated).
    pub fn refresh_usage(&self) {
        self.cache_usage.set(disk_cache::usage());
    }

    /// Wipe every cached file (art, Canvas videos, API JSON) and refresh
    /// the usage bar. Returns bytes freed. Fast — the cache is capped.
    pub fn clear_cache(&self) -> u64 {
        let freed = disk_cache::clear();
        self.refresh_usage();
        freed
    }

    /// Open the native folder picker on a worker thread (the dialog
    /// blocks) and stash the chosen path for the frame loop to apply via
    /// [`take_pending_dir`](Self::take_pending_dir); `wake` re-runs the
    /// loop once a folder is picked.
    pub fn pick_cache_dir(&self, wake: Arc<WakeHandle>) {
        let pending = self.pending_cache_dir.clone();
        std::thread::spawn(move || {
            if let Some(dir) = rfd::FileDialog::new()
                .set_title("Choose cache folder")
                .pick_folder()
            {
                *pending.lock().unwrap() = Some(dir);
                wake.wake();
            }
        });
    }

    /// Take a cache-dir pick stashed by the folder-picker thread, if one
    /// has landed since the last poll.
    pub fn take_pending_dir(&self) -> Option<PathBuf> {
        self.pending_cache_dir.lock().unwrap().take()
    }
}

impl Default for SettingsModel {
    fn default() -> Self {
        Self::new()
    }
}
