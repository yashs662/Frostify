use librespot_core::{Session, SessionConfig};

/// Build an *un-connected* librespot Session. The actual `Session::connect`
/// is performed inside `Spirc::new` — calling it ourselves before Spirc
/// invalidates the AP socket the moment Spirc re-connects with its own
/// credentials (manifests as `Service unavailable { Session is not connected }`
/// at Spirc init). Mirrors `librespot/examples/play_connect.rs`.
///
/// We expose this as its own factory so `add_listen_for(...)` (which buffers
/// against the session's pre-connect builder) can be wired before Spirc
/// starts driving the dealer.
pub fn new_session() -> Session {
    Session::new(SessionConfig::default(), None)
}
