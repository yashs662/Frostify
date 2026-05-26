use crate::album_art;
use crate::api::{self, CurrentlyPlaying, HomeData, TrackDetails};
use crate::disk_cache;
use crate::auth::oauth::{self, SpotifyAuthResponse, listen_for_callback, refresh_token};
use crate::auth::token_manager::{self, StoredTokens};
use crate::{cluster_listener, spirc_bootstrap, spotify_session};
use frostify_gfx::{ImageHandle, Uploader, WakeHandle};
use librespot_connect::Spirc;
use librespot_core::Session;
use librespot_core::authentication::Credentials;
use log::{debug, error, info, warn};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::mpsc::{self as tmpsc, UnboundedSender};

/// Cap the longest side of decoded album art. The 2048² atlas holds
/// several covers at this size; 640 matches Spotify's largest variant
/// and keeps the now-playing pane (~308 px logical, ~616 px @2× DPI) and
/// full-window backdrop crisp. The 56 px player-bar thumb downsamples
/// from the same handle.
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
    ConnectSpotifySession { access_token: String },
    #[allow(dead_code)]
    Shutdown,
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
                        WorkerCommand::ConnectSpotifySession { access_token } => spawn_connect_session(
                            resp.clone(),
                            session.clone(),
                            spirc.clone(),
                            access_token,
                        ),
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
    pub fn connect_spotify_session(&self, access_token: String) {
        let _ = self
            .cmd_tx
            .send(WorkerCommand::ConnectSpotifySession { access_token });
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
        let (profile, playlists, recent) = tokio::join!(
            api::get_me(&access_token),
            api::get_playlists(&access_token),
            api::get_recently_played(&access_token),
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
        info!(
            "home data: profile={} playlists={} recent={}",
            data.profile.is_some(),
            data.playlists.len(),
            data.recent.len()
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

fn spawn_fetch_track_details(resp: Responder, access_token: String, track_id: String) {
    tokio::spawn(async move {
        match api::get_track(&access_token, &track_id).await {
            Ok(details) => resp.send(WorkerResponse::TrackDetails { details }),
            Err(e) => warn!("get_track({track_id}) failed: {e}"),
        }
    });
}

/// Fetch the raw image bytes for `url`, retrying once on transient
/// failure. Returns `None` only after both attempts fail.
async fn fetch_art_bytes(url: &str) -> Option<Vec<u8>> {
    for attempt in 1..=2 {
        match reqwest::get(url).await.and_then(|r| r.error_for_status()) {
            Ok(r) => match r.bytes().await {
                Ok(b) => return Some(b.to_vec()),
                Err(e) => warn!("album art read body failed ({url}) attempt {attempt}: {e}"),
            },
            Err(e) => warn!("album art fetch failed ({url}) attempt {attempt}: {e}"),
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
            None => match fetch_art_bytes(&url).await {
                Some(b) => (b, true),
                None => {
                    resp.send(WorkerResponse::AlbumArtFailed { key });
                    return;
                }
            },
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
