use std::time::Instant;

use futures::StreamExt;
use librespot_core::dealer::Subscription;
use librespot_core::dealer::protocol::PayloadValue;
use librespot_protocol::connect::ClusterUpdate;
use librespot_protocol::player::PlayerState as ProtoPlayerState;
use log::{debug, info, warn};
use protobuf::Message as _;

use crate::api::{CurrentlyPlaying, RepeatMode};

/// Drain the dealer `hm://connect-state/v1/cluster` subscription forever,
/// emitting our domain `CurrentlyPlaying` for each cluster update.
///
/// Spotify's cluster carries the *globally-active* playback — the same
/// state the official app shows regardless of which device is the
/// audio output. So even if Frostify isn't the active device, we still
/// reflect what the user's phone / web player is doing.
pub async fn run<F>(mut sub: Subscription, mut on_update: F)
where
    F: FnMut(Option<CurrentlyPlaying>),
{
    info!("cluster listener started — awaiting connect-state pushes");
    while let Some(msg) = sub.next().await {
        let bytes = match msg.payload {
            PayloadValue::Raw(b) => b,
            PayloadValue::Empty => {
                debug!("cluster msg with empty payload — skipping");
                continue;
            }
            PayloadValue::Json(j) => {
                debug!("cluster msg unexpectedly JSON-encoded: {j}");
                continue;
            }
        };
        let update = match ClusterUpdate::parse_from_bytes(&bytes) {
            Ok(u) => u,
            Err(e) => {
                warn!("failed to parse ClusterUpdate protobuf: {e}");
                continue;
            }
        };
        info!(
            "cluster update: reason={:?} ack={} devices_changed={:?}",
            update.update_reason, update.ack_id, update.devices_that_changed
        );
        let Some(cluster) = update.cluster.into_option() else {
            info!("  cluster: <empty>");
            on_update(None);
            continue;
        };
        info!(
            "  cluster: active_device={} devices={:?}",
            cluster.active_device_id,
            cluster.device.keys().collect::<Vec<_>>()
        );
        let Some(state) = cluster.player_state.into_option() else {
            info!("  player_state: <empty>");
            on_update(None);
            continue;
        };
        if !state.track.metadata.is_empty() {
            info!(
                "  track metadata keys: {:?}",
                state.track.metadata.keys().collect::<Vec<_>>()
            );
        }
        let cp = into_currently_playing(state);
        info!(
            "  -> CurrentlyPlaying: name='{}' artist='{}' playing={} progress={}/{} img={:?}",
            cp.name, cp.artist, cp.is_playing, cp.progress_ms, cp.duration_ms, cp.album_image_url
        );
        on_update(Some(cp));
    }
    debug!("cluster subscription stream ended");
}

fn into_currently_playing(state: ProtoPlayerState) -> CurrentlyPlaying {
    let track = state.track.unwrap_or_default();
    let md = &track.metadata;
    let name = md.get("title").cloned().unwrap_or_default();
    let artist = md
        .get("artist_name")
        .cloned()
        .or_else(|| md.get("artist_name:0").cloned())
        .unwrap_or_default();
    // Prefer the largest variant Spotify offers (xlarge/large ≈ 640px)
    // so the now-playing pane + full-window backdrop render crisp. The
    // 56px player-bar thumb bilinear-downsamples from the same handle.
    // Medium (`image_url` ≈ 300px) and small are last-resort fallbacks.
    let album_image_url = md
        .get("image_xlarge_url")
        .or_else(|| md.get("image_large_url"))
        .or_else(|| md.get("image_url"))
        .or_else(|| md.get("image_small_url"))
        .map(|s| spotify_image_uri_to_https(s));

    let is_playing = !state.is_paused && state.is_playing;
    let progress_ms = state.position_as_of_timestamp.max(0) as u64;
    let progress_anchor = Instant::now();
    let duration_ms = state.duration.max(0) as u64;

    // Repeat enum on PlayerState comes through as `repeating_context`
    // / `repeating_track` flags — Spotify doesn't model the three-state
    // explicitly here.
    let opts = state.options.unwrap_or_default();
    let repeat = if opts.repeating_track {
        RepeatMode::Track
    } else if opts.repeating_context {
        RepeatMode::Context
    } else {
        RepeatMode::Off
    };
    let shuffle = opts.shuffling_context;

    let track_id = track.uri.clone();

    CurrentlyPlaying {
        track_id,
        name,
        artist,
        album_image_url,
        is_playing,
        progress_ms,
        progress_anchor,
        duration_ms,
        shuffle,
        repeat,
    }
}

/// `spotify:image:HEX` → `https://i.scdn.co/image/HEX`. Pass through any
/// other shape (already https, or unknown) untouched.
fn spotify_image_uri_to_https(uri: &str) -> String {
    if let Some(hex) = uri.strip_prefix("spotify:image:") {
        format!("https://i.scdn.co/image/{hex}")
    } else {
        uri.to_string()
    }
}
