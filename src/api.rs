//! Spotify Web API bindings + the app's domain structs parsed from them.
//!
//! `dead_code` is allowed module-wide on purpose: this is a data-binding
//! layer that captures the full shape of each entity (ids, totals, avatar
//! URLs, …) even where the UI doesn't consume every field *yet*. Those
//! fields are wired up as features land (clickable tiles need `id`, the
//! profile chip needs `avatar_url`, etc.) — they are scaffolding, not rot.
#![allow(dead_code)]

use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::disk_cache;
use crate::errors::AuthError;

const API: &str = "https://api.spotify.com/v1";

/// Cache TTLs per endpoint class — the single knob for "how long is a
/// cached Web API response good for". Every [`get_json`] caller picks one;
/// adding an endpoint is a one-line choice here, and the response is then
/// cached on disk automatically (see [`get_json`]). Use [`ttl::NONE`] to
/// bypass the cache for live/volatile reads.
///
/// Cache entries are keyed by URL only (query params included), not by the
/// authenticated user — fine for this single-account app; sign-out clears
/// the cache via the settings "Clear cache" button.
pub mod ttl {
    use std::time::Duration;
    const HOUR: u64 = 3600;
    const DAY: u64 = 24 * HOUR;
    /// Immutable resources keyed by id (track + album metadata): never
    /// change, so the long bound just caps growth.
    pub const IMMUTABLE: Duration = Duration::from_secs(30 * DAY);
    /// Slowly-changing user data: profile, top artists/tracks (recomputed
    /// ~weekly), artist discography (catch new releases within a day).
    pub const SLOW: Duration = Duration::from_secs(DAY);
    /// User-editable collections: playlist list + metadata + track pages.
    pub const MUTABLE: Duration = Duration::from_secs(6 * HOUR);
    /// Volatile feeds: recently-played.
    pub const VOLATILE: Duration = Duration::from_secs(10 * 60);
    /// Bypass the cache entirely (live player state).
    pub const NONE: Duration = Duration::ZERO;
}

/// Disk-cache key for a GET URL: a stable hash of the full URL (query
/// params included, so each distinct request gets its own entry). The
/// `DefaultHasher` seed is fixed, so the key is stable across runs. Bearer
/// token lives in a header, not the URL, so token refresh never busts it.
fn url_key(url: &str) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut h);
    format!("api_{:016x}", h.finish())
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub display_name: String,
    pub avatar_url: Option<String>,
    /// ISO 3166-1 alpha-2 country (from `user-read-private`). Used as the
    /// `market` for endpoints that require one (artist top-tracks).
    pub country: String,
}

#[derive(Debug, Clone)]
pub struct PlaylistRef {
    pub id: String,
    pub name: String,
    /// Full-res (640 px) cover — the "Made For You" home tile.
    pub image_url: Option<String>,
    /// Tiny (64 px) cover — the sidebar library icon. Same album, smaller
    /// scdn tier (sidebar rows are ~48 logical px); kept separate so the
    /// two consumers don't share one over- or under-sized fetch.
    pub image_url_small: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RecentTrack {
    pub id: String,
    pub name: String,
    pub artist: String,
    /// Album id — the tile opens this album's detail page.
    pub album_id: String,
    pub album_image_url: Option<String>,
    /// ISO-8601 `played_at` timestamp (`YYYY-MM-DDT…Z`); the leading date
    /// drives the "Today/Yesterday/…" grouping on the Show-all page.
    pub played_at: String,
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
    /// Album id — the tile opens this album's detail page.
    pub album_id: String,
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

/// A single track inside a playlist (or the Liked Songs collection).
/// `uri` is the `spotify:track:…` form Web API playback needs; `id` is
/// the bare hex (album-art cache keys etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub id: String,
    pub uri: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub album_image_url: Option<String>,
    pub duration_ms: u64,
}

/// A fully-loaded playlist (metadata + first page of tracks). Liked
/// Songs is modelled as one of these with a synthetic name/owner and
/// `context_uri = None` (it has no playable context URI on the Web API,
/// so playback falls back to an explicit `uris` list — see [`PlayTarget`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistDetail {
    pub id: String,
    pub name: String,
    pub owner: String,
    pub image_url: Option<String>,
    /// `spotify:playlist:…` for real playlists; `None` for Liked Songs.
    pub context_uri: Option<String>,
    pub tracks: Vec<PlaylistTrack>,
    /// Total tracks reported by Spotify (may exceed `tracks.len()` since
    /// we only load the first page).
    pub total: u32,
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
        images: Vec<RawImg>,
        #[serde(default)]
        country: String,
    }
    let r: R = get_json(token, &format!("{API}/me"), ttl::SLOW).await?;
    Ok(Profile {
        display_name: r.display_name,
        avatar_url: pick_thumb(&r.images),
        country: r.country,
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
        images: Vec<RawImg>,
    }
    let r: R = get_json(token, &format!("{API}/me/playlists?limit=20"), ttl::MUTABLE).await?;
    Ok(r.items
        .into_iter()
        .map(|p| PlaylistRef {
            id: p.id,
            name: p.name,
            // Full res for the "Made For You" home tile; tiny for the
            // sidebar library icon — same album, two scdn tiers.
            image_url: pick_full(&p.images),
            image_url_small: pick_tiny(&p.images),
        })
        .collect())
}

/// Sentinel id used everywhere (nav state, cache key) to mean the Liked
/// Songs collection rather than a real playlist.
pub const LIKED_SONGS_ID: &str = "__liked__";

// Shared deserialize shapes for the playlist + saved-tracks endpoints —
// both wrap items in `{ track: <RawTrack> }`, so one set of structs maps
// both.
#[derive(Deserialize)]
struct RawItem {
    track: Option<RawTrack>,
}
#[derive(Deserialize)]
struct RawTrack {
    #[serde(default)]
    id: String,
    #[serde(default)]
    uri: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    duration_ms: u64,
    #[serde(default)]
    artists: Vec<RawArtist>,
    #[serde(default)]
    album: RawAlbum,
}
#[derive(Deserialize)]
struct RawArtist {
    #[serde(default)]
    name: String,
}
#[derive(Deserialize, Default)]
struct RawAlbum {
    #[serde(default)]
    name: String,
    #[serde(default)]
    images: Vec<RawImg>,
}
#[derive(Deserialize)]
struct RawImg {
    url: String,
    /// Spotify reports each image's pixel width (`null` for some sources).
    #[serde(default)]
    width: Option<u32>,
}

/// Pick the **smallest** image whose width is ≥ `min_w`. Spotify returns
/// a widest-first `[640, 300, 64]` array per album/artist. Matching the
/// fetched resolution to the on-screen display size is a big win: a
/// playlist row thumb shows at ~40 logical px (~80 physical at 2× DPI),
/// so the 640 px cover is ~250× the pixels drawn — decoding + uploading
/// 640² (1.6 MB) per row is what stalls a fast scroll over a large list.
/// The ~300 px variant is crisp at every thumb/tile size here and ~5×
/// cheaper to fetch + decode + upload. Falls back to the first (largest)
/// entry when nothing meets `min_w` or width metadata is absent.
fn pick_image_at_least(images: &[RawImg], min_w: u32) -> Option<String> {
    let mut best: Option<(u32, &str)> = None;
    let mut any: Option<&str> = None;
    for img in images {
        if any.is_none() {
            any = Some(&img.url);
        }
        if let Some(w) = img.width
            && w >= min_w
            && best.as_ref().map(|(bw, _)| w < *bw).unwrap_or(true)
        {
            best = Some((w, &img.url));
        }
    }
    best.map(|(_, u)| u).or(any).map(str::to_string)
}

// Spotify album art comes in three fixed tiers — **640 / 300 / 64 px**
// (640 is the ceiling; there is no higher variant via the API). Match the
// fetched tier to the on-screen display box (× 2 for DPI) so images are
// crisp without over-fetching:
//
//   • now-playing cover, home tiles, "new release" card, playlist header
//     — display ≥ ~160 logical px (≥ 320 physical) → the **640** tier
//     (300 upscaled into a 320 px box reads slightly soft).
//   • playlist track rows — ~40 logical px, but there are 1000s of them,
//     so the **300** tier keeps fast-scroll fetch/decode/upload cheap.
//   • sidebar library rows — tiny (~48 logical px) and few, so the **64**
//     tier is plenty and lightest.

/// Playlist-row thumbs (~300 px): smallest variant ≥ this.
const ROW_MIN_W: u32 = 160;
/// Sidebar library icons (~64 px): smallest variant ≥ this.
const SIDEBAR_MIN_W: u32 = 48;

/// Row-thumb pick (≈ 300 px) — for the virtualized playlist track list.
fn pick_thumb(images: &[RawImg]) -> Option<String> {
    pick_image_at_least(images, ROW_MIN_W)
}

/// Tiny pick (≈ 64 px) — sidebar library icons.
fn pick_tiny(images: &[RawImg]) -> Option<String> {
    pick_image_at_least(images, SIDEBAR_MIN_W)
}

/// Full-resolution pick: the largest (640 px) variant. Now-playing cover,
/// home tiles, "new release" card, playlist header — anything shown large
/// enough that 300 px upscales visibly.
fn pick_full(images: &[RawImg]) -> Option<String> {
    pick_image_at_least(images, u32::MAX)
}

impl RawTrack {
    fn into_track(self) -> PlaylistTrack {
        PlaylistTrack {
            id: self.id,
            uri: self.uri,
            name: self.name,
            artist: self
                .artists
                .into_iter()
                .next()
                .map(|a| a.name)
                .unwrap_or_default(),
            album: self.album.name,
            album_image_url: pick_thumb(&self.album.images),
            duration_ms: self.duration_ms,
        }
    }
}

fn tracks_from_items(items: Vec<RawItem>) -> Vec<PlaylistTrack> {
    items
        .into_iter()
        .filter_map(|i| i.track)
        .filter(|t| !t.id.is_empty())
        .map(RawTrack::into_track)
        .collect()
}

/// Page size for the streaming track loads. Playlist-tracks endpoint
/// caps at 100; saved-tracks (`/me/tracks`) caps at 50.
pub const PLAYLIST_PAGE: u32 = 100;
pub const LIKED_PAGE: u32 = 50;

/// Lightweight playlist metadata (no tracks) — fetched first so the
/// header + scrollbar length appear before any track page lands.
#[derive(Debug, Clone)]
pub struct PlaylistMeta {
    pub name: String,
    pub owner: String,
    pub image_url: Option<String>,
    pub total: u32,
}

pub async fn playlist_meta(token: &str, playlist_id: &str) -> Result<PlaylistMeta, AuthError> {
    #[derive(Deserialize)]
    struct R {
        #[serde(default)]
        name: String,
        #[serde(default)]
        owner: Owner,
        #[serde(default)]
        images: Vec<RawImg>,
        #[serde(default)]
        tracks: TotalOnly,
    }
    #[derive(Deserialize, Default)]
    struct Owner {
        #[serde(default)]
        display_name: String,
    }
    #[derive(Deserialize, Default)]
    struct TotalOnly {
        #[serde(default)]
        total: u32,
    }
    let fields = "name,owner(display_name),images,tracks.total";
    let r: R = get_json(
        token,
        &format!("{API}/playlists/{playlist_id}?fields={fields}"),
        ttl::MUTABLE,
    )
    .await?;
    Ok(PlaylistMeta {
        name: r.name,
        owner: r.owner.display_name,
        // Playlist header cover (large) — full res.
        image_url: pick_full(&r.images),
        total: r.tracks.total,
    })
}

/// One page of tracks plus the endpoint's reported `total` and the raw
/// item count (incl. nulls — needed to detect the last page when some
/// entries get filtered out).
#[derive(Debug, Clone)]
pub struct TracksPage {
    pub tracks: Vec<PlaylistTrack>,
    pub total: u32,
    pub raw_count: u32,
}

pub async fn fetch_tracks_page(token: &str, url: &str) -> Result<TracksPage, AuthError> {
    #[derive(Deserialize)]
    struct Page {
        #[serde(default)]
        total: u32,
        #[serde(default)]
        items: Vec<RawItem>,
    }
    let page: Page = get_json(token, url, ttl::MUTABLE).await?;
    let raw_count = page.items.len() as u32;
    Ok(TracksPage {
        tracks: tracks_from_items(page.items),
        total: page.total,
        raw_count,
    })
}

/// URL for a page of a real playlist's tracks (fields-masked).
pub fn playlist_tracks_url(playlist_id: &str, offset: u32, limit: u32) -> String {
    let fields = "total,items(track(id,uri,name,duration_ms,artists(name),album(name,images)))";
    format!("{API}/playlists/{playlist_id}/tracks?limit={limit}&offset={offset}&fields={fields}")
}

/// URL for a page of the saved-tracks (Liked Songs) collection.
pub fn liked_tracks_url(offset: u32, limit: u32) -> String {
    format!("{API}/me/tracks?limit={limit}&offset={offset}")
}

pub async fn get_recently_played(token: &str) -> Result<Vec<RecentTrack>, AuthError> {
    #[derive(Deserialize)]
    struct R {
        items: Vec<Item>,
    }
    #[derive(Deserialize)]
    struct Item {
        track: Track,
        #[serde(default)]
        played_at: String,
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
        id: String,
        #[serde(default)]
        images: Vec<RawImg>,
    }
    let r: R = get_json(
        token,
        &format!("{API}/me/player/recently-played?limit=12"),
        ttl::VOLATILE,
    )
    .await?;
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
            album_id: i.track.album.id,
            // Home "Recently played" tiles (TILE_THUMB ≈ 320 px physical).
            album_image_url: pick_full(&i.track.album.images),
            played_at: i.played_at,
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
        images: Vec<RawImg>,
    }
    let url = format!("{API}/me/top/artists?time_range=short_term&limit={limit}");
    let r: R = get_json(token, &url, ttl::SLOW).await?;
    Ok(r.items
        .into_iter()
        .map(|a| ArtistRef {
            id: a.id,
            name: a.name,
            // Home "Your top artists" tiles.
            image_url: pick_full(&a.images),
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
        id: String,
        #[serde(default)]
        images: Vec<RawImg>,
    }
    let url = format!("{API}/me/top/tracks?time_range=short_term&limit={limit}");
    let r: R = get_json(token, &url, ttl::SLOW).await?;
    Ok(r.items
        .into_iter()
        .map(|t| TrackRef {
            id: t.id,
            name: t.name,
            artist: t
                .artists
                .into_iter()
                .next()
                .map(|a| a.name)
                .unwrap_or_default(),
            album_id: t.album.id,
            // Home "Your top tracks" tiles.
            album_image_url: pick_full(&t.album.images),
        })
        .collect())
}

/// Artist header info (name + image) for the artist page. Discography is
/// fetched separately via [`get_artist_albums`].
#[derive(Debug, Clone)]
pub struct ArtistDetail {
    pub id: String,
    pub name: String,
    pub image_url: Option<String>,
    /// Total followers — shown under the artist name.
    pub followers: u64,
}

/// Fetch an artist's profile (`/v1/artists/{id}`). Name changes ~never, so
/// a long-ish `SLOW` TTL is plenty.
pub async fn get_artist(token: &str, artist_id: &str) -> Result<ArtistDetail, AuthError> {
    #[derive(Deserialize)]
    struct R {
        #[serde(default)]
        id: String,
        #[serde(default)]
        name: String,
        #[serde(default)]
        images: Vec<RawImg>,
        #[serde(default)]
        followers: Followers,
    }
    #[derive(Deserialize, Default)]
    struct Followers {
        #[serde(default)]
        total: u64,
    }
    let r: R = get_json(token, &format!("{API}/artists/{artist_id}"), ttl::SLOW).await?;
    Ok(ArtistDetail {
        id: if r.id.is_empty() {
            artist_id.to_string()
        } else {
            r.id
        },
        name: r.name,
        // Artist hero image (large).
        image_url: pick_full(&r.images),
        followers: r.followers.total,
    })
}

/// An artist's most popular tracks (`/v1/artists/{id}/top-tracks`). Requires
/// a `market` (the user's country); falls back to `US` if unknown. Mapped to
/// `PlaylistTrack` so the artist page reuses the track-row rendering.
pub async fn get_artist_top_tracks(
    token: &str,
    artist_id: &str,
    market: &str,
) -> Result<Vec<PlaylistTrack>, AuthError> {
    #[derive(Deserialize)]
    struct R {
        #[serde(default)]
        tracks: Vec<RawTrack>,
    }
    let market = if market.is_empty() { "US" } else { market };
    let url = format!("{API}/artists/{artist_id}/top-tracks?market={market}");
    let r: R = get_json(token, &url, ttl::SLOW).await?;
    Ok(r.tracks
        .into_iter()
        .filter(|t| !t.id.is_empty())
        .map(RawTrack::into_track)
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
        images: Vec<RawImg>,
        #[serde(default)]
        release_date: String,
    }
    #[derive(Deserialize)]
    struct Artist {
        #[serde(default)]
        name: String,
    }
    let url = format!("{API}/artists/{artist_id}/albums?include_groups=album,single&limit={limit}");
    let r: R = get_json(token, &url, ttl::SLOW).await?;
    let mut albums: Vec<AlbumRef> = r
        .items
        .into_iter()
        .map(|a| AlbumRef {
            id: a.id,
            name: a.name,
            artist: a
                .artists
                .into_iter()
                .next()
                .map(|a| a.name)
                .unwrap_or_default(),
            // "New release" spotlight card (THUMB_XL) — full res.
            image_url: pick_full(&a.images),
            release_date: a.release_date,
        })
        .collect();
    // Lexicographic sort on `YYYY[-MM[-DD]]` is chronological.
    albums.sort_by(|a, b| b.release_date.cmp(&a.release_date));
    Ok(albums)
}

/// Full album page: metadata + first page (≤ 50) of tracks, mapped to the
/// shared [`PlaylistDetail`] so an album reuses the playlist track-list
/// pipeline (cache, view, playback). `context_uri` is `spotify:album:{id}`
/// (albums are a playable context). `owner` carries the album artist;
/// `total` is the loaded count (albums past 50 tracks aren't paged).
pub async fn get_album(token: &str, album_id: &str) -> Result<PlaylistDetail, AuthError> {
    #[derive(Deserialize)]
    struct R {
        #[serde(default)]
        name: String,
        #[serde(default)]
        artists: Vec<Artist>,
        #[serde(default)]
        images: Vec<RawImg>,
        #[serde(default)]
        tracks: Tracks,
    }
    #[derive(Deserialize, Default)]
    struct Tracks {
        #[serde(default)]
        items: Vec<Item>,
    }
    #[derive(Deserialize)]
    struct Item {
        #[serde(default)]
        id: String,
        #[serde(default)]
        uri: String,
        #[serde(default)]
        name: String,
        #[serde(default)]
        duration_ms: u64,
        #[serde(default)]
        artists: Vec<Artist>,
    }
    #[derive(Deserialize)]
    struct Artist {
        #[serde(default)]
        name: String,
    }
    let r: R = get_json(
        token,
        &format!("{API}/albums/{album_id}?limit=50"),
        ttl::IMMUTABLE,
    )
    .await?;
    let album_name = r.name.clone();
    let artist = r
        .artists
        .into_iter()
        .next()
        .map(|a| a.name)
        .unwrap_or_default();
    let image_url = pick_full(&r.images);
    let row_thumb = pick_thumb(&r.images);
    let tracks: Vec<PlaylistTrack> = r
        .tracks
        .items
        .into_iter()
        .filter(|t| !t.id.is_empty())
        .map(|t| PlaylistTrack {
            id: t.id,
            uri: t.uri,
            name: t.name,
            artist: t
                .artists
                .into_iter()
                .next()
                .map(|a| a.name)
                .unwrap_or_default(),
            album: album_name.clone(),
            // Album tracks share the album cover; row-thumb tier for the list.
            album_image_url: row_thumb.clone(),
            duration_ms: t.duration_ms,
        })
        .collect();
    let total = tracks.len() as u32;
    Ok(PlaylistDetail {
        id: album_id.to_string(),
        name: r.name,
        owner: artist,
        image_url,
        context_uri: Some(format!("spotify:album:{album_id}")),
        tracks,
        total,
    })
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
        images: Vec<RawImg>,
    }
    // Track metadata is immutable — the hot path (refetched on every
    // track change) gets the longest TTL so repeated plays never re-hit.
    let r: R = get_json(token, &format!("{API}/tracks/{track_id}"), ttl::IMMUTABLE).await?;
    Ok(TrackDetails {
        track_id: r.id,
        artist: r
            .artists
            .into_iter()
            .next()
            .map(|a| a.name)
            .unwrap_or_default(),
        // Now-playing cover (large + full-window blurred backdrop) — full res.
        album_image_url: pick_full(&r.album.images),
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
        uri: String,
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
        images: Vec<RawImg>,
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
        // Use the full `spotify:track:…` URI to match the cluster path
        // (the rest of the app — canvas fetch, track-id parsing — expects
        // the URI form, not the bare id).
        track_id: if item.uri.is_empty() {
            format!("spotify:track:{}", item.id)
        } else {
            item.uri
        },
        name: item.name,
        artist: item
            .artists
            .into_iter()
            .next()
            .map(|a| a.name)
            .unwrap_or_default(),
        // Now-playing cover — full res (large + blurred backdrop).
        album_image_url: pick_full(&item.album.images),
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
async fn player_command(token: &str, method: reqwest::Method, path: &str) -> Result<(), AuthError> {
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

/// What to start playing on the active device. Real playlists/albums use
/// a `context_uri` (so Spotify queues the whole context); the Liked
/// Songs collection has no playable context URI, so it ships an explicit
/// `uris` list. Both carry an `offset` = the index to start at.
#[derive(Debug, Clone)]
pub enum PlayTarget {
    Context { context_uri: String, offset: u32 },
    Uris { uris: Vec<String>, offset: u32 },
}

/// Start playback of a context (playlist/album) or explicit track list
/// on the user's active Connect device. Body shape mirrors the official
/// client's `PUT /me/player/play`. A `404` (no active device) surfaces
/// as `AuthError::Api` for the caller to log.
pub async fn play_context(token: &str, target: PlayTarget) -> Result<(), AuthError> {
    let body = match target {
        PlayTarget::Context {
            context_uri,
            offset,
        } => serde_json::json!({
            "context_uri": context_uri,
            "offset": { "position": offset },
        }),
        PlayTarget::Uris { uris, offset } => serde_json::json!({
            "uris": uris,
            "offset": { "position": offset },
        }),
    };
    let res = reqwest::Client::new()
        .put(format!("{API}/me/player/play"))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?;
    let status = res.status();
    if status.is_success() {
        return Ok(());
    }
    let body = res.text().await.unwrap_or_default();
    Err(AuthError::Api(body, Some(status.as_u16())))
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
    player_command(
        token,
        reqwest::Method::PUT,
        &format!("/me/player/shuffle?state={on}"),
    )
    .await
}

pub async fn set_repeat(token: &str, mode: RepeatMode) -> Result<(), AuthError> {
    let state = match mode {
        RepeatMode::Off => "off",
        RepeatMode::Track => "track",
        RepeatMode::Context => "context",
    };
    player_command(
        token,
        reqwest::Method::PUT,
        &format!("/me/player/repeat?state={state}"),
    )
    .await
}

/// Seek the active Connect device to `position_ms`. Used by the
/// (in-progress) scrubbable progress bar — drag/click to seek, with a
/// hover preview of the target timestamp.
pub async fn seek(token: &str, position_ms: u32) -> Result<(), AuthError> {
    player_command(
        token,
        reqwest::Method::PUT,
        &format!("/me/player/seek?position_ms={position_ms}"),
    )
    .await
}

/// GET + deserialize a Web API JSON endpoint, transparently cached on disk.
///
/// The disk cache is keyed by [`url_key`] and stores the **raw response
/// bytes** (not the deserialized `T`) — so every endpoint, current or
/// future, is cached for free just by passing a `ttl`; no per-endpoint
/// `Serialize` impl is needed. A non-expired cache hit skips the network
/// entirely. `ttl == ttl::NONE` (zero) bypasses the cache on both read and
/// write. Only successful (`2xx`) responses are cached, and only after they
/// parse — error bodies and malformed payloads never poison the cache.
async fn get_json<T: for<'de> Deserialize<'de>>(
    token: &str,
    url: &str,
    ttl: Duration,
) -> Result<T, AuthError> {
    let key = url_key(url);
    // Cache read (off-thread — the cache is blocking fs IO).
    if !ttl.is_zero() {
        let k = key.clone();
        if let Ok(Some(bytes)) =
            tokio::task::spawn_blocking(move || disk_cache::read_raw_json(&k, ttl)).await
            && let Ok(value) = serde_json::from_slice::<T>(&bytes)
        {
            return Ok(value);
        }
    }
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
    let bytes = res.bytes().await?;
    let value: T = serde_json::from_slice(&bytes)?;
    // Persist for next time — best-effort, off the async runtime.
    if !ttl.is_zero() {
        let raw = bytes.to_vec();
        tokio::task::spawn_blocking(move || disk_cache::write_raw_json(&key, &raw));
    }
    Ok(value)
}
