//! Worker-response reducer — routes each `WorkerResponse` to the model(s)
//! it mutates. Pure model-update logic (no view code); the frame loop
//! drains the worker and calls this per response.

use std::rc::Rc;

use crate::album_art;
use crate::api::track_id_from_uri;
use crate::app::AppState;
use crate::app::cx::Cx;
use crate::views::View;
use crate::worker::{Worker, WorkerResponse};

pub fn handle(state: &Rc<AppState>, cx: &mut Cx, worker: &Rc<Worker>, resp: WorkerResponse) {
    match resp {
        WorkerResponse::OAuthStarted { auth_url } => {
            log::info!("opening browser for OAuth");
            if let Err(e) = webbrowser::open(&auth_url) {
                log::error!("open browser: {e}");
            }
        }
        WorkerResponse::OAuthComplete { auth } | WorkerResponse::TokensLoaded { auth } => {
            log::info!("auth ok — switching to Home");
            worker.fetch_home(auth.access_token.clone());
            worker.connect_spotify_session(auth.access_token.clone());
            state.auth.set(auth);
            if state.router.view.get() != View::Home {
                state.router.view.set(View::Home);
                cx.rebuild();
            }
        }
        WorkerResponse::OAuthFailed { error } => {
            log::error!("OAuth failed: {error}");
            if state.router.view.get() != View::Login {
                state.router.view.set(View::Login);
                cx.rebuild();
            }
        }
        WorkerResponse::NoStoredTokens => {
            log::info!("no stored tokens — showing Login");
            if state.router.view.get() != View::Login {
                state.router.view.set(View::Login);
                cx.rebuild();
            }
        }
        WorkerResponse::HomeData { data } => {
            log::info!(
                "home data ready: playlists={} recent={} top_artists={} top_tracks={}",
                data.playlists.len(),
                data.recent.len(),
                data.top_artists.len(),
                data.top_tracks.len(),
            );
            state.art.prefetch(worker, &data);
            *state.library.home.borrow_mut() = data;
            cx.rebuild();
        }
        WorkerResponse::SpotifySessionConnected => {
            log::info!("librespot session ready — seeding initial /me/player state");
            if let Some(token) = state.auth.token() {
                worker.seed_player_state(token);
            }
        }
        WorkerResponse::SpotifySessionFailed { error } => {
            log::warn!("librespot session failed: {error}. Falling back to Web API polling.");
        }
        WorkerResponse::PlayerState { mut player } => {
            // Overlay cached track details (artist) and request a fetch
            // for any track we haven't resolved yet. The cluster's
            // `ProvidedTrack.metadata` only carries `artist_uri`, so the
            // artist name comes from `/v1/tracks/{id}`.
            if let Some(p) = player.as_mut() {
                if let Some(id) = track_id_from_uri(&p.track_id) {
                    match state.art.cached_artist(id) {
                        Some(artist) => p.artist = artist,
                        None => {
                            if let Some(token) = state.auth.token() {
                                worker.fetch_track_details(token, id.to_string());
                            }
                        }
                    }
                }
                // Dispatch an album-art fetch when the cover actually
                // changes. Skip when it's already what's on screen (same
                // track, just a progress tick) or a fetch is already in
                // flight. The fetch is disk-backed, so re-loading a cover
                // we've seen before is cheap and yields a fresh, tree-live
                // handle — see `art_inflight` doc for why we don't cache
                // handles across tracks.
                if let Some(url) = p.album_image_url.as_ref() {
                    let key = album_art::cache_key(url);
                    if !state.art.is_shown(&key) && !state.art.is_inflight(&key) {
                        state.art.mark_inflight(key.clone());
                        worker.fetch_album_art(url.clone(), key.clone());
                    }
                    // Spotify's own accent for this cover (authoritative over
                    // the pixel-average extracted on art decode). Dispatched
                    // **independently of the art dedup** — otherwise a cover
                    // whose art is already shown / in flight would never get
                    // its accent fetched, leaving the *previous* track's
                    // accent on screen. Gated once per cover (disk-cached).
                    if !state.art.has_accent(&key) {
                        worker.fetch_accent(key);
                    }
                }
                // Canvas video: fetch on a real track change (not a
                // progress tick). Gate on the cached canvas not already
                // matching this track id; clear any stale canvas first so
                // the UI falls back to art until the new one resolves.
                if let Some(id) = track_id_from_uri(&p.track_id) {
                    let have = state.canvas.path_matches(id);
                    log::debug!(
                        "canvas gate: track={id} have={have} show={}",
                        state.canvas.show.get()
                    );
                    if !have && state.canvas.show.get() {
                        state.canvas.clear_path();
                        // Stop the previous track's video now so it doesn't
                        // linger over the new track's art until the new
                        // Canvas (if any) resolves.
                        state.canvas.stop_decode();
                        worker.fetch_canvas(p.track_id.clone(), id.to_string());
                    }
                }
            }
            // Push every dynamic field into its reactive signal (all
            // dedup'd, so a same-track progress tick only bumps what
            // changed). Title/artist → text binds, is_playing → play/pause
            // image bind, shuffle/repeat → tint colour binds, progress →
            // % width bind. Nothing here needs a scene rebuild anymore.
            // Push the snapshot into the reactive chrome (all dedup'd, so a
            // same-track progress tick only bumps what changed). A `None`
            // (nothing playing on any device) keeps the last track visible —
            // the cold-start path seeds title/artist/progress from
            // `prefs.last_player`, so we just mark stopped + freeze the bar
            // rather than clobbering that restored state to a dash.
            match player.as_ref() {
                Some(p) => state.player_ui.sync(p, cx.tl, cx.now),
                None => state.player_ui.stopped(cx.tl),
            }
            *state.player_ui.snapshot.borrow_mut() = player;
        }
        WorkerResponse::AlbumArtReady {
            key,
            handle,
            accent,
        } => {
            state.art.clear_inflight(&key);
            // Push the resolved handle into the per-URL Home signal (if
            // any tile bound to this key) — repaints just those nodes via
            // the image bind, no rebuild.
            state.art.set_resolved(&key, handle);
            // Promote into the crossfade if this cover matches either:
            // (a) the live player (steady-state path — a live track
            //     change resolved), or
            // (b) the persisted `last_player` snapshot AND no live
            //     player has landed yet (cold-start path — disk cache
            //     hit beats the first cluster push so we'd otherwise
            //     discard the art handle and re-fetch later, costing
            //     the user a visible "blank → fade-in" delay).
            // No handle cache: the fresh handle is tree-live once
            // promoted, so it survives atlas eviction. A rapid switch
            // that moved on before the upload landed just leaves the
            // orphan handle for the atlas to evict.
            let live_match = state
                .player_ui
                .snapshot
                .borrow()
                .as_ref()
                .and_then(|p| p.album_image_url.as_ref().map(|u| album_art::cache_key(u)))
                .map(|k| k == key)
                .unwrap_or(false);
            let cold_start_match = !live_match
                && state.player_ui.snapshot.borrow().is_none()
                && state
                    .prefs
                    .data
                    .borrow()
                    .last_player
                    .as_ref()
                    .and_then(|p| p.album_image_url.as_ref().map(|u| album_art::cache_key(u)))
                    .map(|k| k == key)
                    .unwrap_or(false);
            if live_match || cold_start_match {
                // Prefer Spotify's own extracted colour if it already
                // arrived for this cover; otherwise use the pixel-average
                // as a provisional accent (a later `AccentReady` overrides
                // it). This makes the result order-independent between the
                // two parallel requests.
                let accent = state.art.accent(&key).unwrap_or(accent);
                // No rebuild: promote swaps the handles via the reactive
                // image-handle binds and starts the crossfade tween, both
                // pumped by the lib without re-running the scene closure.
                state.backdrop.promote(handle, Some(accent), cx.tl, cx.now);
                state.art.set_shown(key);
            }
        }
        WorkerResponse::AlbumArtFailed { key } => {
            state.art.clear_inflight(&key);
        }
        WorkerResponse::AccentReady { key, accent } => {
            state.art.cache_accent(key.clone(), accent);
            // Apply only if this cover is the one on screen now (or the
            // live player's) — a late arrival for a skipped track is kept
            // in the map but not tweened in. Overrides any provisional
            // pixel-average accent with Spotify's exact colour.
            let is_current = state.art.is_shown(&key)
                || state
                    .player_ui
                    .snapshot
                    .borrow()
                    .as_ref()
                    .and_then(|p| p.album_image_url.as_ref().map(|u| album_art::cache_key(u)))
                    .map(|k| k == key)
                    .unwrap_or(false);
            if is_current {
                state.backdrop.set_accent(accent, cx.tl, cx.now);
            }
        }
        WorkerResponse::CanvasReady { track_id, path } => {
            // Stage 1-2: the Canvas MP4 is fetched + cached. Frame decode
            // + texture pump (stages 3-4) and the now-playing UI swap
            // (stage 5) land next; for now record the cached path so the
            // decoder task can pick it up.
            log::info!("canvas ready for {track_id}: {}", path.display());
            state.canvas.set_path(track_id.clone(), path.clone());
            // Only decode if still wanted (canvas enabled). A late arrival
            // for a track the user already skipped past is harmless — the
            // next track change stops/replaces this session.
            if state.canvas.show.get() {
                state.canvas.start_decode(track_id, path);
            }
        }
        WorkerResponse::CanvasNone { track_id } => {
            log::debug!("no canvas for {track_id} — album art fallback");
            if state.canvas.path_matches(&track_id) {
                state.canvas.clear_path();
            }
            // No Canvas for this track → stop any running decode + fall
            // back to art.
            state.canvas.stop_decode();
        }
        WorkerResponse::PlaylistOpened { detail, complete } => {
            let id = detail.id.clone();
            // Apply to the open pane if it's still showing this playlist:
            // overwrite metadata + seed the first page, then rebuild ONCE
            // to mount the full-length virtualised list (item_count =
            // total). Subsequent pages append without a rebuild.
            let applies = state.router.nav_is_open(&id);
            if applies {
                let buf = {
                    let mut op = state.library.open_playlist.borrow_mut();
                    op.as_mut().map(|o| {
                        o.name = detail.name.clone();
                        o.owner = detail.owner.clone();
                        o.image_url = detail.image_url.clone();
                        o.context_uri = detail.context_uri.clone();
                        o.total = detail.total;
                        o.loading = false;
                        o.complete = complete;
                        o.rows.clone()
                    })
                };
                if let Some(buf) = buf {
                    buf.borrow_mut().clear();
                    state.library.build_rows(&state.art, &buf, &detail.tracks);
                    cx.rebuild();
                }
            }
            // A `complete` response (disk-cache hit or single-page) carries
            // the whole listing — cache it in memory for an instant
            // re-open and clear the inflight gate.
            if complete {
                state.library.clear_inflight(&id);
                state.library.cache(detail);
            }
        }
        WorkerResponse::PlaylistTracks { id, tracks, done } => {
            // Append a streamed page into the live buffer — no rebuild;
            // the lazy_list reads it on scroll. (Covers fill in reactively
            // via the per-row image bind baked in `build_rows`.)
            let applies = state.router.nav_is_open(&id);
            if applies {
                let buf = state
                    .library
                    .open_playlist
                    .borrow()
                    .as_ref()
                    .map(|o| o.rows.clone());
                if let Some(buf) = buf {
                    state.library.build_rows(&state.art, &buf, &tracks);
                }
                if done && let Some(o) = state.library.open_playlist.borrow_mut().as_mut() {
                    o.complete = true;
                }
            }
            if done {
                state.library.clear_inflight(&id);
            }
        }
        WorkerResponse::PlaylistFailed { id, error } => {
            state.library.clear_inflight(&id);
            log::warn!("playlist {id} load failed: {error}");
        }
        WorkerResponse::ArtistOpened {
            id,
            name,
            image_url,
            followers,
            top_tracks,
            albums,
        } => {
            state.library.clear_inflight(&id);
            if state.router.nav_is_artist(&id) {
                // Create the reactive cover signals + dispatch fetches HERE
                // (not in the view build) — the build holds an immutable
                // borrow of `home_art`, so `or_signal`'s borrow_mut there
                // panics. Later `AlbumArtReady` fills these via set_resolved.
                if let Some(u) = &image_url {
                    state.art.or_signal(album_art::cache_key(u));
                    state.art.dispatch_cover(worker, u.clone());
                }
                let covers = albums
                    .iter()
                    .filter_map(|al| al.image_url.as_ref())
                    .chain(top_tracks.iter().filter_map(|t| t.album_image_url.as_ref()));
                for u in covers {
                    state.art.or_signal(album_art::cache_key(u));
                    state.art.dispatch_cover(worker, u.clone());
                }
                if let Some(a) = state.library.open_artist.borrow_mut().as_mut() {
                    a.name = name;
                    a.image_url = image_url;
                    a.followers = followers;
                    a.top_tracks = top_tracks;
                    a.albums = albums;
                    a.loading = false;
                }
                cx.rebuild();
            }
        }
        WorkerResponse::ArtistFailed { id, error } => {
            state.library.clear_inflight(&id);
            log::warn!("artist {id} load failed: {error}");
        }
        WorkerResponse::TrackDetails { details } => {
            let track_id = details.track_id.clone();
            let artist = details.artist.clone();
            state.art.insert_track_detail(details);
            // Patch the live player view if it still matches, and push the
            // artist into the reactive text signal — updates the label via
            // the text bind, no rebuild (this is the one that used to land
            // mid-crossfade).
            let mut player = state.player_ui.snapshot.borrow_mut();
            if let Some(p) = player.as_mut()
                && track_id_from_uri(&p.track_id) == Some(track_id.as_str())
            {
                p.artist = artist.clone();
                state.player_ui.artist.set(artist.as_str());
            }
        }
    }
}
