#![allow(dead_code)]

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

#[derive(Debug, Clone, Default)]
pub struct HomeData {
    pub profile: Option<Profile>,
    pub playlists: Vec<PlaylistRef>,
    pub recent: Vec<RecentTrack>,
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
