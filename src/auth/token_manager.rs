use crate::auth::oauth::SpotifyAuthResponse;
use crate::constants::{CREDENTIAL_SERVICE_NAME, CREDENTIAL_USER_NAME};
use crate::errors::AuthError;
use keyring_core::Entry;
use log::debug;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Spotify refresh tokens stop working after ~180 days; past that the
/// refresh grant fails and the user must re-authorise. We proactively wipe
/// + force re-login a touch early so a stale token never 400s mid-launch.
pub const REFRESH_TOKEN_MAX_AGE_SECS: u64 = 180 * 24 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub token_type: String,
    pub scope: String,
    /// Unix seconds when the current `refresh_token` was issued. Set on the
    /// initial OAuth and reset whenever Spotify rotates the refresh token on
    /// a refresh; used to enforce the 180-day cap. `0` = legacy file written
    /// before this field existed — treated as "unknown age", never expired.
    #[serde(default)]
    pub refresh_issued_at: u64,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

impl From<SpotifyAuthResponse> for StoredTokens {
    fn from(a: SpotifyAuthResponse) -> Self {
        StoredTokens {
            access_token: a.access_token,
            refresh_token: a.refresh_token,
            expires_at: now_secs() + a.expires_in,
            token_type: a.token_type,
            scope: a.scope,
            // A bare conversion is a *fresh* authorisation → the refresh
            // token's clock starts now. Refresh-grant results go through
            // [`StoredTokens::from_refresh`] instead to carry the age over.
            refresh_issued_at: now_secs(),
        }
    }
}

impl StoredTokens {
    /// Build stored tokens from a refresh-grant response, preserving the
    /// refresh token's age unless Spotify actually rotated it. Spotify
    /// usually returns a new refresh token on each refresh (the clock
    /// resets); when it returns the same one, we keep the original
    /// `refresh_issued_at` so the 180-day cap counts from first issue.
    pub fn from_refresh(auth: SpotifyAuthResponse, prev: Option<&StoredTokens>) -> Self {
        let mut t = StoredTokens::from(auth);
        if let Some(p) = prev
            && t.refresh_token == p.refresh_token
            && p.refresh_issued_at != 0
        {
            t.refresh_issued_at = p.refresh_issued_at;
        }
        t
    }

    /// True when the refresh token is past the 180-day cap and should be
    /// discarded in favour of a fresh login. `0` (legacy/unknown) is never
    /// considered expired.
    pub fn refresh_expired(&self) -> bool {
        self.refresh_issued_at != 0
            && now_secs().saturating_sub(self.refresh_issued_at) > REFRESH_TOKEN_MAX_AGE_SECS
    }

    pub fn to_auth_response(&self) -> SpotifyAuthResponse {
        let expires_in = self.expires_at.saturating_sub(now_secs());
        SpotifyAuthResponse {
            access_token: self.access_token.clone(),
            refresh_token: self.refresh_token.clone(),
            expires_in,
            token_type: self.token_type.clone(),
            scope: self.scope.clone(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at < now_secs() + 60
    }

    /// Returns true if every comma-separated scope in `required` is
    /// present in this token's granted scope string. Used at startup
    /// to invalidate tokens that were minted before a constants update
    /// added new scopes — avoids a 401 loop on the first API call.
    pub fn has_scopes(&self, required: &str) -> bool {
        // OAuth tokens return scope space-separated; our constants list
        // is comma-separated. Normalise both sides on either delimiter.
        let granted: std::collections::HashSet<&str> = self
            .scope
            .split([' ', ','])
            .filter(|s| !s.is_empty())
            .collect();
        required
            .split([' ', ','])
            .filter(|s| !s.is_empty())
            .all(|s| granted.contains(s))
    }
}

pub fn save_tokens(t: &StoredTokens) -> Result<(), AuthError> {
    let entry = Entry::new(CREDENTIAL_SERVICE_NAME, CREDENTIAL_USER_NAME)?;
    entry.set_secret(serde_json::to_string(t)?.as_bytes())?;
    debug!("tokens saved");
    Ok(())
}

pub fn load_tokens() -> Result<StoredTokens, AuthError> {
    let entry = Entry::new(CREDENTIAL_SERVICE_NAME, CREDENTIAL_USER_NAME)?;
    let bytes = entry.get_secret()?;
    let s = std::str::from_utf8(&bytes)?;
    Ok(serde_json::from_str(s)?)
}

pub fn delete_tokens() -> Result<(), AuthError> {
    let entry = Entry::new(CREDENTIAL_SERVICE_NAME, CREDENTIAL_USER_NAME)?;
    entry.delete_credential()?;
    Ok(())
}
