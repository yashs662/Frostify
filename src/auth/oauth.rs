use crate::{
    constants::{
        FROSTIFY_LOGIN_ERROR_HTML, FROSTIFY_LOGIN_SUCCESS_HTML, SPOTIFY_ACCESS_SCOPES,
        SPOTIFY_CLIENT_ID, SPOTIFY_REDIRECT_URI,
    },
    errors::AuthError,
    ui::asset::get_asset,
};
use base64::{Engine, engine::general_purpose, prelude::BASE64_URL_SAFE_NO_PAD};
use log::debug;
use rand::{Rng, distr::Alphanumeric};
use reqwest::Client;
use sha2::{Digest, Sha256};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use url::Url;

// Ensure this type can be cloned for passing between threads
#[derive(Debug, Clone)]
pub struct SpotifyAuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
}

pub fn get_spotify_auth_url() -> (String, String) {
    let code_verifier = generate_pkce_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);

    let auth_url = Url::parse_with_params(
        "https://accounts.spotify.com/authorize",
        &[
            ("client_id", SPOTIFY_CLIENT_ID),
            ("response_type", "code"),
            ("redirect_uri", SPOTIFY_REDIRECT_URI),
            ("scope", SPOTIFY_ACCESS_SCOPES),
            ("code_challenge_method", "S256"),
            ("code_challenge", &code_challenge),
        ],
    )
    .unwrap()
    .to_string();

    (auth_url, code_verifier)
}

fn generate_pkce_code_verifier() -> String {
    let random_alphanumeric = rand::rng().sample_iter(Alphanumeric).take(64);
    random_alphanumeric.map(char::from).collect()
}

fn generate_code_challenge(code_verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code_verifier);
    let result = hasher.finalize();
    BASE64_URL_SAFE_NO_PAD.encode(result)
}

// Updated to use AuthError
pub async fn listen_for_callback(code_verifier: String) -> Result<SpotifyAuthResponse, AuthError> {
    let listener = TcpListener::bind("127.0.0.1:8888").await.map_err(|e| {
        log::error!("Failed to bind TCP listener: {e}");
        AuthError::Server(e.to_string())
    })?;

    debug!("Listening on http://127.0.0.1:8888...");

    // Add a timeout for accepting connections
    let accept_result = tokio::time::timeout(
        std::time::Duration::from_secs(120), // 2 minute timeout
        listener.accept(),
    )
    .await
    .map_err(|_| AuthError::Timeout("Timed out waiting for callback".into()))?;

    let (mut socket, _) = accept_result.map_err(|e| {
        log::error!("Failed to accept connection: {e}");
        AuthError::Server(format!("Failed to accept connection: {e}"))
    })?;

    let mut buffer = [0; 1024];
    socket
        .read(&mut buffer)
        .await
        .map_err(|e| AuthError::Server(format!("Failed to read from socket: {e}")))?;

    let request = String::from_utf8_lossy(&buffer);
    let code = request
        .find("code=")
        .and_then(|start| {
            request[start..]
                .find(" ")
                .map(|end| &request[start + 5..start + end])
        })
        .ok_or_else(|| AuthError::Parse("Failed to extract code from request".into()))?;

    let client = Client::new();
    let params = [
        ("client_id", SPOTIFY_CLIENT_ID),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", SPOTIFY_REDIRECT_URI),
        ("code_verifier", &code_verifier),
    ];

    // Send the token request
    let res = client
        .post("https://accounts.spotify.com/api/token")
        .form(&params)
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let error_text = res
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error response".to_string());
        log::error!(
            "Token request failed with status {status}: {error_text}"
        );
        return Err(AuthError::Api(error_text, Some(status.as_u16())));
    }

    let body = res.json::<serde_json::Value>().await?;

    // Validate response fields
    if body.get("access_token").is_none()
        || body.get("token_type").is_none()
        || body.get("expires_in").is_none()
        || body.get("refresh_token").is_none()
        || body.get("scope").is_none()
    {
        debug!("Invalid response from Spotify: {body:#?}");

        let error_html =
            FROSTIFY_LOGIN_ERROR_HTML.replace("LOGO_BASE64", &get_frostify_logo_base64());
        let http_response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/html; charset=utf-8\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            error_html.len(),
            error_html
        );
        socket.write_all(http_response.as_bytes()).await?;
        socket.flush().await?;

        // Add a small delay before shutting down to ensure the browser receives everything
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let _ = socket.shutdown().await;

        return Err(AuthError::Parse("Invalid response from Spotify".into()));
    }

    let access_token = body["access_token"].as_str().unwrap().to_string();
    let token_type = body["token_type"].as_str().unwrap().to_string();
    let expires_in = body["expires_in"].as_u64().unwrap();
    let refresh_token = body["refresh_token"].as_str().unwrap().to_string();
    let scope = body["scope"].as_str().unwrap().to_string();

    // In the listen_for_callback function, replace the response sending part with:
    let success_html =
        FROSTIFY_LOGIN_SUCCESS_HTML.replace("LOGO_BASE64", &get_frostify_logo_base64());
    let http_response = format!(
        "HTTP/1.1 200 OK\r\n\
     Content-Type: text/html; charset=utf-8\r\n\
     Content-Length: {}\r\n\
     Connection: close\r\n\
     \r\n\
     {}",
        success_html.len(),
        success_html
    );

    socket.write_all(http_response.as_bytes()).await?;
    socket.flush().await?;

    // Add a small delay before shutting down to ensure the browser receives everything
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let _ = socket.shutdown().await;

    Ok(SpotifyAuthResponse {
        access_token,
        token_type,
        expires_in,
        refresh_token,
        scope,
    })
}

// Updated to use AuthError
pub async fn refresh_token(refresh_token: &str) -> Result<SpotifyAuthResponse, AuthError> {
    let client = Client::new();
    let params = [
        ("client_id", SPOTIFY_CLIENT_ID),
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];

    // Send refresh token request
    let res = client
        .post("https://accounts.spotify.com/api/token")
        .form(&params)
        .send()
        .await?;

    if !res.status().is_success() {
        let status = res.status();
        let error_text = res
            .text()
            .await
            .unwrap_or_else(|_| "Could not read error response".to_string());
        log::error!(
            "Refresh token request failed with status {status}: {error_text}"
        );
        return Err(AuthError::Api(error_text, Some(status.as_u16())));
    }

    let body = res.json::<serde_json::Value>().await?;

    // The refresh token response might not include a new refresh token
    // In that case, we should reuse the existing one
    let new_refresh_token = body["refresh_token"]
        .as_str()
        .unwrap_or(refresh_token)
        .to_string();

    Ok(SpotifyAuthResponse {
        access_token: body["access_token"].as_str().unwrap().to_string(),
        token_type: body["token_type"].as_str().unwrap().to_string(),
        expires_in: body["expires_in"].as_u64().unwrap(),
        refresh_token: new_refresh_token,
        scope: body["scope"]
            .as_str()
            .unwrap_or(SPOTIFY_ACCESS_SCOPES)
            .to_string(),
    })
}

fn get_frostify_logo_base64() -> String {
    let logo_bytes = get_asset("frostify_logo.png").unwrap();
    general_purpose::STANDARD.encode(logo_bytes)
}
