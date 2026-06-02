#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod album_art;
mod api;
mod auth;
mod cluster_listener;
mod constants;
#[cfg(feature = "automation")]
mod debug_config;
mod disk_cache;
mod errors;
mod extracted_color;
mod null_sink;
mod prefs;
mod spirc_bootstrap;
mod spotify_session;
mod ui;
mod worker;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::{Duration, Instant};

use frostify_gfx::{App, Curve, ImageHandle, Overlay, Signal, TextSignal, Timeline};

use crate::api::{
    CurrentlyPlaying, HomeData, PlaylistDetail, PlaylistTrack, RepeatMode, TrackDetails,
    track_id_from_uri,
};
use crate::auth::oauth::SpotifyAuthResponse;
use crate::prefs::{StoredPlayer, UserPreferences};
use crate::ui::home::{NavFn, PlayFn, PlayerAction};
use crate::ui::playlist::{self, PlaylistRow, PlaylistViewData, RowBuf};
use crate::ui::tokens;
use crate::ui::{MainNav, View};
use crate::worker::{PlaybackCmd, Worker, WorkerResponse};

const W: u32 = 1280;
const H: u32 = 780;

/// A loaded playlist plus the wall-clock at which it was fetched —
/// drives the in-memory TTL cache so re-opening a playlist within
/// [`PLAYLIST_TTL`] reuses the data instead of re-hitting the Web API.
struct CachedPlaylist {
    detail: PlaylistDetail,
    fetched: Instant,
}

/// The playlist currently open in the centre pane. Holds the metadata
/// plus a **live, growable** row buffer the streaming worker pages fill —
/// the view's `lazy_list` reads it on scroll, so later pages appear
/// without a rebuild. `total` drives the list length from the first
/// response so the scrollbar is correct before everything has streamed.
struct OpenPlaylist {
    liked: bool,
    name: String,
    owner: String,
    image_url: Option<String>,
    context_uri: Option<String>,
    total: u32,
    rows: RowBuf,
    /// Metadata not yet arrived (header shows the sidebar-known name).
    loading: bool,
    /// Every page has streamed in.
    complete: bool,
}

struct AppState {
    view: Cell<View>,
    auth: RefCell<Option<SpotifyAuthResponse>>,
    home: RefCell<HomeData>,
    /// What the centre pane is showing. Mutated by `navigate`, read by
    /// the scene closure to pick Home-feed vs playlist content.
    nav: RefCell<MainNav>,
    /// Playlist detail TTL cache (id → detail + fetch time). Liked Songs
    /// lives here under `api::LIKED_SONGS_ID`. Keeps repeated opens from
    /// spamming `/v1/playlists/{id}`; entries past [`PLAYLIST_TTL`] are
    /// re-fetched so edits show up.
    playlist_cache: RefCell<HashMap<String, CachedPlaylist>>,
    /// Playlist ids with a fetch in flight — gate so navigating back and
    /// forth doesn't dispatch duplicate loads.
    playlist_inflight: RefCell<HashSet<String>>,
    /// The playlist open in the centre pane (live streaming buffer).
    open_playlist: RefCell<Option<OpenPlaylist>>,
    /// 0 → 1 slide/fade progress for the centre-pane content, retween'd
    /// on every nav change (see `navigate`). Parks at 1.0 (settled).
    main_t: Signal<f32>,
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
    /// Per-URL (cache_key) reactive handle for every cover shown anywhere
    /// in Home — tiles bind their image to the signal so an art arrival
    /// repaints just the affected nodes, no scene rebuild. Pre-allocated
    /// when `HomeData` lands; the worker fills each as the fetch resolves.
    /// Cleared and rebuilt on the next `HomeData`.
    home_art: RefCell<HashMap<String, Signal<Option<ImageHandle>>>>,
    /// Outgoing backdrop layer — the previous track's art. A reactive
    /// `Signal` (not a plain field) so `promote_backdrop` can swap it via
    /// the lib's image-handle bind, updating every node bound to it
    /// (backdrop + both panels) WITHOUT a scene rebuild.
    backdrop_prev: Signal<Option<ImageHandle>>,
    /// Incoming backdrop layer — the current track's art, fading in.
    backdrop_curr: Signal<Option<ImageHandle>>,
    /// 0 → 1 crossfade progress between `backdrop_prev` and
    /// `backdrop_curr`. Driven by `frostify_gfx::Timeline`:
    /// `promote_backdrop` resets to 0 and `timeline.animate(&crossfade_t,
    /// 1.0, …)` over `CROSSFADE_DURATION`. The incoming backdrop image is
    /// a `.layer_opacity(crossfade_t)` compositor layer, so the lib drives
    /// its composite opacity each frame (composite-only, no per-frame
    /// image re-raster) — the lib also owns the redraw cadence, no manual
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
    /// Spotify's own extracted accent per cover (keyed by image hex).
    /// The authoritative source for `accent` — overrides the pixel-
    /// average fallback. Cached so a late `AlbumArtReady` promotes the
    /// right colour regardless of which of the two requests resolves
    /// first (see `handle_worker_response`).
    spotify_accent: RefCell<HashMap<String, [f32; 4]>>,
    /// The settings modal. A self-contained `Overlay` widget: owns its
    /// fade opacity + timeline key, blocks input beneath it, and costs
    /// nothing when closed. Opened/closed via `settings.open/close`.
    settings: Overlay,
    /// Whether to show the looping Canvas video in now-playing. Persisted
    /// via prefs; the playback pipeline that consumes it is not built yet.
    show_canvas: Signal<bool>,
    /// Persistent user preferences. Mutated in-place when the user
    /// resizes panels / tweaks settings; written to disk by the
    /// debounced save tick in `on_frame`.
    prefs: RefCell<UserPreferences>,
    /// Resizable panel widths in logical px. Driven by the splitters
    /// (via `width_px_bind`), persisted into `prefs.panels` on save.
    sidebar_w: Signal<f32>,
    now_playing_w: Signal<f32>,
    /// Timestamp of the earliest unsaved pref change since the last
    /// save. `None` = clean. `tick_prefs_save` writes when the
    /// elapsed time crosses `PREFS_DEBOUNCE`.
    prefs_dirty_since: Cell<Option<Instant>>,
    /// Throwaway signal whose only purpose is to anchor a timeline
    /// tween that keeps the loop awake long enough for the debounced
    /// save deadline to fire. The value itself is never read or
    /// rendered.
    prefs_save_anchor: Signal<f32>,
}

impl AppState {
    fn from_prefs(prefs: UserPreferences) -> Self {
        let sidebar_w = Signal::new(prefs.panels.sidebar_w);
        let now_playing_w = Signal::new(prefs.panels.now_playing_w);
        // Seed the player chrome from the persisted snapshot so cold
        // start renders the last-played track immediately instead of a
        // dash placeholder. The first live cluster push overwrites
        // these; if Spotify has nothing playing on launch, the snapshot
        // stays visible and matches the user's last session.
        let (title, artist, progress) = match prefs.last_player.as_ref() {
            Some(p) => {
                let frac = if p.duration_ms > 0 {
                    (p.progress_ms as f32 / p.duration_ms as f32).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                (p.name.as_str(), p.artist.as_str(), frac)
            }
            None => ("\u{2014}", "", 0.0),
        };
        Self {
            view: Cell::default(),
            auth: RefCell::default(),
            home: RefCell::default(),
            nav: RefCell::default(),
            playlist_cache: RefCell::default(),
            playlist_inflight: RefCell::default(),
            open_playlist: RefCell::default(),
            main_t: Signal::new(1.0),
            player: RefCell::default(),
            track_details: RefCell::default(),
            art_inflight: RefCell::default(),
            shown_art_key: RefCell::default(),
            home_art: RefCell::default(),
            backdrop_prev: Signal::new(None),
            backdrop_curr: Signal::new(None),
            crossfade_t: Signal::new(1.0),
            panel_crossfade_t: Signal::new(1.0),
            accent: Signal::new(tokens::ACCENT),
            track_title: TextSignal::new(title),
            track_artist: TextSignal::new(artist),
            is_playing: Signal::new(false),
            shuffle: Signal::new(false),
            repeat_on: Signal::new(false),
            progress: Signal::new(progress),
            spotify_accent: RefCell::default(),
            settings: Overlay::new(),
            show_canvas: Signal::new(prefs.show_canvas),
            prefs: RefCell::new(prefs),
            sidebar_w,
            now_playing_w,
            prefs_dirty_since: Cell::new(None),
            prefs_save_anchor: Signal::new(0.0),
        }
    }
}

// Tweens are keyed by signal identity via `timeline.animate(&sig, …)` /
// `timeline.stop_for(&sig)` — no hand-authored tween keys.

/// Centre-pane content transition duration on nav change.
const MAIN_NAV_DURATION: Duration = Duration::from_millis(260);

/// How long a cached playlist stays fresh before a re-open re-fetches it.
/// Long enough to make back-and-forth navigation free, short enough that
/// edits made elsewhere show up within a few minutes.
const PLAYLIST_TTL: Duration = Duration::from_secs(300);

/// How long to wait after the last pref mutation before writing the
/// file. Smooths out splitter-drag bursts (~60 events/sec) into a
/// single disk write per drag.
const PREFS_DEBOUNCE: Duration = Duration::from_millis(500);

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
    // Debug-only launch config (REMOVABLE — `automation` feature). Parsed
    // before logging so it can override the filter.
    #[cfg(feature = "automation")]
    let debug_cfg = debug_config::from_args();

    // `frostify_gfx=debug` would spam per-frame `[loop] WaitUntil(...)` +
    // active-tick lines while the progress tween runs (60 fps during
    // playback). Drop the lib to `info`; keep `frostify` at debug.
    let default_filter = "info,wgpu_hal=warn,wgpu_core=warn,frostify=debug,frostify_gfx=info";
    #[cfg(feature = "automation")]
    let filter = debug_cfg
        .as_ref()
        .and_then(|c| c.log_filter.clone())
        .unwrap_or_else(|| default_filter.to_string());
    #[cfg(not(feature = "automation"))]
    let filter = default_filter.to_string();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter)).init();

    // Load persisted preferences before any window work — initial size
    // + panel widths come from here. Fail-soft: a missing or malformed
    // file yields defaults so first launch always boots.
    let mut prefs = UserPreferences::load();
    // Snap any out-of-range panel widths back into a valid state —
    // handles corrupted JSON, schema additions where MIN/MAX moved past
    // a saved value, and the float-drift edge cases. Values close to
    // the collapsed snap stay collapsed; everything else clamps to
    // `[MIN, MAX]`.
    prefs.panels.sidebar_w = prefs::clamp_panel_width(
        prefs.panels.sidebar_w,
        tokens::SIDEBAR_MIN,
        tokens::SIDEBAR_MAX,
        tokens::SIDEBAR_COLLAPSED,
    );
    prefs.panels.now_playing_w = prefs::clamp_panel_width(
        prefs.panels.now_playing_w,
        tokens::NOW_PLAYING_MIN,
        tokens::NOW_PLAYING_MAX,
        0.0, // now-playing collapses fully
    );
    let win_w = prefs.window.width.unwrap_or(W);
    let win_h = prefs.window.height.unwrap_or(H);
    // Debug config may pin the window size (REMOVABLE — shadows the above).
    #[cfg(feature = "automation")]
    let (win_w, win_h) = match debug_cfg.as_ref().and_then(|c| c.window) {
        Some([w, h]) => (w, h),
        None => (win_w, win_h),
    };

    let state = Rc::new(AppState::from_prefs(prefs));
    let force_home = std::env::var_os("FROSTIFY_FORCE_HOME").is_some();
    #[cfg(feature = "automation")]
    let force_home = force_home || debug_cfg.as_ref().map(|c| c.force_home).unwrap_or(false);
    if force_home {
        state.view.set(View::Home);
    }

    let mut app = App::new("Frostify", win_w, win_h)
        .decorations(false)
        .window_corner_radius(tokens::R_XL)
        .capture_from_env();
    let icons = std::rc::Rc::new(ui::icon::load_all(&mut app));
    let rebuild = app.rebuild_token();
    let worker = Rc::new(Worker::new(app.wake_handle(), app.uploader()));
    worker.try_load_tokens();

    // Re-hydrate the album-art backdrop from the persisted last track —
    // the disk cache makes this near-instant on relaunch, so the
    // backdrop is already populated by the time the user sees Home
    // instead of waiting for the first live cluster push.
    if let Some(p) = state.prefs.borrow().last_player.as_ref()
        && let Some(url) = p.album_image_url.as_ref()
    {
        let key = album_art::cache_key(url);
        state.art_inflight.borrow_mut().insert(key.clone());
        worker.fetch_album_art(url.clone(), key);
    }

    let on_login: Rc<dyn Fn()> = {
        let worker = worker.clone();
        Rc::new(move || worker.start_oauth())
    };

    // Transport dispatcher: maps a player-bar button intent to an
    // optimistic signal flip (instant visual feedback) plus a Web API
    // playback command on the worker. The dealer cluster subscription
    // pushes the authoritative state back shortly after, correcting the
    // optimistic guess (or reverting it if the command failed, e.g. no
    // active device). Reads the live token at fire time so it survives a
    // token refresh.
    let on_action: Rc<dyn Fn(PlayerAction)> = {
        let state = state.clone();
        let worker = worker.clone();
        Rc::new(move |action| {
            let Some(token) = state.auth.borrow().as_ref().map(|a| a.access_token.clone()) else {
                log::warn!("playback action ignored — no auth token");
                return;
            };
            match action {
                PlayerAction::PlayPause => {
                    let was_playing = state.is_playing.get();
                    state.is_playing.set(!was_playing);
                    worker.playback(
                        token,
                        if was_playing { PlaybackCmd::Pause } else { PlaybackCmd::Play },
                    );
                }
                PlayerAction::Next => worker.playback(token, PlaybackCmd::Next),
                PlayerAction::Prev => worker.playback(token, PlaybackCmd::Prev),
                PlayerAction::ToggleShuffle => {
                    let next = !state.shuffle.get();
                    state.shuffle.set(next);
                    worker.playback(token, PlaybackCmd::Shuffle(next));
                }
                PlayerAction::CycleRepeat => {
                    // Off → Context → Track → Off, mirroring Spotify's
                    // repeat-button cycle. Drive off the live player's
                    // actual mode (not just the `repeat_on` bool) so the
                    // three-state cycle is correct.
                    let current = state
                        .player
                        .borrow()
                        .as_ref()
                        .map(|p| p.repeat)
                        .unwrap_or(RepeatMode::Off);
                    let next = match current {
                        RepeatMode::Off => RepeatMode::Context,
                        RepeatMode::Context => RepeatMode::Track,
                        RepeatMode::Track => RepeatMode::Off,
                    };
                    state.repeat_on.set(!matches!(next, RepeatMode::Off));
                    worker.playback(token, PlaybackCmd::Repeat(next));
                }
            }
        })
    };

    // Persist after the canvas toggle flips (debounced).
    let on_canvas_change: Rc<dyn Fn()> = {
        let state = state.clone();
        Rc::new(move || mark_prefs_dirty(&state, Instant::now()))
    };
    let sign_out: Rc<dyn Fn()> = {
        let state = state.clone();
        let rebuild = rebuild.clone();
        Rc::new(move || {
            if let Err(e) = crate::auth::token_manager::delete_tokens() {
                log::warn!("sign-out: failed to clear stored token: {e}");
            }
            *state.auth.borrow_mut() = None;
            // We're leaving Home — snap the modal shut (no fade against an
            // unmounted tree) so it isn't still up on next sign-in.
            state.settings.reset();
            state.view.set(View::Login);
            rebuild.set(true);
        })
    };

    // Centre-pane navigation: open a playlist (or Home). Receives the
    // `EventCtx` from the triggering click so it can start the entrance
    // transition tween at click time.
    let on_navigate: NavFn = {
        let state = state.clone();
        let worker = worker.clone();
        let rebuild = rebuild.clone();
        Rc::new(move |ctx, nav| navigate(&state, &rebuild, &worker, ctx.timeline, ctx.now, nav))
    };

    // Lazily fetch a track cover when its row scrolls into view. Created
    // once + cloned into the scene; gated so it never re-dispatches.
    let request_cover: playlist::CoverFn = {
        let state = state.clone();
        let worker = worker.clone();
        Rc::new(move |url| dispatch_cover(&state, &worker, url))
    };

    // Start playback of a playlist context / track selection. Optimistic
    // is_playing flip; the dealer cluster push corrects the real state.
    let on_play: PlayFn = {
        let state = state.clone();
        let worker = worker.clone();
        Rc::new(move |target| {
            let Some(token) = state.auth.borrow().as_ref().map(|a| a.access_token.clone()) else {
                log::warn!("play ignored — no auth token");
                return;
            };
            state.is_playing.set(true);
            worker.playback(token, PlaybackCmd::PlayContext(target));
        })
    };

    let app = {
        let state = state.clone();
        let on_login = on_login.clone();
        let icons = icons.clone();
        let on_action = on_action.clone();
        let on_canvas_change = on_canvas_change.clone();
        let sign_out = sign_out.clone();
        let on_navigate = on_navigate.clone();
        let on_play = on_play.clone();
        let request_cover = request_cover.clone();
        app.scene(move |s| match state.view.get() {
            View::Splash | View::Login => {
                let checking = matches!(state.view.get(), View::Splash);
                ui::login::build(s, &icons, on_login.clone(), checking)
            }
            View::Home => {
                // Splitter on_change → mark prefs dirty. The closure
                // captures an Rc-clone of state so it can outlive the
                // build call; debounced save in `on_frame` picks up
                // the new widths on the next deadline.
                let dirty_state = state.clone();
                let mark_dirty: Rc<dyn Fn()> =
                    Rc::new(move || mark_prefs_dirty(&dirty_state, Instant::now()));
                // Hold the nav borrow for the build. Build the playlist
                // view data (metadata + live row buffer handle) when a
                // playlist is open — cheap clones; the rows `Rc` is shared
                // with the streaming appends so new pages appear without
                // rebuilding this.
                let nav = state.nav.borrow();
                let playlist: Option<PlaylistViewData> = match &*nav {
                    MainNav::Playlist { .. } => {
                        state.open_playlist.borrow().as_ref().map(|o| {
                            let cover = o.image_url.as_ref().and_then(|u| {
                                state.home_art.borrow().get(&album_art::cache_key(u)).cloned()
                            });
                            PlaylistViewData {
                                name: o.name.clone(),
                                owner: o.owner.clone(),
                                total: o.total,
                                liked: o.liked,
                                loading: o.loading,
                                cover,
                                context_uri: o.context_uri.clone(),
                                rows: o.rows.clone(),
                                request_cover: request_cover.clone(),
                            }
                        })
                    }
                    MainNav::Home => None,
                };
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
                    sidebar_w: &state.sidebar_w,
                    now_playing_w: &state.now_playing_w,
                    nav: &nav,
                    playlist: playlist.as_ref(),
                    main_t: &state.main_t,
                    on_navigate: on_navigate.clone(),
                    on_play: on_play.clone(),
                    mark_dirty,
                    on_action: on_action.clone(),
                    settings: &state.settings,
                    show_canvas: &state.show_canvas,
                    on_canvas_change: on_canvas_change.clone(),
                    sign_out: sign_out.clone(),
                };
                ui::home::build(s, &icons, &state.home.borrow(), &state.home_art.borrow(), &view)
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
        tick_prefs_save(&state_for_frame, tl, now);
    });

    // Force a final prefs flush on app close — picks up any mouse-up
    // event we might have missed (e.g. drag released outside the
    // window) and persists the live player snapshot so the next
    // launch can re-hydrate the chrome immediately.
    let state_for_exit = state.clone();
    let app = app.on_exit(move || {
        snapshot_player_into_prefs(&state_for_exit);
        let mut prefs = state_for_exit.prefs.borrow_mut();
        prefs.panels.sidebar_w = state_for_exit.sidebar_w.get();
        prefs.panels.now_playing_w = state_for_exit.now_playing_w.get();
        prefs.show_canvas = state_for_exit.show_canvas.get();
        match prefs.save() {
            Ok(()) => log::info!("prefs flushed on exit"),
            Err(e) => log::warn!("prefs flush on exit failed: {e}"),
        }
    });

    // Attach a scripted-input run if the debug config carries one
    // (REMOVABLE — `automation` feature).
    #[cfg(feature = "automation")]
    let app = match debug_cfg.as_ref().map(|c| c.script()) {
        Some(script) if !script.steps.is_empty() => app.automation(script),
        _ => app,
    };

    app.run()
}

/// Copy the live player snapshot into prefs so the next launch can
/// re-hydrate the chrome. **Only writes when there's a live player** —
/// if the user closes before the first cluster push lands (or with
/// nothing playing), the previously persisted snapshot is preserved
/// instead of being wiped. That's the "sometimes works" failure mode
/// the naive overwrite caused: close fast → `state.player = None` →
/// snapshot cleared → next launch boots blank.
fn snapshot_player_into_prefs(state: &AppState) {
    if let Some(p) = state.player.borrow().as_ref() {
        state.prefs.borrow_mut().last_player = Some(StoredPlayer {
            track_id: p.track_id.clone(),
            name: p.name.clone(),
            artist: p.artist.clone(),
            album_image_url: p.album_image_url.clone(),
            progress_ms: p.live_progress_ms().min(p.duration_ms),
            duration_ms: p.duration_ms,
        });
    }
}

/// Mark the user prefs as dirty without writing — the actual save runs
/// later in `tick_prefs_save` after [`PREFS_DEBOUNCE`] of *quiescence*.
/// Slides the timestamp forward on every call so the debounce window
/// resets with each new event: a continuous splitter drag now produces
/// **one** save 500 ms after the drag ends, not a save every 500 ms
/// during the drag.
fn mark_prefs_dirty(state: &AppState, now: Instant) {
    state.prefs_dirty_since.set(Some(now));
}

/// Run from `on_frame`. When prefs have been dirty for at least
/// [`PREFS_DEBOUNCE`], snapshot the current panel widths back into the
/// prefs struct and write to disk. Otherwise, keep the loop awake by
/// re-anchoring a placeholder timeline tween so `on_frame` keeps firing
/// past the deadline even after the last user event.
fn tick_prefs_save(state: &AppState, timeline: &mut Timeline, now: Instant) {
    let Some(dirty_at) = state.prefs_dirty_since.get() else { return };
    let elapsed = now.saturating_duration_since(dirty_at);
    if elapsed >= PREFS_DEBOUNCE {
        // Snapshot resizable values that live in signals back into the
        // serialized prefs before writing. The player snapshot also
        // rides along so a mid-session crash doesn't lose the
        // currently-playing track on next launch.
        snapshot_player_into_prefs(state);
        {
            let mut prefs = state.prefs.borrow_mut();
            prefs.panels.sidebar_w = state.sidebar_w.get();
            prefs.panels.now_playing_w = state.now_playing_w.get();
            prefs.show_canvas = state.show_canvas.get();
        }
        match state.prefs.borrow().save() {
            Ok(()) => log::debug!("prefs saved"),
            Err(e) => log::warn!("prefs save failed: {e}"),
        }
        state.prefs_dirty_since.set(None);
        timeline.stop_for(&state.prefs_save_anchor);
    } else {
        // Keep the loop awake until the deadline. Restart the anchor
        // tween (idempotent — `animate` on the same signal replaces any
        // in-flight one) so this fires whether or not other input is
        // keeping the loop ticking.
        let remaining = PREFS_DEBOUNCE - elapsed + Duration::from_millis(50);
        state.prefs_save_anchor.set(0.0);
        timeline.animate(&state.prefs_save_anchor, 1.0, Curve::Linear, remaining, now);
    }
}

/// Promote `curr` art for the given handle if it's different from the
/// one we're already showing. Kicks off a crossfade by stashing the
/// outgoing handle in `prev`, snapping `crossfade_t = 0`, and starting
/// a timeline tween to `1.0` over `CROSSFADE_DURATION`. The accent
/// rides the same timeline so colour and backdrop cross-fade in
/// lock-step. `accent = None` keeps the previous accent so a cache-
/// miss doesn't flash to the default.
///
/// The lib's timeline pump drives the tween at 60 Hz; the incoming
/// backdrop's `.layer_opacity(crossfade_t)` layer picks up the new value
/// each tick as a composite-only opacity (no per-frame image re-raster),
/// and the panel `crossfaded_art` reads it through `Computed` colour
/// binds. No manual rebuild cadence.
/// Walk every image URL in `data` and ensure a reactive handle signal
/// exists per cache_key. Dispatches a worker fetch for each key that's
/// neither already in flight nor already resolved (signal carries
/// `Some`). Subsequent `AlbumArtReady` arrivals push their handle into
/// the matching signal — no scene rebuild.
fn prefetch_home_art(state: &AppState, worker: &Worker, data: &HomeData) {
    // Count items per section vs items that carry an image_url — Spotify
    // returns `images: []` for many artists/playlists, which is the
    // common reason a Home tile stays a placeholder.
    let (pl, pl_with) = count_with_image(&data.playlists, |p| p.image_url.is_some());
    let (rc, rc_with) = count_with_image(&data.recent, |t| t.album_image_url.is_some());
    let (ta, ta_with) = count_with_image(&data.top_artists, |a| a.image_url.is_some());
    let (tt, tt_with) = count_with_image(&data.top_tracks, |t| t.album_image_url.is_some());
    log::info!(
        "home art coverage: playlists {pl_with}/{pl}, recent {rc_with}/{rc}, \
         top_artists {ta_with}/{ta}, top_tracks {tt_with}/{tt}, \
         latest_release {}",
        if data.latest_release.as_ref().and_then(|a| a.image_url.as_ref()).is_some() { "1/1" } else { "0/1" },
    );

    let urls = data
        .playlists
        .iter()
        .filter_map(|p| p.image_url.as_ref())
        // Sidebar library icons fetch the tiny (64 px) tier separately —
        // distinct scdn key from the full-res home tile, so both load.
        .chain(data.playlists.iter().filter_map(|p| p.image_url_small.as_ref()))
        .chain(data.recent.iter().filter_map(|t| t.album_image_url.as_ref()))
        .chain(data.top_artists.iter().filter_map(|a| a.image_url.as_ref()))
        .chain(data.top_tracks.iter().filter_map(|t| t.album_image_url.as_ref()))
        .chain(data.latest_release.iter().filter_map(|a| a.image_url.as_ref()));
    let mut signals = state.home_art.borrow_mut();
    let mut inflight = state.art_inflight.borrow_mut();
    let mut dispatched = 0_usize;
    for url in urls {
        let key = album_art::cache_key(url);
        let sig = signals
            .entry(key.clone())
            .or_insert_with(|| Signal::new(None))
            .clone();
        // Already resolved (handle present) or in flight → skip.
        if sig.get().is_some() || inflight.contains(&key) {
            continue;
        }
        inflight.insert(key.clone());
        worker.fetch_album_art(url.clone(), key);
        dispatched += 1;
    }
    log::info!("dispatched {dispatched} new art fetches");
}

fn count_with_image<T>(items: &[T], has: impl Fn(&T) -> bool) -> (usize, usize) {
    let total = items.len();
    let with = items.iter().filter(|i| has(i)).count();
    (total, with)
}

/// Ease-out for the centre-pane entrance — fast start, gentle settle.
const NAV_CURVE: Curve = Curve::CubicBezier([0.16, 1.0, 0.3, 1.0]);

/// Switch the centre pane to `nav`. Ensures the target playlist is
/// loaded (TTL cache → fetch on miss/stale), flips the nav state, kicks
/// the slide/fade-in transition, and requests the one scene rebuild that
/// swaps the pane content. Navigation is the one place a deliberate
/// rebuild is correct (the content is structurally different); the
/// reactive path can't restructure the tree.
fn navigate(
    state: &Rc<AppState>,
    rebuild: &Rc<Cell<bool>>,
    worker: &Worker,
    timeline: &mut Timeline,
    now: Instant,
    nav: MainNav,
) {
    match &nav {
        MainNav::Playlist { id, liked } => open_playlist_for(state, worker, id, *liked),
        MainNav::Home => *state.open_playlist.borrow_mut() = None,
    }
    *state.nav.borrow_mut() = nav;
    // Restart the entrance transition from 0. The scene rebuild mounts
    // the new content; the tween fades + slides it in over the next
    // ~260 ms (timeline-pumped, no manual rebuild cadence).
    state.main_t.set(0.0);
    timeline.animate(&state.main_t, 1.0, NAV_CURVE, MAIN_NAV_DURATION, now);
    rebuild.set(true);
}

/// Set up `open_playlist` for a nav target. A fresh in-memory cache hit
/// populates the row buffer fully (instant). Otherwise a shell is built
/// from the sidebar-known name/cover (so the header shows immediately)
/// and a streaming fetch is dispatched — the first page + later pages
/// fill the buffer progressively, never a blocking load.
fn open_playlist_for(state: &Rc<AppState>, worker: &Worker, id: &str, liked: bool) {
    let cached = state
        .playlist_cache
        .borrow()
        .get(id)
        .filter(|c| c.fetched.elapsed() < PLAYLIST_TTL)
        .map(|c| c.detail.clone());
    let buf: RowBuf = std::rc::Rc::new(RefCell::new(Vec::new()));

    if let Some(detail) = cached {
        build_rows(state, &buf, &detail.tracks);
        *state.open_playlist.borrow_mut() = Some(OpenPlaylist {
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

    // Shell from whatever the sidebar already knows, so the header isn't
    // blank while metadata + the first page stream in.
    let (name, image_url) = if liked {
        ("Liked Songs".to_string(), None)
    } else {
        state
            .home
            .borrow()
            .playlists
            .iter()
            .find(|p| p.id == id)
            .map(|p| (p.name.clone(), p.image_url.clone()))
            .unwrap_or((String::new(), None))
    };
    let context_uri = if liked {
        None
    } else {
        Some(format!("spotify:playlist:{id}"))
    };
    *state.open_playlist.borrow_mut() = Some(OpenPlaylist {
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
    ensure_playlist_loaded(state, worker, id, liked);
}

/// Dispatch a streaming playlist fetch unless a fresh copy is cached or a
/// load is already in flight. Liked Songs routes through the same path
/// under its sentinel id.
fn ensure_playlist_loaded(state: &AppState, worker: &Worker, id: &str, liked: bool) {
    if state.playlist_inflight.borrow().contains(id) {
        return;
    }
    let Some(token) = state.auth.borrow().as_ref().map(|a| a.access_token.clone()) else {
        log::warn!("playlist load skipped — no auth token");
        return;
    };
    state.playlist_inflight.borrow_mut().insert(id.to_string());
    worker.fetch_playlist(token, id.to_string(), liked);
}

/// Bake `tracks` into [`PlaylistRow`]s appended to `buf`. Each cover gets
/// a reactive `Signal` off the shared `home_art` map (so an arriving
/// handle repaints just that thumb) but the **fetch is not dispatched
/// here** — the row's `cover_url` is kept and the download is triggered
/// lazily when the row scrolls into view (`request_cover`), so opening a
/// 989-track playlist doesn't kick off 989 downloads. Called for both
/// cache-hit opens and every streamed page.
fn build_rows(state: &AppState, buf: &RowBuf, tracks: &[PlaylistTrack]) {
    let mut signals = state.home_art.borrow_mut();
    let mut out = buf.borrow_mut();
    out.reserve(tracks.len());
    for t in tracks {
        let art = t.album_image_url.as_ref().map(|u| {
            let key = album_art::cache_key(u);
            signals
                .entry(key)
                .or_insert_with(|| Signal::new(None))
                .clone()
        });
        out.push(PlaylistRow {
            title: t.name.clone(),
            artist: t.artist.clone(),
            album: t.album.clone(),
            duration: playlist::fmt_duration(t.duration_ms),
            uri: t.uri.clone(),
            art,
            cover_url: t.album_image_url.clone(),
        });
    }
}

/// Lazily fetch a track cover (called when a row materializes). Gated so
/// repeated materializes / already-resolved / in-flight covers are
/// no-ops — only the first sight of an unresolved cover dispatches.
fn dispatch_cover(state: &AppState, worker: &Worker, url: String) {
    let key = album_art::cache_key(&url);
    if let Some(sig) = state.home_art.borrow().get(&key)
        && sig.get().is_some()
    {
        return;
    }
    if state.art_inflight.borrow().contains(&key) {
        return;
    }
    state.art_inflight.borrow_mut().insert(key.clone());
    worker.fetch_album_art(url, key);
}

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
        timeline.animate(&state.crossfade_t, 1.0, Curve::EaseInOut, CROSSFADE_DURATION, now);
        // Foreground cover/thumb ride a separate, faster tween so they
        // swap snappily while the backdrop + accent lag behind.
        state.panel_crossfade_t.set(0.0);
        timeline.animate(
            &state.panel_crossfade_t,
            1.0,
            Curve::EaseInOut,
            PANEL_CROSSFADE_DURATION,
            now,
        );
    }
    if let Some(c) = accent {
        timeline.animate(&state.accent, c, Curve::EaseInOut, CROSSFADE_DURATION, now);
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
                "home data ready: playlists={} recent={} top_artists={} top_tracks={}",
                data.playlists.len(),
                data.recent.len(),
                data.top_artists.len(),
                data.top_tracks.len(),
            );
            prefetch_home_art(state, worker, &data);
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
                        worker.fetch_album_art(url.clone(), key.clone());
                        // Spotify's own accent for this cover (authoritative
                        // over the pixel-average extracted on art decode).
                        if !state.spotify_accent.borrow().contains_key(&key) {
                            worker.fetch_accent(key);
                        }
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
                        timeline.animate(
                            &state.progress,
                            1.0,
                            Curve::Linear,
                            Duration::from_millis(remaining),
                            now,
                        );
                    } else {
                        timeline.stop_for(&state.progress);
                    }
                }
                None => {
                    // Nothing playing on any device. Don't wipe the chrome
                    // to an em-dash — Spotify keeps the last track visible,
                    // paused. The cold-start path seeds title/artist/
                    // progress from `prefs.last_player`, and a `None` here
                    // (e.g. the initial `/me/player` seed when the account
                    // is idle) used to clobber that nice restored state.
                    // Just mark stopped and freeze the progress bar; leave
                    // title/artist/progress showing the last-known track.
                    state.is_playing.set(false);
                    timeline.stop_for(&state.progress);
                }
            }
            *state.player.borrow_mut() = player;
        }
        WorkerResponse::AlbumArtReady { key, handle, accent } => {
            state.art_inflight.borrow_mut().remove(&key);
            // Push the resolved handle into the per-URL Home signal (if
            // any tile bound to this key) — repaints just those nodes via
            // the image bind, no rebuild.
            if let Some(sig) = state.home_art.borrow().get(&key) {
                sig.set(Some(handle));
            }
            // Promote into the crossfade if this cover matches either:
            // (a) the live player (steady-state path — a live track
            //     change resolved), or
            // (b) the persisted `last_player` snapshot AND no live
            //     player has landed yet (cold-start path — disk cache
            //     hit beats the first cluster push so we'd otherwise
            //     discard the art handle and re-fetch later, costing
            //     the user a visible "blank → fade-in" delay).
            // No handle cache: the fresh handle is tree-live once
            // promoted, so it survives atlas eviction. A rapid switch
            // that moved on before the upload landed just leaves the
            // orphan handle for the atlas to evict.
            let live_match = state
                .player
                .borrow()
                .as_ref()
                .and_then(|p| p.album_image_url.as_ref().map(|u| album_art::cache_key(u)))
                .map(|k| k == key)
                .unwrap_or(false);
            let cold_start_match = !live_match
                && state.player.borrow().is_none()
                && state
                    .prefs
                    .borrow()
                    .last_player
                    .as_ref()
                    .and_then(|p| p.album_image_url.as_ref().map(|u| album_art::cache_key(u)))
                    .map(|k| k == key)
                    .unwrap_or(false);
            if live_match || cold_start_match {
                // Prefer Spotify's own extracted colour if it already
                // arrived for this cover; otherwise use the pixel-average
                // as a provisional accent (a later `AccentReady` overrides
                // it). This makes the result order-independent between the
                // two parallel requests.
                let accent = state
                    .spotify_accent
                    .borrow()
                    .get(&key)
                    .copied()
                    .unwrap_or(accent);
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
        WorkerResponse::AccentReady { key, accent } => {
            state.spotify_accent.borrow_mut().insert(key.clone(), accent);
            // Apply only if this cover is the one on screen now (or the
            // live player's) — a late arrival for a skipped track is kept
            // in the map but not tweened in. Overrides any provisional
            // pixel-average accent with Spotify's exact colour.
            let is_current = state.shown_art_key.borrow().as_deref() == Some(key.as_str())
                || state
                    .player
                    .borrow()
                    .as_ref()
                    .and_then(|p| p.album_image_url.as_ref().map(|u| album_art::cache_key(u)))
                    .map(|k| k == key)
                    .unwrap_or(false);
            if is_current {
                timeline.animate(&state.accent, accent, Curve::EaseInOut, CROSSFADE_DURATION, now);
            }
        }
        WorkerResponse::PlaylistOpened { detail, complete } => {
            let id = detail.id.clone();
            // Apply to the open pane if it's still showing this playlist:
            // overwrite metadata + seed the first page, then rebuild ONCE
            // to mount the full-length virtualised list (item_count =
            // total). Subsequent pages append without a rebuild.
            let applies =
                matches!(&*state.nav.borrow(), MainNav::Playlist { id: nid, .. } if *nid == id);
            if applies {
                let buf = {
                    let mut op = state.open_playlist.borrow_mut();
                    op.as_mut().map(|o| {
                        o.name = detail.name.clone();
                        o.owner = detail.owner.clone();
                        o.image_url = detail.image_url.clone();
                        o.context_uri = detail.context_uri.clone();
                        o.total = detail.total;
                        o.loading = false;
                        o.complete = complete;
                        o.rows.clone()
                    })
                };
                if let Some(buf) = buf {
                    buf.borrow_mut().clear();
                    build_rows(state, &buf, &detail.tracks);
                    rebuild.set(true);
                }
            }
            // A `complete` response (disk-cache hit or single-page) carries
            // the whole listing — cache it in memory for an instant
            // re-open and clear the inflight gate.
            if complete {
                state.playlist_inflight.borrow_mut().remove(&id);
                state.playlist_cache.borrow_mut().insert(
                    id.clone(),
                    CachedPlaylist {
                        detail,
                        fetched: Instant::now(),
                    },
                );
            }
        }
        WorkerResponse::PlaylistTracks { id, tracks, done } => {
            // Append a streamed page into the live buffer — no rebuild;
            // the lazy_list reads it on scroll. (Covers fill in reactively
            // via the per-row image bind baked in `build_rows`.)
            let applies =
                matches!(&*state.nav.borrow(), MainNav::Playlist { id: nid, .. } if *nid == id);
            if applies {
                let buf = state.open_playlist.borrow().as_ref().map(|o| o.rows.clone());
                if let Some(buf) = buf {
                    build_rows(state, &buf, &tracks);
                }
                if done && let Some(o) = state.open_playlist.borrow_mut().as_mut() {
                    o.complete = true;
                }
            }
            if done {
                state.playlist_inflight.borrow_mut().remove(&id);
            }
        }
        WorkerResponse::PlaylistFailed { id, error } => {
            state.playlist_inflight.borrow_mut().remove(&id);
            log::warn!("playlist {id} load failed: {error}");
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

