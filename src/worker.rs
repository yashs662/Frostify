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
    FetchAlbumArt { url: String, key: String },
    /// Fetch Spotify's own extracted accent colour for a cover, via the
    /// librespot session's extended-metadata endpoint. `image_hex` is the
    /// `i.scdn.co/image/<hex>` trailing hash (our cache key).
    FetchAccent { image_hex: String },
    ConnectSpotifySession { access_token: String },
    /// Transport control on the active Connect device (Web API).
    Playback { access_token: String, cmd: PlaybackCmd },
    #[allow(dead_code)]
    Shutdown,
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
    SpotifySessionConnected,
    SpotifySessionFailed { error: String },
}

pub struct Worker {
    cmd_tx: UnboundedSender<WorkerCommand>,
    resp_rx: Receiver<WorkerResponse>,
    #[allow(dead_code)]
    handle: Option<thread::JoinHandle<()>>,
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

        let handle = thread::spawn(move || {
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
                        WorkerCommand::FetchAlbumArt { url, key } => spawn_fetch_album_art(
                            resp.clone(),
                            uploader.clone(),
                            url,
                            key,
                        ),
                        WorkerCommand::FetchAccent { image_hex } => {
                            spawn_fetch_accent(resp.clone(), session.clone(), image_hex)
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
                        WorkerCommand::Shutdown => break,
                    }
                }
            });
        });

        Self { cmd_tx, resp_rx, handle: Some(handle) }
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
    pub fn fetch_album_art(&self, url: String, key: String) {
        let _ = self.cmd_tx.send(WorkerCommand::FetchAlbumArt { url, key });
    }
    pub fn fetch_accent(&self, image_hex: String) {
        let _ = self.cmd_tx.send(WorkerCommand::FetchAccent { image_hex });
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
    #[allow(dead_code)]
    pub fn shutdown(mut self) {
        let _ = self.cmd_tx.send(WorkerCommand::Shutdown);
        if let Some(h) = self.handle.take() { let _ = h.join(); }
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

/// Fetch Spotify's own extracted accent colour for a cover via the
/// librespot session's `/extended-metadata` endpoint (`EXTRACTED_COLOR`
/// extension). This is the exact colour the official client tints its
/// now-playing UI with — far cleaner than our pixel-average fallback.
/// No-ops silently if the session isn't up yet or the cover has no
/// extracted colour (UI keeps the art-derived accent).
fn spawn_fetch_accent(
    resp: Responder,
    session_slot: Arc<AsyncMutex<Option<Session>>>,
    image_hex: String,
) {
    tokio::spawn(async move {
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

fn spawn_playback(access_token: String, cmd: PlaybackCmd) {
    tokio::spawn(async move {
        let result = match cmd.clone() {
            PlaybackCmd::Play => api::play(&access_token).await,
            PlaybackCmd::Pause => api::pause(&access_token).await,
            PlaybackCmd::Next => api::next_track(&access_token).await,
            PlaybackCmd::Prev => api::previous_track(&access_token).await,
            PlaybackCmd::Shuffle(on) => api::set_shuffle(&access_token, on).await,
            PlaybackCmd::Repeat(mode) => api::set_repeat(&access_token, mode).await,
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
