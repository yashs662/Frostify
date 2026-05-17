use crate::auth::oauth::SpotifyAuthResponse;
use crate::constants::{CREDENTIAL_SERVICE_NAME, CREDENTIAL_USER_NAME};
use crate::errors::AuthError;
use keyring::Entry;
use log::debug;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
    pub token_type: String,
    pub scope: String,
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
        }
    }
}

impl StoredTokens {
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

#[allow(dead_code)]
pub fn delete_tokens() -> Result<(), AuthError> {
    let entry = Entry::new(CREDENTIAL_SERVICE_NAME, CREDENTIAL_USER_NAME)?;
    entry.delete_credential()?;
    Ok(())
}
