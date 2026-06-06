//! View layer.
//!
//! Each view owns its components, its callbacks, and its scene build; the
//! app's router state ([`View`]) selects which one is active. This is the
//! layer that replaces "`main` composes everything" — `main` only
//! constructs the views and dispatches to them.

pub mod home;
pub mod login;

/// Which top-level view is mounted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Splash,
    Login,
    Home,
}

/// What the centre (main) pane of the Home view is showing. The sidebar,
/// now-playing pane, and player bar stay mounted across these; only the
/// main pane's content swaps (with a slide/fade transition). Switching is
/// a deliberate one-shot scene rebuild — distinct from the periodic
/// rebuilds the reactive path was built to avoid.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MainNav {
    /// The default Home feed (greeting, recents, top artists, …).
    #[default]
    Home,
    /// A playlist detail page. `id` is the Spotify playlist id, or
    /// [`crate::api::LIKED_SONGS_ID`] when `liked` is set.
    Playlist { id: String, liked: bool },
}
