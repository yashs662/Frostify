use crate::{
    auth::oauth::SpotifyAuthResponse,
    constants::{CREDENTIAL_SERVICE_NAME, CREDENTIAL_USER_NAME},
    errors::AuthError,
};
use keyring::Entry;
use log::debug;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64, // Unix timestamp when the token expires
    pub token_type: String,
    pub scope: String,
}

impl From<SpotifyAuthResponse> for StoredTokens {
    fn from(auth: SpotifyAuthResponse) -> Self {
        // Calculate expiration time by adding expires_in to current time
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        StoredTokens {
            access_token: auth.access_token,
            refresh_token: auth.refresh_token,
            expires_at: now + auth.expires_in,
            token_type: auth.token_type,
            scope: auth.scope,
        }
    }
}

impl StoredTokens {
    pub fn to_auth_response(&self) -> SpotifyAuthResponse {
        // Calculate remaining time until expiration
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let expires_in = self.expires_at.saturating_sub(now);

        SpotifyAuthResponse {
            access_token: self.access_token.clone(),
            refresh_token: self.refresh_token.clone(),
            expires_in,
            token_type: self.token_type.clone(),
            scope: self.scope.clone(),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        // Consider token expired if it expires in less than 60 seconds
        self.expires_at < now + 60
    }
}

pub fn save_tokens(tokens: &StoredTokens) -> Result<(), AuthError> {
    let entry = Entry::new(CREDENTIAL_SERVICE_NAME, CREDENTIAL_USER_NAME)?;
    let serialized = serde_json::to_string(tokens)?;
    let serialized = serialized.as_bytes();
    entry.set_secret(serialized)?;
    debug!("Spotify tokens saved successfully");
    Ok(())
}

pub fn load_tokens() -> Result<StoredTokens, AuthError> {
    let entry = Entry::new(CREDENTIAL_SERVICE_NAME, CREDENTIAL_USER_NAME)?;
    let serialized = entry.get_secret()?;
    let serialized = std::str::from_utf8(&serialized)?;
    let tokens: StoredTokens = serde_json::from_str(serialized)?;
    // debug the time in minutes/seconds until the token expires
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    if tokens.expires_at < now {
        debug!("Stored Spotify tokens have expired");
    } else {
        let time_until_expiration = tokens.expires_at - now;
        debug!(
            "Loaded stored Spotify tokens, time until expiration: {} minutes {} seconds",
            time_until_expiration / 60,
            time_until_expiration % 60
        );
    }
    debug!("Loaded stored Spotify tokens");
    Ok(tokens)
}

pub fn delete_tokens() -> Result<(), AuthError> {
    let entry = Entry::new(CREDENTIAL_SERVICE_NAME, CREDENTIAL_USER_NAME)?;
    entry.delete_credential()?;
    debug!("Spotify tokens deleted successfully");
    Ok(())
}
