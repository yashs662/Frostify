#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod album_art;
mod api;
mod auth;
mod cluster_listener;
mod constants;
mod disk_cache;
mod errors;
mod null_sink;
mod spirc_bootstrap;
mod spotify_session;
mod ui;
mod worker;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::{Duration, Instant};

use frostify_gfx::{App, Curve, ImageHandle, Signal, TextSignal, Timeline};

use crate::api::{CurrentlyPlaying, HomeData, TrackDetails, track_id_from_uri};
use crate::auth::oauth::SpotifyAuthResponse;
use crate::ui::View;
use crate::ui::theme;
use crate::worker::{Worker, WorkerResponse};

const W: u32 = 1280;
const H: u32 = 780;

struct AppState {
    view: Cell<View>,
    auth: RefCell<Option<SpotifyAuthResponse>>,
    home: RefCell<HomeData>,
    player: RefCell<Option<CurrentlyPlaying>>,
    /// `/v1/tracks/{id}` results keyed by bare track ID. Cluster
    /// updates carry only `artist_uri`, not the resolved name, so we
    /// fetch+cache once per track and overlay onto the player view.
    track_details: RefCell<HashMap<String, TrackDetails>>,
    /// Cache keys currently in flight — gate so we don't dispatch a
    /// second fetch for the same cover while the first is still
    /// resolving. We deliberately do *not* keep a long-lived URL→handle
    /// cache: the GPU atlas can only hold a handful of covers and evicts
    /// any not currently rendered, so a cached handle for a non-displayed
    /// track goes dangling (the "rapid-switch loses all art" bug). The
    /// disk cache (raw bytes) makes re-decode cheap, and the only handles
    /// we render — `backdrop_prev`/`backdrop_curr` — are always tree-live
    /// and so never evicted. So we re-fetch (disk-backed) per real track
    /// change instead.
    art_inflight: RefCell<HashSet<String>>,
    /// `cache_key` of the cover currently promoted into the backdrop, so
    /// repeated PlayerState pushes for the *same* track (progress ticks)
    /// don't re-dispatch a fetch.
    shown_art_key: RefCell<Option<String>>,
    /// Outgoing backdrop layer — the previous track's art. A reactive
    /// `Signal` (not a plain field) so `promote_backdrop` can swap it via
    /// the lib's image-handle bind, updating every node bound to it
    /// (backdrop + both panels) WITHOUT a scene rebuild.
    backdrop_prev: Signal<Option<ImageHandle>>,
    /// Incoming backdrop layer — the current track's art, fading in.
    backdrop_curr: Signal<Option<ImageHandle>>,
    /// 0 → 1 crossfade progress between `backdrop_prev` and
    /// `backdrop_curr`. Driven directly by `frostify_gfx::Timeline`:
    /// `promote_backdrop` resets to 0 and `timeline.start(...)`
    /// tweens to 1 over `CROSSFADE_DURATION`. The scene closure binds
    /// per-layer alpha through `Computed<[f32; 4]>` derived from this
    /// signal, so the lib drives the redraw cadence — no manual
    /// rebuild ticking.
    crossfade_t: Signal<f32>,
    /// Separate, faster crossfade for the foreground panel art (now-
    /// playing cover + player-bar thumb). They're small and in focus, so
    /// a snappy swap reads better; the large blurred backdrop + accent
    /// deliberately lag behind on the slower `crossfade_t`/accent tween
    /// so the ambient colour catches up gradually.
    panel_crossfade_t: Signal<f32>,
    /// Dominant colour of the current track's art, driving the
    /// accent-tinted UI (play pill, active toggles, login button).
    /// Tweened directly by the timeline in `promote_backdrop` so the
    /// colour cross-fades synchronously with the backdrop crossfade.
    accent: Signal<[f32; 4]>,
    /// Live track title + artist as reactive text. Handlers update these
    /// (`PlayerState` for both, `TrackDetails` for the artist) so the
    /// labels refresh via the lib's text bind without a scene rebuild —
    /// in particular this kills the `TrackDetails` rebuild that used to
    /// land mid-crossfade.
    track_title: TextSignal,
    track_artist: TextSignal,
    /// Reactive playback state driving the player bar without rebuilds:
    /// `is_playing` → play/pause glyph (image bind), `shuffle`/`repeat_on`
    /// → toggle tints (colour bind), `progress` (0..=1) → bar fill width
    /// (% width bind).
    is_playing: Signal<bool>,
    shuffle: Signal<bool>,
    repeat_on: Signal<bool>,
    progress: Signal<f32>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: Cell::default(),
            auth: RefCell::default(),
            home: RefCell::default(),
            player: RefCell::default(),
            track_details: RefCell::default(),
            art_inflight: RefCell::default(),
            shown_art_key: RefCell::default(),
            backdrop_prev: Signal::new(None),
            backdrop_curr: Signal::new(None),
            crossfade_t: Signal::new(1.0),
            panel_crossfade_t: Signal::new(1.0),
            accent: Signal::new(theme::ACCENT),
            track_title: TextSignal::new("\u{2014}"),
            track_artist: TextSignal::new(""),
            is_playing: Signal::new(false),
            shuffle: Signal::new(false),
            repeat_on: Signal::new(false),
            progress: Signal::new(0.0),
        }
    }
}

/// Tween key namespace. Each timeline tween is identified by a `u32`
/// and replacing one with the same key restarts smoothly mid-flight.
const TWEEN_KEY_CROSSFADE: u32 = 0x0001_0001;
const TWEEN_KEY_ACCENT: u32 = 0x0001_0002;
const TWEEN_KEY_PANEL: u32 = 0x0001_0003;
const TWEEN_KEY_PROGRESS: u32 = 0x0001_0004;

/// How long the backdrop crossfade + accent colour transition takes on
/// track change. An ambient cross-dissolve — the previous cover fades
/// out as the next fades in over this window. 600 ms read as an abrupt
/// snap, 3 s dragged; ~1.5 s is the sweet spot.
const CROSSFADE_DURATION: Duration = Duration::from_millis(1500);

/// Foreground panel-art crossfade — deliberately much snappier than the
/// backdrop so the cover/thumb feel responsive on track change while the
/// big blurred backdrop + accent catch up behind them.
const PANEL_CROSSFADE_DURATION: Duration = Duration::from_millis(450);


fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(
        "info,wgpu_hal=warn,wgpu_core=warn,frostify=debug,frostify_gfx=debug",
    ))
    .init();

    let state = Rc::new(AppState::default());
    if std::env::var_os("FROSTIFY_FORCE_HOME").is_some() {
        state.view.set(View::Home);
    }

    let mut app = App::new("Frostify", W, H).decorations(false).capture_from_env();
    let icons = std::rc::Rc::new(ui::icon::load_all(&mut app));
    let rebuild = app.rebuild_token();
    let worker = Rc::new(Worker::new(app.wake_handle(), app.uploader()));
    worker.try_load_tokens();

    let on_login: Rc<dyn Fn()> = {
        let worker = worker.clone();
        Rc::new(move || worker.start_oauth())
    };

    let app = {
        let state = state.clone();
        let on_login = on_login.clone();
        let icons = icons.clone();
        app.scene(move |s| match state.view.get() {
            View::Splash | View::Login => {
                let checking = matches!(state.view.get(), View::Splash);
                ui::login::build(s, &icons, on_login.clone(), checking)
            }
            View::Home => {
                let view = ui::home::HomeView {
                    backdrop_prev: &state.backdrop_prev,
                    backdrop_curr: &state.backdrop_curr,
                    crossfade_t: &state.crossfade_t,
                    panel_crossfade_t: &state.panel_crossfade_t,
                    accent: &state.accent,
                    title: &state.track_title,
                    artist: &state.track_artist,
                    is_playing: &state.is_playing,
                    shuffle: &state.shuffle,
                    repeat_on: &state.repeat_on,
                    progress: &state.progress,
                };
                ui::home::build(s, &icons, &state.home.borrow(), &view)
            }
        })
    };

    let state_for_frame = state.clone();
    let worker_for_frame = worker.clone();
    let rebuild_for_frame = rebuild.clone();
    let app = app.on_frame(move |_ctx, tl, now| {
        while let Some(resp) = worker_for_frame.poll() {
            handle_worker_response(
                &state_for_frame,
                &rebuild_for_frame,
                &worker_for_frame,
                tl,
                now,
                resp,
            );
        }
    });

    app.run()
}

/// Promote `curr` art for the given handle if it's different from the
/// one we're already showing. Kicks off a crossfade by stashing the
/// outgoing handle in `prev`, snapping `crossfade_t = 0`, and starting
/// a timeline tween to `1.0` over `CROSSFADE_DURATION`. The accent
/// rides the same timeline so colour and backdrop cross-fade in
/// lock-step. `accent = None` keeps the previous accent so a cache-
/// miss doesn't flash to the default.
///
/// The lib's timeline pump drives the tween at 60 Hz; the scene's
/// `Computed` colour binds map `crossfade_t` to per-layer alphas and
/// `process_binds` (per about_to_wait tick) pushes the new values to
/// the tree. No manual rebuild cadence.
fn promote_backdrop(
    state: &Rc<AppState>,
    next: ImageHandle,
    accent: Option<[f32; 4]>,
    timeline: &mut Timeline,
    now: Instant,
) {
    let current = state.backdrop_curr.get();
    let same_image = current == Some(next);
    if !same_image {
        // Swap handles via the reactive signals — the lib's image-handle
        // bind pushes them to every bound node (backdrop + panels) on the
        // next `process_binds`, no scene rebuild. Outgoing stays opaque,
        // incoming fades in over it (see `home::fade_in_alpha`).
        state.backdrop_prev.set(current);
        state.backdrop_curr.set(Some(next));
        state.crossfade_t.set(0.0);
        timeline.start(
            TWEEN_KEY_CROSSFADE,
            state.crossfade_t.clone(),
            1.0,
            Curve::EaseInOut,
            CROSSFADE_DURATION,
            now,
        );
        // Foreground cover/thumb ride a separate, faster tween so they
        // swap snappily while the backdrop + accent lag behind.
        state.panel_crossfade_t.set(0.0);
        timeline.start(
            TWEEN_KEY_PANEL,
            state.panel_crossfade_t.clone(),
            1.0,
            Curve::EaseInOut,
            PANEL_CROSSFADE_DURATION,
            now,
        );
    }
    if let Some(c) = accent {
        timeline.start(
            TWEEN_KEY_ACCENT,
            state.accent.clone(),
            c,
            Curve::EaseInOut,
            CROSSFADE_DURATION,
            now,
        );
    }
}

fn handle_worker_response(
    state: &Rc<AppState>,
    rebuild: &Rc<Cell<bool>>,
    worker: &Rc<Worker>,
    timeline: &mut Timeline,
    now: Instant,
    resp: WorkerResponse,
) {
    match resp {
        WorkerResponse::OAuthStarted { auth_url } => {
            log::info!("opening browser for OAuth");
            if let Err(e) = webbrowser::open(&auth_url) {
                log::error!("open browser: {e}");
            }
        }
        WorkerResponse::OAuthComplete { auth } | WorkerResponse::TokensLoaded { auth } => {
            log::info!("auth ok — switching to Home");
            worker.fetch_home(auth.access_token.clone());
            worker.connect_spotify_session(auth.access_token.clone());
            *state.auth.borrow_mut() = Some(auth);
            if state.view.get() != View::Home {
                state.view.set(View::Home);
                rebuild.set(true);
            }
        }
        WorkerResponse::OAuthFailed { error } => {
            log::error!("OAuth failed: {error}");
            if state.view.get() != View::Login {
                state.view.set(View::Login);
                rebuild.set(true);
            }
        }
        WorkerResponse::NoStoredTokens => {
            log::info!("no stored tokens — showing Login");
            if state.view.get() != View::Login {
                state.view.set(View::Login);
                rebuild.set(true);
            }
        }
        WorkerResponse::HomeData { data } => {
            log::info!(
                "home data ready: playlists={} recent={}",
                data.playlists.len(),
                data.recent.len()
            );
            *state.home.borrow_mut() = data;
            rebuild.set(true);
        }
        WorkerResponse::SpotifySessionConnected => {
            log::info!("librespot session ready — seeding initial /me/player state");
            if let Some(token) = state.auth.borrow().as_ref().map(|a| a.access_token.clone()) {
                worker.seed_player_state(token);
            }
        }
        WorkerResponse::SpotifySessionFailed { error } => {
            log::warn!("librespot session failed: {error}. Falling back to Web API polling.");
        }
        WorkerResponse::PlayerState { mut player } => {
            // Overlay cached track details (artist) and request a fetch
            // for any track we haven't resolved yet. The cluster's
            // `ProvidedTrack.metadata` only carries `artist_uri`, so the
            // artist name comes from `/v1/tracks/{id}`.
            if let Some(p) = player.as_mut() {
                if let Some(id) = track_id_from_uri(&p.track_id) {
                    let details = state.track_details.borrow();
                    match details.get(id) {
                        Some(d) if !d.artist.is_empty() => p.artist = d.artist.clone(),
                        _ => {
                            drop(details);
                            if let Some(token) =
                                state.auth.borrow().as_ref().map(|a| a.access_token.clone())
                            {
                                worker.fetch_track_details(token, id.to_string());
                            }
                        }
                    }
                }
                // Dispatch an album-art fetch when the cover actually
                // changes. Skip when it's already what's on screen (same
                // track, just a progress tick) or a fetch is already in
                // flight. The fetch is disk-backed, so re-loading a cover
                // we've seen before is cheap and yields a fresh, tree-live
                // handle — see `art_inflight` doc for why we don't cache
                // handles across tracks.
                if let Some(url) = p.album_image_url.as_ref() {
                    let key = album_art::cache_key(url);
                    let already_shown = state.shown_art_key.borrow().as_deref() == Some(key.as_str());
                    let inflight = state.art_inflight.borrow().contains(&key);
                    if !already_shown && !inflight {
                        state.art_inflight.borrow_mut().insert(key.clone());
                        worker.fetch_album_art(url.clone(), key);
                    }
                }
            }
            // Push every dynamic field into its reactive signal (all
            // dedup'd, so a same-track progress tick only bumps what
            // changed). Title/artist → text binds, is_playing → play/pause
            // image bind, shuffle/repeat → tint colour binds, progress →
            // % width bind. Nothing here needs a scene rebuild anymore.
            match player.as_ref() {
                Some(p) => {
                    state.track_title.set(p.name.as_str());
                    state.track_artist.set(p.artist.as_str());
                    state.is_playing.set(p.is_playing);
                    state.shuffle.set(p.shuffle);
                    state
                        .repeat_on
                        .set(!matches!(p.repeat, crate::api::RepeatMode::Off));
                    // Progress: snap to the live position, then (if
                    // playing) tween the signal to 1.0 over the remaining
                    // duration so the bar advances smoothly between cluster
                    // pushes — the timeline keeps the loop awake and the
                    // % width bind follows. Paused → stop the tween, the
                    // bar holds. Cluster pushes (seek/play/pause/track)
                    // restart it from the fresh position.
                    let live = p.live_progress_ms().min(p.duration_ms);
                    let frac = if p.duration_ms > 0 {
                        live as f32 / p.duration_ms as f32
                    } else {
                        0.0
                    };
                    state.progress.set(frac);
                    if p.is_playing && p.duration_ms > 0 {
                        let remaining = p.duration_ms.saturating_sub(live);
                        timeline.start(
                            TWEEN_KEY_PROGRESS,
                            state.progress.clone(),
                            1.0,
                            Curve::Linear,
                            Duration::from_millis(remaining),
                            now,
                        );
                    } else {
                        timeline.stop(TWEEN_KEY_PROGRESS);
                    }
                }
                None => {
                    state.track_title.set("\u{2014}");
                    state.track_artist.set("");
                    state.is_playing.set(false);
                    state.shuffle.set(false);
                    state.repeat_on.set(false);
                    timeline.stop(TWEEN_KEY_PROGRESS);
                    state.progress.set(0.0);
                }
            }
            *state.player.borrow_mut() = player;
        }
        WorkerResponse::AlbumArtReady { key, handle, accent } => {
            state.art_inflight.borrow_mut().remove(&key);
            // Promote into the crossfade only if this cover is still the
            // live track's (a rapid switch may have moved on before the
            // upload landed — that handle just stays an orphan the atlas
            // will evict). No handle cache: the fresh handle is tree-live
            // once promoted, so it survives atlas eviction.
            let matches_current = state
                .player
                .borrow()
                .as_ref()
                .and_then(|p| p.album_image_url.as_ref().map(|u| album_art::cache_key(u)))
                .map(|k| k == key)
                .unwrap_or(false);
            if matches_current {
                // No rebuild: promote swaps the handles via the reactive
                // image-handle binds and starts the crossfade tween, both
                // pumped by the lib without re-running the scene closure.
                promote_backdrop(state, handle, Some(accent), timeline, now);
                *state.shown_art_key.borrow_mut() = Some(key);
            }
        }
        WorkerResponse::AlbumArtFailed { key } => {
            state.art_inflight.borrow_mut().remove(&key);
        }
        WorkerResponse::TrackDetails { details } => {
            let track_id = details.track_id.clone();
            state
                .track_details
                .borrow_mut()
                .insert(track_id.clone(), details.clone());
            // Patch the live player view if it still matches, and push the
            // artist into the reactive text signal — updates the label via
            // the text bind, no rebuild (this is the one that used to land
            // mid-crossfade).
            let mut player = state.player.borrow_mut();
            if let Some(p) = player.as_mut()
                && track_id_from_uri(&p.track_id) == Some(track_id.as_str())
            {
                p.artist = details.artist.clone();
                state.track_artist.set(details.artist.as_str());
            }
        }
    }
}

