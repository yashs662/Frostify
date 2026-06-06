//! Library slice — the Home feed data + playlist loading/caching.
//!
//! Owns the fetched [`HomeData`], the open centre-pane playlist (a live
//! streaming row buffer the worker pages fill), the playlist TTL cache,
//! and the in-flight gate. Playlists load **progressively**: a shell from
//! sidebar-known metadata appears immediately, the first page mounts the
//! virtualised list, and later pages stream into the shared buffer the
//! `lazy_list` reads on scroll — no blocking "loading all 989 songs".

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::album_art;
use crate::api::{HomeData, PlaylistDetail, PlaylistTrack};
use crate::model::ArtModel;
use crate::views::home::playlist::{self, PlaylistRow, RowBuf};
use crate::worker::Worker;

/// How long a cached playlist stays fresh before a re-open re-fetches it.
/// Long enough to make back-and-forth navigation free, short enough that
/// edits made elsewhere show up within a few minutes.
const PLAYLIST_TTL: Duration = Duration::from_secs(300);

/// A loaded playlist plus the wall-clock at which it was fetched — drives
/// the in-memory TTL cache so re-opening within [`PLAYLIST_TTL`] reuses
/// the data instead of re-hitting the Web API.
struct CachedPlaylist {
    detail: PlaylistDetail,
    fetched: Instant,
}

/// The playlist currently open in the centre pane. Holds the metadata
/// plus a **live, growable** row buffer the streaming worker pages fill —
/// the view's `lazy_list` reads it on scroll, so later pages appear
/// without a rebuild. `total` drives the list length from the first
/// response so the scrollbar is correct before everything has streamed.
pub struct OpenPlaylist {
    pub liked: bool,
    pub name: String,
    pub owner: String,
    pub image_url: Option<String>,
    pub context_uri: Option<String>,
    pub total: u32,
    pub rows: RowBuf,
    /// Metadata not yet arrived (header shows the sidebar-known name).
    pub loading: bool,
    /// Every page has streamed in.
    pub complete: bool,
}

pub struct LibraryModel {
    /// The Home feed (greeting, recents, top artists, playlists, …).
    pub home: RefCell<HomeData>,
    /// The playlist open in the centre pane (live streaming buffer).
    pub open_playlist: RefCell<Option<OpenPlaylist>>,
    /// Playlist detail TTL cache (id → detail + fetch time). Liked Songs
    /// lives here under `api::LIKED_SONGS_ID`.
    playlist_cache: RefCell<HashMap<String, CachedPlaylist>>,
    /// Playlist ids with a fetch in flight — gate so navigating back and
    /// forth doesn't dispatch duplicate loads.
    playlist_inflight: RefCell<HashSet<String>>,
}

impl LibraryModel {
    pub fn new() -> Self {
        Self {
            home: RefCell::default(),
            open_playlist: RefCell::default(),
            playlist_cache: RefCell::default(),
            playlist_inflight: RefCell::default(),
        }
    }

    // --- in-flight gate + TTL cache -----------------------------------

    pub fn is_inflight(&self, id: &str) -> bool {
        self.playlist_inflight.borrow().contains(id)
    }

    pub fn clear_inflight(&self, id: &str) {
        self.playlist_inflight.borrow_mut().remove(id);
    }

    /// Cache a fully-loaded playlist for an instant re-open.
    pub fn cache(&self, detail: PlaylistDetail) {
        self.playlist_cache
            .borrow_mut()
            .insert(detail.id.clone(), CachedPlaylist { detail, fetched: Instant::now() });
    }

    /// A fresh (within TTL) cached detail clone, if any.
    fn cached_detail(&self, id: &str) -> Option<PlaylistDetail> {
        self.playlist_cache
            .borrow()
            .get(id)
            .filter(|c| c.fetched.elapsed() < PLAYLIST_TTL)
            .map(|c| c.detail.clone())
    }

    // --- row baking ---------------------------------------------------

    /// Bake `tracks` into [`PlaylistRow`]s appended to `buf`. Each cover
    /// gets a reactive `Signal` off the shared art cache (so an arriving
    /// handle repaints just that thumb), but the **fetch is not dispatched
    /// here** — the cover downloads lazily when the row scrolls into view,
    /// so opening a 989-track playlist doesn't kick off 989 downloads.
    pub fn build_rows(&self, art: &ArtModel, buf: &RowBuf, tracks: &[PlaylistTrack]) {
        let mut out = buf.borrow_mut();
        out.reserve(tracks.len());
        for t in tracks {
            let cover = t
                .album_image_url
                .as_ref()
                .map(|u| art.or_signal(album_art::cache_key(u)));
            out.push(PlaylistRow {
                title: t.name.clone(),
                artist: t.artist.clone(),
                album: t.album.clone(),
                duration: playlist::fmt_duration(t.duration_ms),
                uri: t.uri.clone(),
                art: cover,
                cover_url: t.album_image_url.clone(),
            });
        }
    }

    // --- opening / loading --------------------------------------------

    /// Set up `open_playlist` for a nav target. A fresh in-memory cache
    /// hit populates the row buffer fully (instant). Otherwise a shell is
    /// built from the sidebar-known name/cover (header shows immediately)
    /// and a streaming fetch is dispatched.
    pub fn open_for(
        &self,
        art: &ArtModel,
        worker: &Worker,
        token: Option<String>,
        id: &str,
        liked: bool,
    ) {
        if let Some(detail) = self.cached_detail(id) {
            let buf: RowBuf = Rc::new(RefCell::new(Vec::new()));
            self.build_rows(art, &buf, &detail.tracks);
            *self.open_playlist.borrow_mut() = Some(OpenPlaylist {
                liked,
                name: detail.name,
                owner: detail.owner,
                image_url: detail.image_url,
                context_uri: detail.context_uri,
                total: detail.total,
                rows: buf,
                loading: false,
                complete: true,
            });
            return;
        }

        // Shell from whatever the sidebar already knows, so the header
        // isn't blank while metadata + the first page stream in.
        let (name, image_url) = if liked {
            ("Liked Songs".to_string(), None)
        } else {
            self.home
                .borrow()
                .playlists
                .iter()
                .find(|p| p.id == id)
                .map(|p| (p.name.clone(), p.image_url.clone()))
                .unwrap_or((String::new(), None))
        };
        let context_uri = if liked { None } else { Some(format!("spotify:playlist:{id}")) };
        let buf: RowBuf = Rc::new(RefCell::new(Vec::new()));
        *self.open_playlist.borrow_mut() = Some(OpenPlaylist {
            liked,
            name,
            owner: String::new(),
            image_url,
            context_uri,
            total: 0,
            rows: buf,
            loading: true,
            complete: false,
        });
        self.ensure_loaded(worker, token, id, liked);
    }

    /// Dispatch a streaming playlist fetch unless a load is already in
    /// flight. Liked Songs routes through the same path under its sentinel
    /// id. `token` is the live access token (read at call time).
    pub fn ensure_loaded(&self, worker: &Worker, token: Option<String>, id: &str, liked: bool) {
        if self.is_inflight(id) {
            return;
        }
        let Some(token) = token else {
            log::warn!("playlist load skipped — no auth token");
            return;
        };
        self.playlist_inflight.borrow_mut().insert(id.to_string());
        worker.fetch_playlist(token, id.to_string(), liked);
    }
}

impl Default for LibraryModel {
    fn default() -> Self {
        Self::new()
    }
}
