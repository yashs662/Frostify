//! View-routing slice — the top-level view + centre-pane nav + the
//! entrance transition.
//!
//! `view` selects Splash/Login/Home; `nav` selects what the Home centre
//! pane shows (feed vs a playlist page). [`RouterModel::go`] flips `nav`
//! and restarts the slide/fade-in tween that the scene rebuild mounts the
//! new content under — the one place a deliberate rebuild is correct
//! (the content is structurally different; the reactive path can't
//! restructure the tree).

use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

use frostify_gfx::{Curve, Signal, Timeline};

use crate::views::{MainNav, View};

/// Centre-pane content transition duration on nav change.
const MAIN_NAV_DURATION: Duration = Duration::from_millis(260);

/// Ease-out for the centre-pane entrance — fast start, gentle settle.
const NAV_CURVE: Curve = Curve::CubicBezier([0.16, 1.0, 0.3, 1.0]);

pub struct RouterModel {
    pub view: Cell<View>,
    /// What the Home centre pane is showing (feed vs a playlist page).
    pub nav: RefCell<MainNav>,
    /// 0 → 1 slide/fade progress for the centre-pane content, retween'd on
    /// every nav change. Parks at 1.0 (settled).
    pub main_t: Signal<f32>,
    /// Detail-page header collapse, 0 (hero fully expanded) → 1 (collapsed
    /// into the sticky bar). Driven each frame from the open detail page's
    /// scroll offset (see `app::frame::tick`); the view slides + fades the
    /// sticky bar from it. Reset to 0 on every nav.
    pub detail_collapse: Signal<f32>,
}

impl RouterModel {
    pub fn new() -> Self {
        Self {
            view: Cell::default(),
            nav: RefCell::default(),
            main_t: Signal::new(1.0),
            detail_collapse: Signal::new(0.0),
        }
    }

    /// Whether the centre pane is showing the detail page (playlist or album)
    /// for `id`. Used by the reducer to decide if a `PlaylistOpened`/`Tracks`
    /// response still applies to the open pane (albums reuse that response).
    pub fn nav_is_open(&self, id: &str) -> bool {
        match &*self.nav.borrow() {
            MainNav::Playlist { id: nid, .. } | MainNav::Album { id: nid } => nid == id,
            MainNav::Home
            | MainNav::Artist { .. }
            | MainNav::ShowAll { .. }
            | MainNav::Queue => false,
        }
    }

    /// Whether the centre pane is showing the artist page for `id`.
    pub fn nav_is_artist(&self, id: &str) -> bool {
        matches!(&*self.nav.borrow(), MainNav::Artist { id: nid } if nid == id)
    }

    /// Flip nav to `nav` and restart the entrance transition from 0 — the
    /// scene rebuild mounts the new content; the tween fades + slides it in
    /// over ~260 ms (timeline-pumped, no manual rebuild cadence).
    pub fn go(&self, nav: MainNav, tl: &mut Timeline, now: Instant) {
        *self.nav.borrow_mut() = nav;
        // New page starts scrolled to top → header fully expanded.
        self.detail_collapse.set(0.0);
        self.main_t.set(0.0);
        tl.animate(&self.main_t, 1.0, NAV_CURVE, MAIN_NAV_DURATION, now);
    }
}

impl Default for RouterModel {
    fn default() -> Self {
        Self::new()
    }
}
