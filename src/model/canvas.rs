//! Spotify Canvas (looping video) slice.
//!
//! Owns the resolved/cached clip path, the off-thread decode session, the
//! engine `FrameSink` + live target node shared with that thread, and the
//! hover-driven dim overlay. The decode thread presents frames at a
//! running deadline (so decode time doesn't drop the effective fps) onto
//! the now-playing `.external()` node, following scene rebuilds via the
//! shared [`node`](Self::node) slot.
//!
//! Cross-slice inputs (the live player, the worker `fetch_canvas`
//! dispatch) stay in the caller; this slice just exposes the decode
//! lifecycle + the per-frame ticks.

use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use frostify_gfx::node::NodeId;
use frostify_gfx::{Curve, FrameSink, ImageHandle, Signal, Timeline};

use crate::api::{CurrentlyPlaying, track_id_from_uri};
use crate::worker::Worker;

/// Resting alpha of the dark overlay over the Canvas video (dimmed until
/// hovered). 0 = no dim.
const CANVAS_DIM_ALPHA: f32 = 0.5;

/// Brightness tween duration when the Canvas video is hovered / un-hovered.
const CANVAS_DIM_DURATION: Duration = Duration::from_millis(280);

/// A running Canvas-video decode: the track it's decoding and the flag
/// the decode thread polls so a track change (or canvas-off) can stop it.
struct CanvasSession {
    track_id: String,
    stop: Arc<AtomicBool>,
}

pub struct CanvasModel {
    /// Whether to show the looping Canvas video in now-playing (persisted
    /// via prefs; toggled in settings; consumed here).
    pub show: Signal<bool>,
    /// UI-thread mirror of [`has_video`](Self::has_video), read at
    /// scene-build time to choose the now-playing layout. Cell (not
    /// Signal): the swap is a deliberate rebuild, not a reactive bind.
    pub active: Cell<bool>,
    /// Hover state of the now-playing Canvas video (set by `.on_hover`).
    pub hover: Signal<bool>,
    /// Alpha of the dark overlay over the video — dimmed at rest, tweened
    /// to 0 (full brightness) on hover. Bound to the overlay's colour.
    pub dim: Signal<f32>,
    /// Staged black dim gradient (opaque top → transparent over the
    /// bottom, matching the video's edge fade), tinted by [`dim`](Self::dim).
    pub dim_grad: Cell<Option<ImageHandle>>,

    /// Resolved + cached clip for the current track: `(track_id, mp4_path)`.
    /// Set by `CanvasReady`, cleared on track change / `CanvasNone`.
    path: RefCell<Option<(String, std::path::PathBuf)>>,
    /// Engine handle for pushing decoded frames onto the now-playing
    /// `.external()` node. `None` until installed after the `App` is built.
    frame_sink: RefCell<Option<Arc<FrameSink>>>,
    /// Live `NodeId` of the now-playing canvas node, refreshed each frame
    /// (the id is not stable across rebuilds). Shared with the decode
    /// thread; `None` when the node isn't in the current tree.
    node: Arc<Mutex<Option<NodeId>>>,
    /// Set true by the decode thread on its first frame, cleared on stop.
    /// [`tick_active`](Self::tick_active) mirrors it into [`active`](Self::active).
    has_video: Arc<AtomicBool>,
    /// Last observed `hover` value, so the brightness tween only (re)starts
    /// on a hover *transition*.
    hover_last: Cell<bool>,
    /// Active decode session. Replaced on track change; `None` when idle.
    decode: RefCell<Option<CanvasSession>>,
}

impl CanvasModel {
    pub fn new(show_canvas: bool) -> Self {
        Self {
            show: Signal::new(show_canvas),
            active: Cell::new(false),
            hover: Signal::new(false),
            dim: Signal::new(CANVAS_DIM_ALPHA),
            dim_grad: Cell::new(None),
            path: RefCell::default(),
            frame_sink: RefCell::default(),
            node: Arc::new(Mutex::new(None)),
            has_video: Arc::new(AtomicBool::new(false)),
            hover_last: Cell::new(false),
            decode: RefCell::default(),
        }
    }

    /// Install the engine frame sink (after the `App` is built).
    pub fn set_frame_sink(&self, sink: Arc<FrameSink>) {
        *self.frame_sink.borrow_mut() = Some(sink);
    }

    /// RGBA pixels `(w, h, rgba)` for the dim-overlay gradient: solid
    /// black over the top, fading to transparent across the bottom 35% so
    /// it matches the video's own `fade_bottom(0.35)` edge fade. The
    /// gradient *shape* is canvas knowledge and lives here; the host only
    /// does the GPU upload (the one part that needs the engine) and hands
    /// the resulting handle back via [`set_dim_grad`](Self::set_dim_grad).
    /// Tinted at draw time by [`dim`](Self::dim).
    pub fn dim_grad_rgba() -> (u32, u32, Vec<u8>) {
        let (gw, gh) = (4u32, 256u32);
        let fade_start = 0.65f32;
        let mut px = Vec::with_capacity((gw * gh * 4) as usize);
        for y in 0..gh {
            let f = y as f32 / (gh - 1) as f32;
            let a = if f < fade_start {
                1.0
            } else {
                ((1.0 - f) / (1.0 - fade_start)).clamp(0.0, 1.0)
            };
            let a = (a * 255.0) as u8;
            for _ in 0..gw {
                px.extend_from_slice(&[0, 0, 0, a]);
            }
        }
        (gw, gh, px)
    }

    /// Install the staged dim-gradient handle (host uploads
    /// [`dim_grad_rgba`](Self::dim_grad_rgba) and hands back the handle).
    pub fn set_dim_grad(&self, handle: ImageHandle) {
        self.dim_grad.set(Some(handle));
    }

    // --- cached clip path ---------------------------------------------

    /// Clone the cached `(track_id, path)`, if any.
    pub fn cached_path(&self) -> Option<(String, std::path::PathBuf)> {
        self.path.borrow().clone()
    }

    /// Whether the cached clip is for `track_id`.
    pub fn path_matches(&self, track_id: &str) -> bool {
        self.path.borrow().as_ref().map(|(t, _)| t == track_id).unwrap_or(false)
    }

    pub fn set_path(&self, track_id: String, path: std::path::PathBuf) {
        self.path.borrow_mut().replace((track_id, path));
    }

    pub fn clear_path(&self) {
        self.path.borrow_mut().take();
    }

    // --- per-frame ticks ----------------------------------------------

    /// Keep the live canvas node id in sync so the decode thread targets
    /// the correct node even after a scene rebuild.
    pub fn sync_node(&self, resolved: Option<NodeId>) {
        let mut slot = self.node.lock().unwrap();
        if slot.is_none() && resolved.is_some() {
            log::debug!("canvas node resolved: {resolved:?}");
        }
        *slot = resolved;
    }

    /// Mirror the decode thread's "video is flowing" flag into the
    /// build-time layout flag. Returns `true` on a change (caller rebuilds
    /// so the now-playing pane swaps album-art ↔ full-bleed video).
    pub fn tick_active(&self) -> bool {
        let want = self.has_video.load(Ordering::Relaxed);
        if want != self.active.get() {
            self.active.set(want);
            true
        } else {
            false
        }
    }

    /// Tween the dim overlay on hover transitions: bright (0) while
    /// hovered, dimmed at rest.
    pub fn tick_dim(&self, tl: &mut Timeline, now: Instant) {
        let hov = self.hover.get();
        if hov != self.hover_last.get() {
            self.hover_last.set(hov);
            let target = if hov { 0.0 } else { CANVAS_DIM_ALPHA };
            tl.animate(&self.dim, target, Curve::EaseInOut, CANVAS_DIM_DURATION, now);
        }
    }

    // --- decode lifecycle ---------------------------------------------

    /// Spawn (or replace) the decode thread for `track_id`, reading the
    /// cached MP4 at `path`. Any prior session is stopped first. The
    /// thread decodes in a loop, presenting each frame at a running
    /// deadline, and pushes to the now-playing external node via the
    /// `FrameSink`, targeting [`node`](Self::node) read fresh each frame so
    /// it follows rebuilds. No-op if already decoding this track or the
    /// frame sink isn't installed yet.
    pub fn start_decode(&self, track_id: String, path: std::path::PathBuf) {
        if self
            .decode
            .borrow()
            .as_ref()
            .map(|s| s.track_id == track_id)
            .unwrap_or(false)
        {
            return;
        }
        log::debug!("start_canvas_decode {track_id}");
        self.stop_decode();
        let Some(sink) = self.frame_sink.borrow().clone() else {
            return;
        };
        let node = self.node.clone();
        let has_video = self.has_video.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let spawned = std::thread::Builder::new()
            .name("canvas-decode".into())
            .spawn(move || {
                let Ok(bytes) = std::fs::read(&path) else {
                    log::warn!("canvas decode: failed to read {}", path.display());
                    return;
                };
                let Some(mut video) = crate::video::CanvasPlayer::open(&bytes) else {
                    log::warn!("canvas decode: {} is not a decodable AVC clip", path.display());
                    return;
                };
                log::debug!("canvas decode: {} samples", video.frame_count());

                // Two-phase playback, both honouring "no per-frame CPU→GPU
                // transfer once looping":
                //   1. Build phase — decode the first pass, uploading each
                //      frame *once* to its own GPU texture (`push_frame`),
                //      showing it live. The whole loop ends up resident in
                //      VRAM; the frame durations are recorded here.
                //   2. Loop phase — replay by `select(index)`, a view re-bind
                //      with no pixel transfer (≈ 0 CPU/GPU).
                // A scene rebuild reassigns the canvas node id; we `migrate`
                // the resident set to the new id rather than re-uploading.
                let mut building = true;
                let mut durations: Vec<std::time::Duration> = Vec::new();
                let mut idx = 0usize;
                let mut bound: Option<NodeId> = None;
                let mut first = true;
                // Present at a running deadline rather than sleeping a full
                // interval *after* the work — otherwise each frame costs
                // `work + interval`, dropping below the clip's native fps.
                let mut next_at = std::time::Instant::now();
                while !stop_thread.load(Ordering::Relaxed) {
                    // Reconcile the resident set with the live node id: a
                    // rebuild swaps the id, so move the textures across once.
                    let cur = *node.lock().unwrap();
                    match (bound, cur) {
                        (Some(b), Some(c)) if b != c => {
                            sink.migrate(b, c);
                            bound = Some(c);
                        }
                        (None, Some(c)) => bound = Some(c),
                        _ => {}
                    }

                    let dur = if building {
                        match video.next_pass_frame() {
                            Some(frame) => {
                                let dur = frame.duration;
                                // Only commit a frame once we have a node to
                                // attach it to, keeping durations aligned with
                                // the resident set.
                                if let Some(b) = bound {
                                    if first {
                                        log::debug!(
                                            "canvas decode: {}x{} @ {:?} (resident)",
                                            frame.width,
                                            frame.height,
                                            frame.duration
                                        );
                                        first = false;
                                        // First frame → swap to the video layout.
                                        has_video.store(true, Ordering::Relaxed);
                                    }
                                    durations.push(dur);
                                    sink.push_frame(b, frame.width, frame.height, frame.rgba);
                                }
                                dur
                            }
                            None => {
                                building = false;
                                if durations.is_empty() {
                                    log::warn!(
                                        "canvas decode: stopped — clip yielded no decodable frame"
                                    );
                                    break;
                                }
                                continue;
                            }
                        }
                    } else {
                        // Loop phase: just re-bind the next resident frame.
                        if let Some(b) = bound {
                            sink.select(b, idx);
                        }
                        let dur = durations[idx];
                        idx = (idx + 1) % durations.len();
                        dur
                    };

                    // Sleep only the remainder of this frame's interval.
                    next_at += dur;
                    let now = std::time::Instant::now();
                    if next_at > now {
                        std::thread::sleep(next_at - now);
                    } else {
                        next_at = now;
                    }
                }
            });
        match spawned {
            Ok(_) => {
                self.decode.borrow_mut().replace(CanvasSession { track_id, stop });
            }
            Err(e) => log::warn!("canvas decode: failed to spawn thread: {e}"),
        }
    }

    /// React to the `show_canvas` toggle flipping. Turned on mid-track:
    /// decode the cached clip if we have it, else fetch it for the current
    /// track. Turned off: stop decoding + drop the video texture. The
    /// caller persists the (debounced) pref separately.
    pub fn on_toggle(&self, snapshot: Option<&CurrentlyPlaying>, worker: &Worker) {
        if self.show.get() {
            match self.cached_path() {
                Some((track_id, path)) => self.start_decode(track_id, path),
                None => {
                    if let Some(p) = snapshot
                        && let Some(id) = track_id_from_uri(&p.track_id)
                    {
                        worker.fetch_canvas(p.track_id.clone(), id.to_string());
                    }
                }
            }
        } else {
            self.stop_decode();
        }
    }

    /// Stop the active decode (if any) and clear the now-playing external
    /// texture so the UI falls back to album art.
    pub fn stop_decode(&self) {
        if let Some(old) = self.decode.borrow_mut().take() {
            old.stop.store(true, Ordering::Relaxed);
        }
        self.has_video.store(false, Ordering::Relaxed);
        if let (Some(sink), Some(node)) =
            (self.frame_sink.borrow().clone(), *self.node.lock().unwrap())
        {
            sink.clear(node);
        }
    }
}
