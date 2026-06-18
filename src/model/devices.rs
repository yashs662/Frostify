//! Connect-devices slice — the devices popup + "playing on Frostify"
//! chrome state.
//!
//! The device list is fetched fresh on every popup open (it's live state
//! — devices appear and vanish with the apps hosting them, so caching
//! would only show ghosts). `active_id`/`playing_on_self` mirror the
//! cluster's active device and update reactively between opens.

use std::cell::RefCell;

use frostify_gfx::{Overlay, Signal};

use crate::api::Device;

pub struct DevicesModel {
    /// The popup's scrim/fade/dismiss owner (same primitive as settings).
    pub overlay: Overlay,
    /// Latest fetched device list (popup rows). Empty until the first
    /// open's fetch lands.
    pub list: RefCell<Vec<Device>>,
    /// The cluster's active device id ("" = none) — highlights the
    /// active row even when the REST list's `is_active` lags a push.
    pub active_id: RefCell<String>,
    /// Frostify's own librespot device id, for the "This device" row tag.
    pub self_id: RefCell<String>,
    /// Frostify is the active device. Drives transport routing (local Spirc
    /// vs Web API) and the "This device" affordances.
    pub playing_on_self: Signal<bool>,
    /// Some *other* device is the active player — lights the player-bar
    /// Devices icon with the accent (Spotify's "connected to a device"
    /// cue). False when Frostify is playing or nothing is active.
    pub remote_active: Signal<bool>,
}

impl DevicesModel {
    pub fn new() -> Self {
        Self {
            overlay: Overlay::new(),
            list: RefCell::default(),
            active_id: RefCell::default(),
            self_id: RefCell::default(),
            playing_on_self: Signal::new(false),
            remote_active: Signal::new(false),
        }
    }
}

impl Default for DevicesModel {
    fn default() -> Self {
        Self::new()
    }
}
