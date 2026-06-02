//! On-disk cache for fetched album-art bytes.
//!
//! Spotify album covers are immutable for a given image id, so the raw
//! JPEG/PNG bytes can be cached on disk indefinitely — re-fetching the
//! same cover hammers the CDN for nothing and re-introduces the
//! track-change "stuck on old art" window while the network round-trips.
//! Keyed by [`crate::album_art::cache_key`] (the trailing hex of the
//! `i.scdn.co/image/<hex>` URL), which is filesystem-safe.
//!
//! Stored bytes are the *encoded* image (not decoded RGBA): smaller on
//! disk and the decode is cheap enough to redo on load. Entries carry an
//! mtime-based TTL and the directory is held under a rough LRU size cap;
//! both are enforced best-effort. Every operation degrades to a silent
//! no-op on any IO error — the cache is an optimisation, never a
//! correctness dependency, so a read-only disk or missing cache dir just
//! falls back to the network path.
//!
//! All functions block on the filesystem; call them from
//! `tokio::task::spawn_blocking`, never directly on an async task.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Entries older than this (by mtime, refreshed on each read) are
/// treated as absent and deleted. Covers don't change, so this is long
/// — it exists to bound unbounded growth from one-off plays, not to
/// catch staleness.
const TTL: Duration = Duration::from_secs(60 * 60 * 24 * 30);

/// Soft ceiling on total cache size. When `write` pushes past this, the
/// oldest entries (by mtime) are evicted down to [`EVICT_TARGET`].
const MAX_BYTES: u64 = 256 * 1024 * 1024;

/// Evict down to this on overflow rather than exactly `MAX_BYTES` so we
/// don't repack on every single write once full.
const EVICT_TARGET: u64 = 200 * 1024 * 1024;

/// `<os-cache-dir>/frostify/art`, created on first use. `None` if the
/// platform has no cache dir or the directory can't be created.
fn cache_dir() -> Option<PathBuf> {
    let dir = dirs::cache_dir()?.join("frostify").join("art");
    fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// Reject anything that isn't a bare filename (defence-in-depth against
/// path traversal — keys are hex / base62 / known sentinels in practice).
fn is_safe_key(key: &str) -> bool {
    !(key.is_empty() || key.contains(['/', '\\']) || key == "." || key == "..")
}

/// Map a cache key to its on-disk path under the art dir.
fn entry_path(key: &str) -> Option<PathBuf> {
    if !is_safe_key(key) {
        return None;
    }
    cache_dir().map(|d| d.join(key))
}

/// Read cached bytes for `key`, or `None` on miss / expiry / IO error.
/// On a hit the entry's mtime is refreshed so the LRU eviction in
/// [`write`] treats recently-used covers as hot.
pub fn read(key: &str) -> Option<Vec<u8>> {
    let path = entry_path(key)?;
    let meta = fs::metadata(&path).ok()?;
    let age = meta
        .modified()
        .ok()
        .and_then(|m| SystemTime::now().duration_since(m).ok())
        .unwrap_or(Duration::ZERO);
    if age > TTL {
        let _ = fs::remove_file(&path);
        return None;
    }
    let bytes = fs::read(&path).ok()?;
    // Touch mtime = "used now" for LRU. Best-effort; ignore failure.
    if let Ok(f) = fs::OpenOptions::new().write(true).open(&path) {
        let _ = f.set_modified(SystemTime::now());
    }
    Some(bytes)
}

/// Persist `bytes` for `key`, then enforce the size cap. Best-effort:
/// any IO failure is swallowed.
pub fn write(key: &str, bytes: &[u8]) {
    let Some(path) = entry_path(key) else { return };
    if fs::write(&path, bytes).is_err() {
        return;
    }
    enforce_cap();
}

/// If the cache directory exceeds [`MAX_BYTES`], delete oldest-by-mtime
/// entries until under [`EVICT_TARGET`].
fn enforce_cap() {
    let Some(dir) = cache_dir() else { return };
    let Ok(rd) = fs::read_dir(&dir) else { return };
    let mut entries: Vec<(PathBuf, SystemTime, u64)> = Vec::new();
    let mut total: u64 = 0;
    for e in rd.flatten() {
        let Ok(meta) = e.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let len = meta.len();
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        total += len;
        entries.push((e.path(), mtime, len));
    }
    if total <= MAX_BYTES {
        return;
    }
    entries.sort_by_key(|(_, mtime, _)| *mtime); // oldest first
    for (path, _, len) in entries {
        if total <= EVICT_TARGET {
            break;
        }
        if fs::remove_file(&path).is_ok() {
            total = total.saturating_sub(len);
        }
    }
}

// ============================================================================
// API JSON cache
//
// Caches deserialized API payloads (playlist track listings, etc.) as
// JSON on disk, keyed by a caller-supplied id. Unlike the album-art
// bytes above (immutable → 30-day TTL, mtime-refreshed LRU), JSON
// listings are *mutable* — a user can edit a playlist — so the caller
// passes a much shorter TTL and reads do NOT touch mtime (an entry ages
// out from its original fetch time, never kept alive by re-reads).
// ============================================================================

/// Soft ceiling on the JSON cache dir; evict oldest past this.
const JSON_MAX_BYTES: u64 = 64 * 1024 * 1024;
const JSON_EVICT_TARGET: u64 = 48 * 1024 * 1024;

/// `<os-cache-dir>/frostify/json`, created on first use.
fn json_dir() -> Option<PathBuf> {
    let dir = dirs::cache_dir()?.join("frostify").join("json");
    fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn json_path(key: &str) -> Option<PathBuf> {
    if !is_safe_key(key) {
        return None;
    }
    json_dir().map(|d| d.join(format!("{key}.json")))
}

/// Read + deserialize a cached JSON value for `key`, or `None` on miss /
/// expiry (older than `ttl`) / IO / parse error. Does not refresh mtime.
pub fn read_json<T: DeserializeOwned>(key: &str, ttl: Duration) -> Option<T> {
    let path = json_path(key)?;
    let meta = fs::metadata(&path).ok()?;
    let age = meta
        .modified()
        .ok()
        .and_then(|m| SystemTime::now().duration_since(m).ok())
        .unwrap_or(Duration::ZERO);
    if age > ttl {
        let _ = fs::remove_file(&path);
        return None;
    }
    let bytes = fs::read(&path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Serialize + persist `value` for `key`, then enforce the size cap.
/// Best-effort: any IO / serialize failure is swallowed.
pub fn write_json<T: Serialize>(key: &str, value: &T) {
    let Some(path) = json_path(key) else { return };
    let Ok(bytes) = serde_json::to_vec(value) else {
        return;
    };
    if fs::write(&path, bytes).is_err() {
        return;
    }
    enforce_json_cap();
}

fn enforce_json_cap() {
    let Some(dir) = json_dir() else { return };
    evict_dir(&dir, JSON_MAX_BYTES, JSON_EVICT_TARGET);
}

/// Shared oldest-by-mtime eviction for a cache directory.
fn evict_dir(dir: &PathBuf, max_bytes: u64, target: u64) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<(PathBuf, SystemTime, u64)> = Vec::new();
    let mut total: u64 = 0;
    for e in rd.flatten() {
        let Ok(meta) = e.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let len = meta.len();
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        total += len;
        entries.push((e.path(), mtime, len));
    }
    if total <= max_bytes {
        return;
    }
    entries.sort_by_key(|(_, mtime, _)| *mtime);
    for (path, _, len) in entries {
        if total <= target {
            break;
        }
        if fs::remove_file(&path).is_ok() {
            total = total.saturating_sub(len);
        }
    }
}
