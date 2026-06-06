use crate::album_art;
use crate::api::{self, CurrentlyPlaying, HomeData, RepeatMode, TrackDetails};
use crate::disk_cache;
use crate::auth::oauth::{self, SpotifyAuthResponse, listen_for_callback, refresh_token};
use crate::auth::token_manager::{self, StoredTokens};
use crate::{cluster_listener, spirc_bootstrap, spotify_session};
use frostify_gfx::{ImageHandle, Uploader, WakeHandle};
use librespot_connect::Spirc;
use librespot_core::Session;
use librespot_core::authentication::Credentials;
use librespot_protocol::extended_metadata::{BatchedEntityRequest, EntityRequest, ExtensionQuery};
use librespot_protocol::extension_kind::ExtensionKind;
use log::{debug, error, info, warn};
use protobuf::EnumOrUnknown;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::mpsc::{self as tmpsc, UnboundedSender};

/// Cap the longest side of decoded album art. The 2048² atlas holds
/// Decoded RGBA cap per cover. Matches Spotify's largest variant
/// (640) — keeps the now-playing pane (~308 px logical, ~616 px @2×
/// DPI) and full-window backdrop sharp. Atlas headroom comes from
/// the 4096² image atlas in frostify-gfx, which fits ~40 covers at
/// this size.
const ALBUM_ART_MAX_DIM: u32 = 640;

#[derive(Debug)]
pub enum WorkerCommand {
    StartOAuth,
    TryLoadTokens,
    FetchHome { access_token: String },
    /// One-shot `/v1/me/player` poll to seed the initial player state.
    /// Dealer cluster pushes only on transitions — without this seed,
    /// the UI is blank from launch until the user toggles play/pause
    /// on whatever device is active.
    SeedPlayerState { access_token: String },
    FetchTrackDetails { access_token: String, track_id: String },
    /// Load a playlist's tracks (or the Liked Songs collection when
    /// `liked` is set). Result flows back as `PlaylistLoaded`.
    FetchPlaylist {
        access_token: String,
        id: String,
        liked: bool,
    },
    FetchAlbumArt { url: String, key: String },
    /// Fetch Spotify's own extracted accent colour for a cover, via the
    /// librespot session's extended-metadata endpoint. `image_hex` is the
    /// `i.scdn.co/image/<hex>` trailing hash (our cache key).
    FetchAccent { image_hex: String },
    /// Fetch the Spotify Canvas (looping video) URL for a track via the
    /// librespot extended-metadata endpoint (`CANVAZ`). `track_uri` is the
    /// `spotify:track:…` form; `track_id` echoes back so the UI can
    /// confirm the response still matches the current track.
    FetchCanvas { track_uri: String, track_id: String },
    ConnectSpotifySession { access_token: String },
    /// Transport control on the active Connect device (Web API).
    Playback { access_token: String, cmd: PlaybackCmd },
}

/// A transport intent dispatched from a player-bar button. Resolved to
/// the matching `api::*` call on the worker; the resulting state change
/// flows back through the dealer cluster subscription, not a direct
/// response, so the UI updates via the same path as remote changes.
#[derive(Debug, Clone)]
pub enum PlaybackCmd {
    Play,
    Pause,
    Next,
    Prev,
    Shuffle(bool),
    Repeat(RepeatMode),
    /// Seek the active device to an absolute position (ms).
    Seek(u32),
    /// Start a playlist/album context (or explicit track list) at an
    /// offset on the active device.
    PlayContext(api::PlayTarget),
}

#[derive(Debug, Clone)]
pub enum WorkerResponse {
    OAuthStarted { auth_url: String },
    OAuthComplete { auth: SpotifyAuthResponse },
    OAuthFailed { error: String },
    TokensLoaded { auth: SpotifyAuthResponse },
    NoStoredTokens,
    HomeData { data: HomeData },
    PlayerState { player: Option<CurrentlyPlaying> },
    TrackDetails { details: TrackDetails },
    /// First response for a playlist open: metadata + the first track
    /// page (or the *full* set when `complete`, e.g. a disk-cache hit or
    /// single-page playlist). The UI rebuilds once here to mount the
    /// header + full-length virtualised list.
    PlaylistOpened { detail: api::PlaylistDetail, complete: bool },
    /// A subsequent streamed track page appended to the open playlist's
    /// live buffer — no rebuild; the virtualised list reads it on scroll.
    PlaylistTracks {
        id: String,
        tracks: Vec<api::PlaylistTrack>,
        done: bool,
    },
    PlaylistFailed { id: String, error: String },
    AlbumArtReady {
        key: String,
        handle: ImageHandle,
        accent: [f32; 4],
    },
    AlbumArtFailed { key: String },
    /// Spotify's extracted accent for a cover. `key` is the image hex so
    /// the UI can confirm it still matches the current track before
    /// applying it.
    AccentReady { key: String, accent: [f32; 4] },
    /// A track's Canvas video resolved (URL fetched + MP4 downloaded to
    /// the disk cache). `track_id` lets the UI confirm it still matches
    /// the current track; `path` is the cached MP4 ready to decode.
    CanvasReady { track_id: String, path: std::path::PathBuf },
    /// No Canvas for the track (or fetch/download failed) — UI keeps the
    /// album art.
    CanvasNone { track_id: String },
    SpotifySessionConnected,
    SpotifySessionFailed { error: String },
}

pub struct Worker {
    cmd_tx: UnboundedSender<WorkerCommand>,
    resp_rx: Receiver<WorkerResponse>,
}

#[derive(Clone)]
struct Responder {
    tx: Sender<WorkerResponse>,
    wake: Arc<WakeHandle>,
}

impl Responder {
    fn send(&self, r: WorkerResponse) {
        let _ = self.tx.send(r);
        self.wake.wake();
    }
}

impl Worker {
    pub fn new(wake: Arc<WakeHandle>, uploader: Arc<Uploader>) -> Self {
        let (cmd_tx, mut cmd_rx) = tmpsc::unbounded_channel::<WorkerCommand>();
        let (resp_tx, resp_rx): (Sender<WorkerResponse>, Receiver<WorkerResponse>) = channel();
        let resp = Responder { tx: resp_tx, wake };

        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            // Long-lived librespot session — held on the worker so its
            // background tasks (AP socket, dealer) stay alive across
            // command iterations. `None` until `ConnectSpotifySession`
            // succeeds for the current login.
            let session: Arc<AsyncMutex<Option<Session>>> = Arc::new(AsyncMutex::new(None));
            // Spirc handle — dropping this disconnects the Connect device.
            // Held on the worker for lifetime parity with `session`.
            let spirc: Arc<AsyncMutex<Option<Spirc>>> = Arc::new(AsyncMutex::new(None));
            rt.block_on(async move {
                while let Some(cmd) = cmd_rx.recv().await {
                    match cmd {
                        WorkerCommand::StartOAuth => spawn_oauth(resp.clone()),
                        WorkerCommand::TryLoadTokens => spawn_try_load(resp.clone()),
                        WorkerCommand::FetchHome { access_token } => {
                            spawn_fetch_home(resp.clone(), access_token)
                        }
                        WorkerCommand::SeedPlayerState { access_token } => {
                            spawn_seed_player(resp.clone(), access_token)
                        }
                        WorkerCommand::FetchTrackDetails {
                            access_token,
                            track_id,
                        } => spawn_fetch_track_details(resp.clone(), access_token, track_id),
                        WorkerCommand::FetchPlaylist {
                            access_token,
                            id,
                            liked,
                        } => spawn_fetch_playlist(resp.clone(), access_token, id, liked),
                        WorkerCommand::FetchAlbumArt { url, key } => spawn_fetch_album_art(
                            resp.clone(),
                            uploader.clone(),
                            url,
                            key,
                        ),
                        WorkerCommand::FetchAccent { image_hex } => {
                            spawn_fetch_accent(resp.clone(), session.clone(), image_hex)
                        }
                        WorkerCommand::FetchCanvas { track_uri, track_id } => {
                            spawn_fetch_canvas(resp.clone(), session.clone(), track_uri, track_id)
                        }
                        WorkerCommand::ConnectSpotifySession { access_token } => spawn_connect_session(
                            resp.clone(),
                            session.clone(),
                            spirc.clone(),
                            access_token,
                        ),
                        WorkerCommand::Playback { access_token, cmd } => {
                            spawn_playback(access_token, cmd)
                        }
                    }
                }
            });
        });

        Self { cmd_tx, resp_rx }
    }

    pub fn start_oauth(&self) {
        let _ = self.cmd_tx.send(WorkerCommand::StartOAuth);
    }
    pub fn try_load_tokens(&self) {
        let _ = self.cmd_tx.send(WorkerCommand::TryLoadTokens);
    }
    pub fn fetch_home(&self, access_token: String) {
        let _ = self.cmd_tx.send(WorkerCommand::FetchHome { access_token });
    }
    pub fn seed_player_state(&self, access_token: String) {
        let _ = self
            .cmd_tx
            .send(WorkerCommand::SeedPlayerState { access_token });
    }
    pub fn fetch_track_details(&self, access_token: String, track_id: String) {
        let _ = self.cmd_tx.send(WorkerCommand::FetchTrackDetails {
            access_token,
            track_id,
        });
    }
    pub fn fetch_playlist(&self, access_token: String, id: String, liked: bool) {
        let _ = self.cmd_tx.send(WorkerCommand::FetchPlaylist {
            access_token,
            id,
            liked,
        });
    }
    pub fn fetch_album_art(&self, url: String, key: String) {
        let _ = self.cmd_tx.send(WorkerCommand::FetchAlbumArt { url, key });
    }
    pub fn fetch_accent(&self, image_hex: String) {
        let _ = self.cmd_tx.send(WorkerCommand::FetchAccent { image_hex });
    }
    pub fn fetch_canvas(&self, track_uri: String, track_id: String) {
        let _ = self
            .cmd_tx
            .send(WorkerCommand::FetchCanvas { track_uri, track_id });
    }
    pub fn connect_spotify_session(&self, access_token: String) {
        let _ = self
            .cmd_tx
            .send(WorkerCommand::ConnectSpotifySession { access_token });
    }
    pub fn playback(&self, access_token: String, cmd: PlaybackCmd) {
        let _ = self
            .cmd_tx
            .send(WorkerCommand::Playback { access_token, cmd });
    }
    pub fn poll(&self) -> Option<WorkerResponse> {
        self.resp_rx.try_recv().ok()
    }
}

fn spawn_oauth(resp: Responder) {
    tokio::spawn(async move {
        let (url, verifier) = oauth::get_spotify_auth_url();
        resp.send(WorkerResponse::OAuthStarted { auth_url: url });
        match listen_for_callback(verifier).await {
            Ok(auth) => {
                debug!("OAuth complete");
                let stored = StoredTokens::from(auth.clone());
                if let Err(e) = token_manager::save_tokens(&stored) {
                    error!("save tokens: {e}");
                }
                resp.send(WorkerResponse::OAuthComplete { auth });
            }
            Err(e) => {
                resp.send(WorkerResponse::OAuthFailed { error: e.to_string() });
            }
        }
    });
}

fn spawn_fetch_home(resp: Responder, access_token: String) {
    tokio::spawn(async move {
        let (profile, playlists, recent, top_artists, top_tracks) = tokio::join!(
            api::get_me(&access_token),
            api::get_playlists(&access_token),
            api::get_recently_played(&access_token),
            api::get_top_artists(&access_token, 10),
            api::get_top_tracks(&access_token, 10),
        );
        let mut data = HomeData::default();
        match profile {
            Ok(p) => data.profile = Some(p),
            Err(e) => warn!("get_me failed: {e}"),
        }
        match playlists {
            Ok(ps) => data.playlists = ps,
            Err(e) => warn!("get_playlists failed: {e}"),
        }
        match recent {
            Ok(rs) => data.recent = rs,
            Err(e) => warn!("get_recently_played failed: {e}"),
        }
        match top_artists {
            Ok(a) => data.top_artists = a,
            Err(e) => warn!("get_top_artists failed: {e}"),
        }
        match top_tracks {
            Ok(t) => data.top_tracks = t,
            Err(e) => warn!("get_top_tracks failed: {e}"),
        }
        // Chained "latest release": newest album from #1 top artist.
        // Skipped silently if top_artists came back empty.
        if let Some(top) = data.top_artists.first() {
            match api::get_artist_albums(&access_token, &top.id, 5).await {
                Ok(mut albums) => data.latest_release = albums.drain(..).next(),
                Err(e) => warn!("get_artist_albums for top artist failed: {e}"),
            }
        }
        info!(
            "home data: profile={} playlists={} recent={} top_artists={} top_tracks={} latest_release={}",
            data.profile.is_some(),
            data.playlists.len(),
            data.recent.len(),
            data.top_artists.len(),
            data.top_tracks.len(),
            data.latest_release.is_some(),
        );
        resp.send(WorkerResponse::HomeData { data });
    });
}

fn spawn_seed_player(resp: Responder, access_token: String) {
    tokio::spawn(async move {
        match api::get_currently_playing(&access_token).await {
            Ok(player) => {
                info!("seeded initial player state from /me/player: present={}", player.is_some());
                resp.send(WorkerResponse::PlayerState { player });
            }
            Err(e) => warn!("seed /me/player failed: {e}"),
        }
    });
}

/// How long a cover's extracted accent stays cached. The colour is
/// immutable for a given cover (keyed by the image hash), so this is long
/// — it just bounds growth, like the art cache.
const ACCENT_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24 * 30);

/// Resolve Spotify's own extracted accent colour for a cover. Checks the
/// JSON disk cache first (instant, no session — kills the track-change
/// window where the new art shows with the *previous* accent because the
/// session round-trip lagged), then falls back to the `EXTRACTED_COLOR`
/// extended-metadata query, caching the result. No-ops silently if there's
/// no session yet and no cache (UI keeps the art-derived pixel average).
fn spawn_fetch_accent(
    resp: Responder,
    session_slot: Arc<AsyncMutex<Option<Session>>>,
    image_hex: String,
) {
    tokio::spawn(async move {
        let cache_key = format!("accent_{image_hex}");
        // Cache hit → apply immediately, no session needed.
        if let Some(accent) = tokio::task::spawn_blocking({
            let cache_key = cache_key.clone();
            move || disk_cache::read_json::<[f32; 4]>(&cache_key, ACCENT_TTL)
        })
        .await
        .ok()
        .flatten()
        {
            resp.send(WorkerResponse::AccentReady { key: image_hex, accent });
            return;
        }
        let session = { session_slot.lock().await.clone() };
        let Some(session) = session else {
            debug!("accent fetch skipped — no session yet ({image_hex})");
            return;
        };
        // The extracted-colour extension is keyed by the cover's image
        // URI, not the track URI.
        let image_uri = format!("spotify:image:{image_hex}");
        match fetch_extracted_color(&session, &image_uri).await {
            Some(accent) => {
                debug!("extracted color {image_hex} -> {accent:?}");
                tokio::task::spawn_blocking(move || disk_cache::write_json(&cache_key, &accent))
                    .await
                    .ok();
                resp.send(WorkerResponse::AccentReady { key: image_hex, accent });
            }
            None => debug!("no extracted color for {image_hex}"),
        }
    });
}

async fn fetch_extracted_color(session: &Session, image_uri: &str) -> Option<[f32; 4]> {
    let req = BatchedEntityRequest {
        entity_request: vec![EntityRequest {
            entity_uri: image_uri.to_string(),
            query: vec![ExtensionQuery {
                extension_kind: EnumOrUnknown::new(ExtensionKind::EXTRACTED_COLOR),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut res = match session.spclient().get_extended_metadata(req).await {
        Ok(r) => r,
        Err(e) => {
            warn!("extracted-color request failed ({image_uri}): {e}");
            return None;
        }
    };
    // BatchedExtensionResponse → first entity → first extension → bytes.
    let mut arr = res.extended_metadata.pop()?;
    let mut data = arr.extension_data.pop()?;
    let any = data.extension_data.take()?;
    crate::extracted_color::parse_color_dark(&any.value)
}

/// Per-track Canvas metadata persisted to the JSON disk cache so we don't
/// re-hit Spotify's spclient (and don't need a live librespot session) for
/// a track we've already resolved. An empty `url` is a **negative** cache
/// entry — "this track has no video canvas" — so we don't re-query tracks
/// that never had one. Keyed by `canvas_meta_<track_id>`.
#[derive(serde::Serialize, serde::Deserialize)]
struct CanvasMeta {
    /// Canvas video URL, or empty for "no video canvas".
    url: String,
}

/// How long a track→canvas mapping stays valid. Canvas rarely changes for
/// a given track, so this is generous; it bounds growth + lets a removed
/// canvas eventually re-resolve, not catch same-day edits.
const CANVAS_META_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24 * 7);

/// Resolve + cache a track's Spotify Canvas video. Resolution order: first
/// the per-track metadata cache (no session / no network needed), then the
/// CANVAZ extended-metadata query (needs a live session). The resolved URL
/// (or a negative marker) is written back to the metadata cache, and the
/// MP4 bytes themselves are disk-cached separately. Responds `CanvasReady
/// { path }` for a video canvas or `CanvasNone` otherwise (no canvas /
/// image-only / fetch fail) so the UI falls back to album art.
fn spawn_fetch_canvas(
    resp: Responder,
    session_slot: Arc<AsyncMutex<Option<Session>>>,
    track_uri: String,
    track_id: String,
) {
    tokio::spawn(async move {
        let meta_key = format!("canvas_meta_{track_id}");
        // 1. Metadata-cache hit → resolve without touching the session.
        let cached = tokio::task::spawn_blocking({
            let meta_key = meta_key.clone();
            move || disk_cache::read_json::<CanvasMeta>(&meta_key, CANVAS_META_TTL)
        })
        .await
        .ok()
        .flatten();
        let url = match cached {
            Some(meta) if !meta.url.is_empty() => {
                debug!("canvas meta-cache hit {track_id}");
                meta.url
            }
            Some(_) => {
                // Negative cache: known to have no video canvas.
                debug!("canvas meta-cache hit (none) {track_id}");
                resp.send(WorkerResponse::CanvasNone { track_id });
                return;
            }
            None => {
                // 2. Cache miss → query spclient (needs a session). Without
                // one yet, bail *without* negative-caching so the retry
                // after the session connects can still resolve it.
                let session = { session_slot.lock().await.clone() };
                let Some(session) = session else {
                    debug!("canvas fetch deferred — no session yet ({track_id})");
                    resp.send(WorkerResponse::CanvasNone { track_id });
                    return;
                };
                let entry = fetch_canvas_entry(&session, &track_uri).await;
                let video = entry.as_ref().map(|e| e.kind.is_video()).unwrap_or(false);
                let url = if video {
                    entry.map(|e| e.url).unwrap_or_default()
                } else {
                    String::new()
                };
                // Write back (positive or negative) so we don't re-query.
                let write_url = url.clone();
                tokio::task::spawn_blocking(move || {
                    disk_cache::write_json(&meta_key, &CanvasMeta { url: write_url });
                })
                .await
                .ok();
                if url.is_empty() {
                    debug!("no video canvas for {track_id}");
                    resp.send(WorkerResponse::CanvasNone { track_id });
                    return;
                }
                url
            }
        };
        // Cache key = trailing path segment of the canvas URL (a stable
        // hash + `.mp4`), prefixed so it never collides with art keys.
        let key = format!("canvas_{}", canvas_cache_key(&url));
        // Disk-cache hit → skip the network.
        if let Some(path) = tokio::task::spawn_blocking({
            let key = key.clone();
            move || disk_cache::path(&key)
        })
        .await
        .ok()
        .flatten()
        {
            debug!("canvas disk-cache hit {track_id}");
            resp.send(WorkerResponse::CanvasReady { track_id, path });
            return;
        }
        let Some(bytes) = fetch_art_bytes(&url).await else {
            warn!("canvas download failed ({url})");
            resp.send(WorkerResponse::CanvasNone { track_id });
            return;
        };
        let path = tokio::task::spawn_blocking(move || {
            disk_cache::write(&key, &bytes);
            disk_cache::path(&key)
        })
        .await
        .ok()
        .flatten();
        match path {
            Some(path) => {
                debug!("canvas cached {track_id} ({} bytes)", path.display());
                resp.send(WorkerResponse::CanvasReady { track_id, path });
            }
            None => resp.send(WorkerResponse::CanvasNone { track_id }),
        }
    });
}

/// Trailing filename of a canvas URL (a stable hash), filesystem-safe.
fn canvas_cache_key(url: &str) -> String {
    url.rsplit('/').next().unwrap_or(url).replace(['?', '&', '='], "_")
}

async fn fetch_canvas_entry(session: &Session, track_uri: &str) -> Option<crate::canvas::CanvasEntry> {
    let req = BatchedEntityRequest {
        entity_request: vec![EntityRequest {
            entity_uri: track_uri.to_string(),
            query: vec![ExtensionQuery {
                extension_kind: EnumOrUnknown::new(ExtensionKind::CANVAZ),
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut res = match session.spclient().get_extended_metadata(req).await {
        Ok(r) => r,
        Err(e) => {
            warn!("canvas request failed ({track_uri}): {e}");
            return None;
        }
    };
    debug!(
        "canvas xmeta {track_uri}: outer={} entries",
        res.extended_metadata.len()
    );
    let mut arr = res.extended_metadata.pop()?;
    debug!("canvas xmeta inner={} entries", arr.extension_data.len());
    let mut data = arr.extension_data.pop()?;
    let any = data.extension_data.take()?;
    debug!(
        "canvas xmeta type_url={:?} value_len={}",
        any.type_url,
        any.value.len()
    );
    crate::canvas::parse_canvas(&any.value)
}

fn spawn_playback(access_token: String, cmd: PlaybackCmd) {
    tokio::spawn(async move {
        let result = match cmd.clone() {
            PlaybackCmd::Play => api::play(&access_token).await,
            PlaybackCmd::Pause => api::pause(&access_token).await,
            PlaybackCmd::Next => api::next_track(&access_token).await,
            PlaybackCmd::Prev => api::previous_track(&access_token).await,
            PlaybackCmd::Shuffle(on) => api::set_shuffle(&access_token, on).await,
            PlaybackCmd::Repeat(mode) => api::set_repeat(&access_token, mode).await,
            PlaybackCmd::Seek(ms) => api::seek(&access_token, ms).await,
            PlaybackCmd::PlayContext(target) => api::play_context(&access_token, target).await,
        };
        match result {
            Ok(()) => debug!("playback cmd {cmd:?} ok"),
            // 404 = no active device; the optimistic UI flip may now be
            // out of sync with reality, but there's nothing to control
            // until the user starts playback on some device.
            Err(e) => warn!("playback cmd {cmd:?} failed: {e}"),
        }
    });
}

fn spawn_fetch_track_details(resp: Responder, access_token: String, track_id: String) {
    tokio::spawn(async move {
        match api::get_track(&access_token, &track_id).await {
            Ok(details) => resp.send(WorkerResponse::TrackDetails { details }),
            Err(e) => warn!("get_track({track_id}) failed: {e}"),
        }
    });
}

/// Disk-cache TTL for playlist track listings. Longer than the UI's
/// in-memory cache (which covers within-session re-opens) so a relaunch
/// re-opening a big playlist skips re-paging the whole thing from the
/// Web API, but short enough that edits made elsewhere surface within
/// the hour. Listings are mutable, so unlike album art this is hours,
/// not days.
const PLAYLIST_DISK_TTL: std::time::Duration = std::time::Duration::from_secs(60 * 30);

/// Hard ceiling on streamed tracks — guards against a pathological
/// `total` driving an unbounded loop. 10k covers every realistic
/// library; the windowed-play UX matters more than completeness beyond.
const MAX_STREAM_TRACKS: usize = 10_000;

fn spawn_fetch_playlist(resp: Responder, access_token: String, id: String, liked: bool) {
    tokio::spawn(async move {
        // 1. Disk cache first — a fresh hit delivers the whole listing in
        //    one `complete` response (no re-paging the CDN/API).
        let key = id.clone();
        let cached = tokio::task::spawn_blocking(move || {
            disk_cache::read_json::<api::PlaylistDetail>(&key, PLAYLIST_DISK_TTL)
        })
        .await
        .ok()
        .flatten();
        if let Some(detail) = cached {
            info!("playlist '{}' disk-cache hit: {} tracks", detail.name, detail.tracks.len());
            resp.send(WorkerResponse::PlaylistOpened { detail, complete: true });
            return;
        }

        // 2. Metadata first (playlists only — Liked Songs gets its total
        //    from the first page) so the header + scrollbar appear before
        //    any track page lands.
        let (name, owner, image_url, context_uri) = if liked {
            ("Liked Songs".to_string(), String::new(), None, None)
        } else {
            match api::playlist_meta(&access_token, &id).await {
                Ok(m) => (
                    m.name,
                    m.owner,
                    m.image_url,
                    Some(format!("spotify:playlist:{id}")),
                ),
                Err(e) => {
                    warn!("playlist_meta({id}) failed: {e}");
                    resp.send(WorkerResponse::PlaylistFailed { id, error: e.to_string() });
                    return;
                }
            }
        };

        // 3. Stream track pages. The first page rides a `PlaylistOpened`
        //    (mounts the list); the rest are `PlaylistTracks` appended to
        //    the live buffer with no rebuild.
        let page_size = if liked { api::LIKED_PAGE } else { api::PLAYLIST_PAGE };
        let mut offset = 0u32;
        let mut first = true;
        let mut total = 0u32;
        let mut accumulated: Vec<api::PlaylistTrack> = Vec::new();
        loop {
            let url = if liked {
                api::liked_tracks_url(offset, page_size)
            } else {
                api::playlist_tracks_url(&id, offset, page_size)
            };
            let page = match api::fetch_tracks_page(&access_token, &url).await {
                Ok(p) => p,
                Err(e) => {
                    warn!("fetch_tracks_page({id} @{offset}) failed: {e}");
                    if first {
                        resp.send(WorkerResponse::PlaylistFailed {
                            id: id.clone(),
                            error: e.to_string(),
                        });
                    }
                    break;
                }
            };
            total = page.total;
            let next = offset + page_size;
            let done = page.raw_count < page_size
                || page.raw_count == 0
                || (total > 0 && next >= total)
                || accumulated.len() + page.tracks.len() >= MAX_STREAM_TRACKS;
            accumulated.extend(page.tracks.iter().cloned());
            if first {
                let detail = api::PlaylistDetail {
                    id: id.clone(),
                    name: name.clone(),
                    owner: owner.clone(),
                    image_url: image_url.clone(),
                    context_uri: context_uri.clone(),
                    tracks: page.tracks,
                    total,
                };
                resp.send(WorkerResponse::PlaylistOpened { detail, complete: done });
                first = false;
            } else {
                resp.send(WorkerResponse::PlaylistTracks {
                    id: id.clone(),
                    tracks: page.tracks,
                    done,
                });
            }
            if done {
                break;
            }
            offset = next;
        }

        // 4. Write the assembled listing to disk for instant re-opens.
        if !first {
            let detail = api::PlaylistDetail {
                id: id.clone(),
                name,
                owner,
                image_url,
                context_uri,
                tracks: accumulated,
                total,
            };
            let key = id.clone();
            tokio::task::spawn_blocking(move || disk_cache::write_json(&key, &detail));
        }
    });
}

/// Global cap on concurrent album-art network fetches. Spotify's CDN
/// generally tolerates parallel requests, but a full Home view can
/// kick off 30–50 covers at once; throttling keeps us friendly + means
/// a 429 from anywhere can't snowball into a flood of retries.
const ART_CONCURRENCY: usize = 4;

fn art_throttle() -> &'static Arc<tokio::sync::Semaphore> {
    static SEM: std::sync::OnceLock<Arc<tokio::sync::Semaphore>> = std::sync::OnceLock::new();
    SEM.get_or_init(|| Arc::new(tokio::sync::Semaphore::new(ART_CONCURRENCY)))
}

/// Fetch the raw image bytes for `url`. Honors `Retry-After` on 429 +
/// retries once on transient failure. Caller-side throttle in
/// `spawn_fetch_album_art` bounds concurrency.
async fn fetch_art_bytes(url: &str) -> Option<Vec<u8>> {
    for attempt in 1..=2 {
        let resp = match reqwest::get(url).await {
            Ok(r) => r,
            Err(e) => {
                warn!("album art fetch failed ({url}) attempt {attempt}: {e}");
                continue;
            }
        };
        let status = resp.status();
        // 429: back off for the server-advertised window before retrying.
        // 5xx: brief pause + retry once.
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let wait = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(2);
            warn!("album art 429 ({url}) — sleeping {wait}s");
            tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
            continue;
        }
        if !status.is_success() {
            warn!("album art status {status} ({url}) attempt {attempt}");
            if status.is_server_error() && attempt == 1 {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            continue;
        }
        match resp.bytes().await {
            Ok(b) => return Some(b.to_vec()),
            Err(e) => warn!("album art read body failed ({url}) attempt {attempt}: {e}"),
        }
    }
    None
}

fn spawn_fetch_album_art(
    resp: Responder,
    uploader: Arc<Uploader>,
    url: String,
    key: String,
) {
    tokio::spawn(async move {
        // 1. Disk cache first — a hit skips the network entirely, which
        //    is what kills the track-change "stuck on old art" window for
        //    any cover seen before and stops re-hammering the CDN.
        let key_for_disk = key.clone();
        let cached = tokio::task::spawn_blocking(move || disk_cache::read(&key_for_disk))
            .await
            .ok()
            .flatten();
        let (bytes, from_network): (Vec<u8>, bool) = match cached {
            Some(b) => {
                debug!("album art disk-cache hit key={key}");
                (b, false)
            }
            None => {
                // Bound concurrent network fetches across all in-flight
                // art tasks. Held only for the actual GET (decode + atlas
                // upload run uncapped).
                let _permit = art_throttle().acquire().await.ok();
                match fetch_art_bytes(&url).await {
                    Some(b) => (b, true),
                    None => {
                        resp.send(WorkerResponse::AlbumArtFailed { key });
                        return;
                    }
                }
            }
        };
        // 2. Decode off the network task — image::decode is blocking CPU
        //    work that would stall the tokio worker. Accent extraction +
        //    the disk write-back (network fetches only) ride the same
        //    spawn_blocking so we never re-walk the buffer on the UI side.
        let key_for_decode = key.clone();
        let decoded = tokio::task::spawn_blocking(move || {
            if from_network {
                disk_cache::write(&key_for_decode, &bytes);
            }
            let (w, h, rgba) = album_art::decode_to_rgba(&bytes, ALBUM_ART_MAX_DIM)?;
            let accent = album_art::extract_accent(&rgba, w, h);
            Some((w, h, rgba, accent))
        })
        .await
        .ok()
        .flatten();
        let Some((w, h, rgba, accent)) = decoded else {
            warn!("album art decode failed for key={key}");
            resp.send(WorkerResponse::AlbumArtFailed { key });
            return;
        };
        // Hand off to the UI thread for atlas upload. Callback fires on
        // the UI thread and ships the resolved handle back through the
        // existing response channel.
        let resp_for_cb = resp.clone();
        uploader.upload_rgba(w, h, rgba, move |maybe_handle| match maybe_handle {
            Some(handle) => resp_for_cb.send(WorkerResponse::AlbumArtReady {
                key,
                handle,
                accent,
            }),
            None => {
                warn!("uploader rejected album art upload");
                resp_for_cb.send(WorkerResponse::AlbumArtFailed { key });
            }
        });
    });
}

fn spawn_connect_session(
    resp: Responder,
    session_slot: Arc<AsyncMutex<Option<Session>>>,
    spirc_slot: Arc<AsyncMutex<Option<Spirc>>>,
    access_token: String,
) {
    tokio::spawn(async move {
        let s = spotify_session::new_session();
        *session_slot.lock().await = Some(s.clone());

        let creds = Credentials::with_access_token(access_token);
        let boot = match spirc_bootstrap::start(s, creds).await {
            Ok(b) => b,
            Err(e) => {
                error!("spirc bootstrap failed: {e}");
                resp.send(WorkerResponse::SpotifySessionFailed { error: e.to_string() });
                return;
            }
        };
        info!("spirc connect device registered as 'Frostify'");
        let spirc_bootstrap::SpircBootstrap {
            spirc,
            spirc_task,
            cluster_sub,
        } = boot;
        *spirc_slot.lock().await = Some(spirc);

        // Drive the Connect device event loop forever.
        tokio::spawn(async move {
            spirc_task.await;
            warn!("spirc_task ended — Connect device offline");
        });

        // Drain cluster updates into UI-thread responses.
        let resp_for_cluster = resp.clone();
        tokio::spawn(async move {
            cluster_listener::run(cluster_sub, move |player| {
                resp_for_cluster.send(WorkerResponse::PlayerState { player });
            })
            .await;
        });

        resp.send(WorkerResponse::SpotifySessionConnected);
    });
}

fn spawn_try_load(resp: Responder) {
    tokio::spawn(async move {
        match token_manager::load_tokens() {
            Ok(tokens) => {
                // Self-heal: if the stored token was minted before a
                // scope addition (constants.rs SPOTIFY_ACCESS_SCOPES),
                // it'll 401 on the new endpoints. Drop and force re-auth.
                if !tokens.has_scopes(crate::constants::SPOTIFY_ACCESS_SCOPES) {
                    info!("stored token missing required scopes — wiping + re-auth");
                    let _ = token_manager::delete_tokens();
                    resp.send(WorkerResponse::NoStoredTokens);
                    return;
                }
                if tokens.is_expired() {
                    info!("refreshing expired token");
                    match refresh_token(&tokens.refresh_token).await {
                        Ok(auth) => {
                            let _ = token_manager::save_tokens(&StoredTokens::from(auth.clone()));
                            resp.send(WorkerResponse::TokensLoaded { auth });
                        }
                        Err(e) => {
                            error!("refresh failed: {e}");
                            resp.send(WorkerResponse::NoStoredTokens);
                        }
                    }
                } else {
                    resp.send(WorkerResponse::TokensLoaded {
                        auth: tokens.to_auth_response(),
                    });
                }
            }
            Err(e) => {
                debug!("no stored tokens: {e}");
                resp.send(WorkerResponse::NoStoredTokens);
            }
        }
    });
}
