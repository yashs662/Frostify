//! Authentication slice — the live Spotify OAuth session.
//!
//! Thin holder around the current [`SpotifyAuthResponse`]. Most call
//! sites only want the access token for a worker command, so [`token`]
//! collapses the former repeated
//! `auth.borrow().as_ref().map(|a| a.access_token.clone())` dance into
//! one accessor.
//!
//! [`token`]: AuthModel::token

use std::cell::RefCell;

use crate::auth::oauth::SpotifyAuthResponse;

#[derive(Default)]
pub struct AuthModel {
    current: RefCell<Option<SpotifyAuthResponse>>,
}

impl AuthModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clone the live access token, or `None` when signed out. Read at
    /// fire time so it survives a token refresh.
    pub fn token(&self) -> Option<String> {
        self.current.borrow().as_ref().map(|a| a.access_token.clone())
    }

    pub fn set(&self, auth: SpotifyAuthResponse) {
        *self.current.borrow_mut() = Some(auth);
    }

    pub fn clear(&self) {
        *self.current.borrow_mut() = None;
    }

    /// Sign out: delete the persisted token from the OS store and drop the
    /// in-memory session. (The caller handles the view switch / modal
    /// reset — those are shell concerns.)
    pub fn sign_out(&self) {
        if let Err(e) = crate::auth::token_manager::delete_tokens() {
            log::warn!("sign-out: failed to clear stored token: {e}");
        }
        self.clear();
    }
}
