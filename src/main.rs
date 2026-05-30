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

use crate::api::{CurrentlyPlaying, HomeData, RepeatMode, TrackDetails, track_id_from_uri};
use crate::auth::oauth::SpotifyAuthResponse;
use crate::prefs::{StoredPlayer, UserPreferences};
use crate::ui::View;
use crate::ui::home::PlayerAction;
use crate::ui::tokens;
use crate::worker::{PlaybackCmd, Worker, WorkerResponse};

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

/// Tween key namespace. Each timeline tween is identified by a `u32`
/// and replacing one with the same key restarts smoothly mid-flight.
const TWEEN_KEY_CROSSFADE: u32 = 0x0001_0001;
const TWEEN_KEY_ACCENT: u32 = 0x0001_0002;
const TWEEN_KEY_PANEL: u32 = 0x0001_0003;
const TWEEN_KEY_PROGRESS: u32 = 0x0001_0004;
/// Anchors a dummy timeline tween that keeps the loop awake long
/// enough for the debounced prefs save to fire after the user stops
/// changing things.
const TWEEN_KEY_PREFS_DEBOUNCE: u32 = 0x0001_0005;

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
    // `frostify_gfx=debug` would spam per-frame `[loop] WaitUntil(...)` +
    // active-tick lines while the progress tween runs (60 fps during
    // playback). Drop the lib to `info`; keep `frostify` at debug.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(
        "info,wgpu_hal=warn,wgpu_core=warn,frostify=debug,frostify_gfx=info",
    ))
    .init();

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

    let state = Rc::new(AppState::from_prefs(prefs));
    if std::env::var_os("FROSTIFY_FORCE_HOME").is_some() {
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

    let app = {
        let state = state.clone();
        let on_login = on_login.clone();
        let icons = icons.clone();
        let on_action = on_action.clone();
        let on_canvas_change = on_canvas_change.clone();
        let sign_out = sign_out.clone();
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
        timeline.stop(TWEEN_KEY_PREFS_DEBOUNCE);
    } else {
        // Keep the loop awake until the deadline. Restart the anchor
        // tween (idempotent — timeline.start with the same key replaces
        // any in-flight one) so this fires whether or not other input
        // is keeping the loop ticking.
        let remaining = PREFS_DEBOUNCE - elapsed + Duration::from_millis(50);
        state.prefs_save_anchor.set(0.0);
        timeline.start(
            TWEEN_KEY_PREFS_DEBOUNCE,
            state.prefs_save_anchor.clone(),
            1.0,
            Curve::Linear,
            remaining,
            now,
        );
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
/// The lib's timeline pump drives the tween at 60 Hz; the scene's
/// `Computed` colour binds map `crossfade_t` to per-layer alphas and
/// `process_binds` (per about_to_wait tick) pushes the new values to
/// the tree. No manual rebuild cadence.
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
                    // Nothing playing on any device. Don't wipe the chrome
                    // to an em-dash — Spotify keeps the last track visible,
                    // paused. The cold-start path seeds title/artist/
                    // progress from `prefs.last_player`, and a `None` here
                    // (e.g. the initial `/me/player` seed when the account
                    // is idle) used to clobber that nice restored state.
                    // Just mark stopped and freeze the progress bar; leave
                    // title/artist/progress showing the last-known track.
                    state.is_playing.set(false);
                    timeline.stop(TWEEN_KEY_PROGRESS);
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
                timeline.start(
                    TWEEN_KEY_ACCENT,
                    state.accent.clone(),
                    accent,
                    Curve::EaseInOut,
                    CROSSFADE_DURATION,
                    now,
                );
            }
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

