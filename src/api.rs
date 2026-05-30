#![allow(dead_code)]

use std::time::Instant;

use serde::Deserialize;

use crate::errors::AuthError;

const API: &str = "https://api.spotify.com/v1";

#[derive(Debug, Clone)]
pub struct Profile {
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlaylistRef {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RecentTrack {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub album_image_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArtistRef {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TrackRef {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub album_image_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AlbumRef {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub image_url: Option<String>,
    /// `YYYY-MM-DD`, `YYYY-MM`, or `YYYY` — Spotify's precision varies.
    pub release_date: String,
}

#[derive(Debug, Clone, Default)]
pub struct HomeData {
    pub profile: Option<Profile>,
    pub playlists: Vec<PlaylistRef>,
    pub recent: Vec<RecentTrack>,
    pub top_artists: Vec<ArtistRef>,
    pub top_tracks: Vec<TrackRef>,
    /// Newest album from the user's #1 top artist — our "New release"
    /// stand-in. `/v1/browse/new-releases` got deprecated for new apps
    /// in Nov 2024 alongside featured-playlists + recommendations.
    pub latest_release: Option<AlbumRef>,
}

#[derive(Debug, Clone)]
pub struct CurrentlyPlaying {
    pub track_id: String,
    pub name: String,
    pub artist: String,
    pub album_image_url: Option<String>,
    pub is_playing: bool,
    /// Position at the moment `progress_anchor` was sampled. Cluster
    /// updates push only on state transitions (play/pause/seek/track),
    /// not on a tick — so this is a snapshot, not a live position.
    /// Call `live_progress_ms` for an interpolated value.
    pub progress_ms: u64,
    /// Local wall-clock at the time the anchor was captured. Used to
    /// interpolate progress between cluster pushes.
    pub progress_anchor: Instant,
    pub duration_ms: u64,
    pub shuffle: bool,
    pub repeat: RepeatMode,
}

impl CurrentlyPlaying {
    /// Position right now, interpolated from `progress_ms + (now - anchor)`
    /// while playing. Clamps to `duration_ms`. Mirrors how the official
    /// Spotify client ticks its progress bar between server pushes.
    pub fn live_progress_ms(&self) -> u64 {
        if !self.is_playing {
            return self.progress_ms.min(self.duration_ms);
        }
        let elapsed = Instant::now()
            .saturating_duration_since(self.progress_anchor)
            .as_millis() as u64;
        (self.progress_ms + elapsed).min(self.duration_ms)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatMode {
    #[default]
    Off,
    Track,
    Context,
}

pub async fn get_me(token: &str) -> Result<Profile, AuthError> {
    #[derive(Deserialize)]
    struct R {
        #[serde(default)]
        display_name: String,
        #[serde(default)]
        images: Vec<Img>,
    }
    #[derive(Deserialize)]
    struct Img {
        url: String,
    }
    let r: R = get_json(token, &format!("{API}/me")).await?;
    Ok(Profile {
        display_name: r.display_name,
        avatar_url: r.images.into_iter().next().map(|i| i.url),
    })
}

pub async fn get_playlists(token: &str) -> Result<Vec<PlaylistRef>, AuthError> {
    #[derive(Deserialize)]
    struct R {
        items: Vec<Item>,
    }
    #[derive(Deserialize)]
    struct Item {
        id: String,
        #[serde(default)]
        name: String,
        #[serde(default)]
        images: Vec<Img>,
    }
    #[derive(Deserialize)]
    struct Img {
        url: String,
    }
    let r: R = get_json(token, &format!("{API}/me/playlists?limit=20")).await?;
    Ok(r.items
        .into_iter()
        .map(|p| PlaylistRef {
            id: p.id,
            name: p.name,
            image_url: p.images.into_iter().next().map(|i| i.url),
        })
        .collect())
}

pub async fn get_recently_played(token: &str) -> Result<Vec<RecentTrack>, AuthError> {
    #[derive(Deserialize)]
    struct R {
        items: Vec<Item>,
    }
    #[derive(Deserialize)]
    struct Item {
        track: Track,
    }
    #[derive(Deserialize)]
    struct Track {
        #[serde(default)]
        id: String,
        #[serde(default)]
        name: String,
        #[serde(default)]
        artists: Vec<Artist>,
        album: Album,
    }
    #[derive(Deserialize)]
    struct Artist {
        #[serde(default)]
        name: String,
    }
    #[derive(Deserialize)]
    struct Album {
        #[serde(default)]
        images: Vec<Img>,
    }
    #[derive(Deserialize)]
    struct Img {
        url: String,
    }
    let r: R = get_json(token, &format!("{API}/me/player/recently-played?limit=10")).await?;
    Ok(r.items
        .into_iter()
        .map(|i| RecentTrack {
            id: i.track.id,
            name: i.track.name,
            artist: i
                .track
                .artists
                .into_iter()
                .next()
                .map(|a| a.name)
                .unwrap_or_default(),
            album_image_url: i.track.album.images.into_iter().next().map(|i| i.url),
        })
        .collect())
}

/// User's top artists for the past ~4 weeks (`short_term`). Up to
/// `limit` items, highest-rank first.
pub async fn get_top_artists(token: &str, limit: u32) -> Result<Vec<ArtistRef>, AuthError> {
    #[derive(Deserialize)]
    struct R {
        items: Vec<Item>,
    }
    #[derive(Deserialize)]
    struct Item {
        #[serde(default)]
        id: String,
        #[serde(default)]
        name: String,
        #[serde(default)]
        images: Vec<Img>,
    }
    #[derive(Deserialize)]
    struct Img {
        url: String,
    }
    let url = format!("{API}/me/top/artists?time_range=short_term&limit={limit}");
    let r: R = get_json(token, &url).await?;
    Ok(r.items
        .into_iter()
        .map(|a| ArtistRef {
            id: a.id,
            name: a.name,
            image_url: a.images.into_iter().next().map(|i| i.url),
        })
        .collect())
}

/// User's top tracks for the past ~4 weeks (`short_term`).
pub async fn get_top_tracks(token: &str, limit: u32) -> Result<Vec<TrackRef>, AuthError> {
    #[derive(Deserialize)]
    struct R {
        items: Vec<Item>,
    }
    #[derive(Deserialize)]
    struct Item {
        #[serde(default)]
        id: String,
        #[serde(default)]
        name: String,
        #[serde(default)]
        artists: Vec<Artist>,
        album: Album,
    }
    #[derive(Deserialize)]
    struct Artist {
        #[serde(default)]
        name: String,
    }
    #[derive(Deserialize)]
    struct Album {
        #[serde(default)]
        images: Vec<Img>,
    }
    #[derive(Deserialize)]
    struct Img {
        url: String,
    }
    let url = format!("{API}/me/top/tracks?time_range=short_term&limit={limit}");
    let r: R = get_json(token, &url).await?;
    Ok(r.items
        .into_iter()
        .map(|t| TrackRef {
            id: t.id,
            name: t.name,
            artist: t.artists.into_iter().next().map(|a| a.name).unwrap_or_default(),
            album_image_url: t.album.images.into_iter().next().map(|i| i.url),
        })
        .collect())
}

/// Albums by an artist, sorted newest-first by `release_date`. We
/// request `include_groups=album,single` (skip appearances + compilations)
/// and re-sort client-side because Spotify's default order is not
/// guaranteed to be by date.
pub async fn get_artist_albums(
    token: &str,
    artist_id: &str,
    limit: u32,
) -> Result<Vec<AlbumRef>, AuthError> {
    #[derive(Deserialize)]
    struct R {
        items: Vec<Item>,
    }
    #[derive(Deserialize)]
    struct Item {
        #[serde(default)]
        id: String,
        #[serde(default)]
        name: String,
        #[serde(default)]
        artists: Vec<Artist>,
        #[serde(default)]
        images: Vec<Img>,
        #[serde(default)]
        release_date: String,
    }
    #[derive(Deserialize)]
    struct Artist {
        #[serde(default)]
        name: String,
    }
    #[derive(Deserialize)]
    struct Img {
        url: String,
    }
    let url = format!(
        "{API}/artists/{artist_id}/albums?include_groups=album,single&limit={limit}"
    );
    let r: R = get_json(token, &url).await?;
    let mut albums: Vec<AlbumRef> = r
        .items
        .into_iter()
        .map(|a| AlbumRef {
            id: a.id,
            name: a.name,
            artist: a.artists.into_iter().next().map(|a| a.name).unwrap_or_default(),
            image_url: a.images.into_iter().next().map(|i| i.url),
            release_date: a.release_date,
        })
        .collect();
    // Lexicographic sort on `YYYY[-MM[-DD]]` is chronological.
    albums.sort_by(|a, b| b.release_date.cmp(&a.release_date));
    Ok(albums)
}

/// Bare-ID lookup against `/v1/tracks/{id}`. Used to fill the artist
/// name (which `ProvidedTrack.metadata` doesn't carry — only an
/// `artist_uri`) on each `track_id` change.
pub async fn get_track(token: &str, track_id: &str) -> Result<TrackDetails, AuthError> {
    #[derive(Deserialize)]
    struct R {
        #[serde(default)]
        id: String,
        #[serde(default)]
        artists: Vec<Artist>,
        #[serde(default)]
        album: Album,
    }
    #[derive(Deserialize)]
    struct Artist {
        #[serde(default)]
        name: String,
    }
    #[derive(Deserialize, Default)]
    struct Album {
        #[serde(default)]
        images: Vec<Img>,
    }
    #[derive(Deserialize)]
    struct Img {
        url: String,
    }
    let r: R = get_json(token, &format!("{API}/tracks/{track_id}")).await?;
    Ok(TrackDetails {
        track_id: r.id,
        artist: r.artists.into_iter().next().map(|a| a.name).unwrap_or_default(),
        album_image_url: r.album.images.into_iter().next().map(|i| i.url),
    })
}

#[derive(Debug, Clone)]
pub struct TrackDetails {
    pub track_id: String,
    pub artist: String,
    pub album_image_url: Option<String>,
}

/// Strip the `spotify:track:` URI prefix to get the bare ID Web API needs.
/// Returns `None` if the input isn't a track URI.
pub fn track_id_from_uri(uri: &str) -> Option<&str> {
    uri.strip_prefix("spotify:track:")
}

pub async fn get_currently_playing(token: &str) -> Result<Option<CurrentlyPlaying>, AuthError> {
    #[derive(Deserialize)]
    struct R {
        #[serde(default)]
        is_playing: bool,
        #[serde(default)]
        progress_ms: u64,
        #[serde(default)]
        shuffle_state: bool,
        #[serde(default)]
        repeat_state: String,
        item: Option<Track>,
    }
    #[derive(Deserialize)]
    struct Track {
        #[serde(default)]
        id: String,
        #[serde(default)]
        name: String,
        #[serde(default)]
        duration_ms: u64,
        #[serde(default)]
        artists: Vec<Artist>,
        #[serde(default)]
        album: Album,
    }
    #[derive(Deserialize)]
    struct Artist {
        #[serde(default)]
        name: String,
    }
    #[derive(Deserialize, Default)]
    struct Album {
        #[serde(default)]
        images: Vec<Img>,
    }
    #[derive(Deserialize)]
    struct Img {
        url: String,
    }

    let res = reqwest::Client::new()
        .get(format!("{API}/me/player"))
        .bearer_auth(token)
        .send()
        .await?;
    let status = res.status();
    if status.as_u16() == 204 {
        return Ok(None);
    }
    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(AuthError::Api(body, Some(status.as_u16())));
    }
    let r: R = res.json().await?;
    let Some(item) = r.item else { return Ok(None) };
    let repeat = match r.repeat_state.as_str() {
        "track" => RepeatMode::Track,
        "context" => RepeatMode::Context,
        _ => RepeatMode::Off,
    };
    Ok(Some(CurrentlyPlaying {
        track_id: item.id,
        name: item.name,
        artist: item.artists.into_iter().next().map(|a| a.name).unwrap_or_default(),
        album_image_url: item.album.images.into_iter().next().map(|i| i.url),
        is_playing: r.is_playing,
        progress_ms: r.progress_ms,
        progress_anchor: Instant::now(),
        duration_ms: item.duration_ms,
        shuffle: r.shuffle_state,
        repeat,
    }))
}

/// Transport control against the user's **active** Connect device.
/// These hit the Web API player endpoints (not librespot/Spirc) on
/// purpose: Frostify registers as a Connect device with a NullSink, so
/// taking over via Spirc would route audio into silence. The Web API
/// commands instead steer whatever device is already playing (phone,
/// desktop app, etc.), and the dealer cluster subscription pushes the
/// resulting state change back so the UI reflects it.
///
/// All endpoints return `204 No Content` on success. A `404` means
/// "no active device" — surfaced as `AuthError::Api` for the caller to
/// log; there's nothing to control until the user starts playback
/// somewhere.
async fn player_command(
    token: &str,
    method: reqwest::Method,
    path: &str,
) -> Result<(), AuthError> {
    let res = reqwest::Client::new()
        .request(method, format!("{API}{path}"))
        .bearer_auth(token)
        // PUT/POST with an empty body — Spotify rejects a missing
        // Content-Length on some of these, so set it explicitly.
        .header(reqwest::header::CONTENT_LENGTH, 0)
        .send()
        .await?;
    let status = res.status();
    if status.is_success() {
        return Ok(());
    }
    let body = res.text().await.unwrap_or_default();
    Err(AuthError::Api(body, Some(status.as_u16())))
}

pub async fn play(token: &str) -> Result<(), AuthError> {
    player_command(token, reqwest::Method::PUT, "/me/player/play").await
}

pub async fn pause(token: &str) -> Result<(), AuthError> {
    player_command(token, reqwest::Method::PUT, "/me/player/pause").await
}

pub async fn next_track(token: &str) -> Result<(), AuthError> {
    player_command(token, reqwest::Method::POST, "/me/player/next").await
}

pub async fn previous_track(token: &str) -> Result<(), AuthError> {
    player_command(token, reqwest::Method::POST, "/me/player/previous").await
}

pub async fn set_shuffle(token: &str, on: bool) -> Result<(), AuthError> {
    player_command(token, reqwest::Method::PUT, &format!("/me/player/shuffle?state={on}")).await
}

pub async fn set_repeat(token: &str, mode: RepeatMode) -> Result<(), AuthError> {
    let state = match mode {
        RepeatMode::Off => "off",
        RepeatMode::Track => "track",
        RepeatMode::Context => "context",
    };
    player_command(token, reqwest::Method::PUT, &format!("/me/player/repeat?state={state}")).await
}

pub async fn seek(token: &str, position_ms: u32) -> Result<(), AuthError> {
    player_command(
        token,
        reqwest::Method::PUT,
        &format!("/me/player/seek?position_ms={position_ms}"),
    )
    .await
}

async fn get_json<T: for<'de> Deserialize<'de>>(token: &str, url: &str) -> Result<T, AuthError> {
    let res = reqwest::Client::new()
        .get(url)
        .bearer_auth(token)
        .send()
        .await?;
    if !res.status().is_success() {
        let status = res.status().as_u16();
        let body = res.text().await.unwrap_or_default();
        return Err(AuthError::Api(body, Some(status)));
    }
    Ok(res.json::<T>().await?)
}
