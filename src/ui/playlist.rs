//! Playlist (and Liked Songs) detail page rendered into the centre pane.
//!
//! Loads **progressively**: the header + scrollbar appear from metadata
//! before any track lands, the first page mounts the virtualised list,
//! and later pages stream into a shared live buffer that the `lazy_list`
//! reads on scroll — no blocking "loading all 989 songs" screen, no
//! full-list rebuilds while paging. Rows past the loaded count render a
//! lightweight skeleton until their page arrives.
//!
//! The buffer ([`RowBuf`]) is owned by `AppState` and mutated on the UI
//! thread as worker pages arrive; the render closure here just indexes
//! it. Covers are baked into each [`PlaylistRow`] as a reactive `Signal`
//! (resolved/dispatched when the row is appended), so a cover arrival
//! repaints just that thumb with no rebuild.

use std::cell::RefCell;
use std::rc::Rc;

use frostify_gfx::{Align, ImageHandle, Justify, Len, Overflow, Scene, Signal};

use crate::api::PlayTarget;
use crate::ui::MainNav;
use crate::ui::home::{NavFn, PlayFn};
use crate::ui::icon::{Icon, IconSet};
use crate::ui::tokens as t;

/// Track-row height. Thumb (40) + breathing room.
const ROW_H: f32 = t::SP_14;

/// Spotify's `PUT /me/player/play` caps the inline `uris` array. For the
/// context-less Liked Songs we send a window from the clicked track so
/// playback begins there and queues the following tracks.
const URIS_WINDOW: usize = 100;

/// A fully-baked track row — built when the track is appended to the
/// buffer (so the cover `Signal` is resolved off the shared art map on
/// the UI thread). The render closure just reads these.
pub struct PlaylistRow {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: String,
    pub uri: String,
    /// Reactive cover handle (bound via `image_bound`). `None` if the
    /// track has no cover URL; the inner `Signal` stays `None` until the
    /// cover is lazily fetched (see `cover_url` + `request_cover`).
    pub art: Option<Signal<Option<ImageHandle>>>,
    /// Source URL, kept so the cover can be fetched **lazily** the first
    /// time the row scrolls into view — avoids dispatching thousands of
    /// downloads up front for a long playlist.
    pub cover_url: Option<String>,
}

/// Request a track cover be fetched (called when a row materializes).
/// Idempotent + gated on the consumer side.
pub type CoverFn = Rc<dyn Fn(String)>;

/// Shared, growable track buffer for the open playlist. `AppState` owns
/// it and appends streamed pages; the `lazy_list` render closure holds a
/// clone and reads it per visible row.
pub type RowBuf = Rc<RefCell<Vec<PlaylistRow>>>;

/// Everything the view needs for one render. Built per rebuild from
/// `AppState.open_playlist`; cheap (small metadata clones + Rc handles).
pub struct PlaylistViewData {
    pub name: String,
    pub owner: String,
    /// Reported track total — drives the list length + scrollbar even
    /// before every page has streamed in.
    pub total: u32,
    pub liked: bool,
    /// Metadata not yet arrived (header shows the sidebar-known name, the
    /// list shows skeletons).
    pub loading: bool,
    pub cover: Option<Signal<Option<ImageHandle>>>,
    /// `spotify:playlist:…` for real playlists; `None` for Liked Songs.
    pub context_uri: Option<String>,
    pub rows: RowBuf,
    /// Fetch a row's cover the first time it scrolls into view.
    pub request_cover: CoverFn,
}

/// Render the centre-pane content for the open playlist. Children are
/// added to `s` (the caller's slide/fade transition wrapper).
pub fn view(
    s: &mut Scene,
    icons: &Rc<IconSet>,
    data: &PlaylistViewData,
    accent: &Signal<[f32; 4]>,
    on_play: PlayFn,
    on_navigate: NavFn,
) {
    header(s, icons, data, accent, &on_play, &on_navigate);

    // Column header strip + divider.
    s.row(())
        .w(Len::Fill)
        .h_px(t::SP_8)
        .pad_ltrb(t::SP_6, t::SP_0, t::SP_6, t::SP_0)
        .gap(t::SP_3)
        .align(Align::Center)
        .child(|h| {
            h.row(()).w_px(t::SP_7).center().child(|c| {
                c.text((), "#", 12.0).color(t::TEXT_DIM);
            });
            h.text((), "Title", 12.0).color(t::TEXT_DIM);
            h.row(())
                .push_end()
                .w_px(t::SP_12)
                .justify(Justify::End)
                .child(|c| {
                    c.text((), "Time", 12.0).color(t::TEXT_DIM);
                });
        });
    s.rect(())
        .w(Len::Fill)
        .h_px(t::SP_PX)
        .pad_xy(t::SP_6, t::SP_0)
        .rgba(1.0, 1.0, 1.0, 0.06);

    let loaded = data.rows.borrow().len() as u32;
    let count = data.total.max(loaded);
    if count == 0 {
        // No length known yet (or genuinely empty). Show skeletons while
        // loading so it reads as "filling in", never a blank/spinner wall.
        if data.loading {
            skeleton_list(s);
        } else {
            s.col(()).w(Len::Fill).h(Len::Fill).center().child(|c| {
                c.text((), "No songs here yet", 14.0).color(t::TEXT_DIM);
            });
        }
        return;
    }

    // Virtualised list of `count` rows. Rows past `loaded` render a
    // skeleton until their page streams into the buffer.
    let rows = data.rows.clone();
    let ctx = data.context_uri.clone();
    let request_cover = data.request_cover.clone();
    s.lazy_list("pl_tracks", count, ROW_H, move |sc, i| {
        let has = i < rows.borrow().len() as u32;
        if has {
            let buf = rows.borrow();
            let r = &buf[i as usize];
            track_row(sc, r, i, &on_play, &ctx, &rows, &request_cover);
        } else {
            skeleton_row(sc, i);
        }
    })
    .w(Len::Fill)
    .h(Len::Fill)
    .pad_ltrb(t::SP_3, t::SP_0, t::SP_3, t::SP_4)
    // Compositor scroll layer (frostify-gfx P3 2b): the 989-row track list
    // rasters its materialized window once into a tall texture; scrolling
    // moves the composite window instead of re-rastering every row. Lazy +
    // glass-free, so it takes the windowed-lazy path.
    .layer()
    .scrollbar(|sb| sb.auto_hide(true).margin(t::SP_0_5).thickness(t::SP_1));
}

/// Build the playback target for the track at `index`. Real playlists
/// play their context at the offset; Liked Songs (no context) sends a
/// capped window of URIs from the buffer starting at the clicked track.
fn make_target(context_uri: &Option<String>, rows: &RowBuf, index: u32) -> PlayTarget {
    match context_uri {
        Some(uri) => PlayTarget::Context {
            context_uri: uri.clone(),
            offset: index,
        },
        None => {
            let buf = rows.borrow();
            let uris = buf
                .iter()
                .skip(index as usize)
                .take(URIS_WINDOW)
                .map(|r| r.uri.clone())
                .collect();
            PlayTarget::Uris { uris, offset: 0 }
        }
    }
}

fn header(
    s: &mut Scene,
    icons: &Rc<IconSet>,
    data: &PlaylistViewData,
    accent: &Signal<[f32; 4]>,
    on_play: &PlayFn,
    on_navigate: &NavFn,
) {
    // Top bar: back chevron.
    s.row(())
        .w(Len::Fill)
        .h_px(t::SP_10)
        .pad_ltrb(t::SP_4, t::SP_3, t::SP_4, t::SP_0)
        .align(Align::Center)
        .child(|r| {
            let nav = on_navigate.clone();
            r.row(())
                .w_px(t::TOPBAR_BTN)
                .h_px(t::TOPBAR_BTN)
                .rgba(0.0, 0.0, 0.0, 0.30)
                .hover_color(t::PANEL_HI)
                .radius(t::R_FULL)
                .center()
                .on_click(move |ctx| nav(ctx, MainNav::Home))
                .child(|c| {
                    icons.render(c, Icon::ChevronLeft, t::ICON_MD, t::TEXT);
                });
        });

    // Hero row: big cover + title block.
    s.row(())
        .w(Len::Fill)
        .h_px(t::SP_44)
        .pad_ltrb(t::SP_6, t::SP_2, t::SP_6, t::SP_2)
        .gap(t::SP_5)
        .align(Align::End)
        .child(|hero| {
            cover_art(hero, icons, data.cover.clone(), data.liked);
            hero.col(())
                .h(Len::Fill)
                .gap(t::SP_2)
                .justify(Justify::End)
                .child(|m| {
                    m.text((), "Playlist", 12.0).color(t::TEXT_DIM);
                    m.text((), &data.name, 32.0).color(t::TEXT).max_width_px(520.0);
                    m.row(()).gap(t::SP_1_5).align(Align::Center).child(|sub| {
                        if !data.owner.is_empty() {
                            sub.text((), &data.owner, 12.0).color(t::TEXT);
                            sub.text((), "•", 12.0).color(t::TEXT_DIM);
                        }
                        sub.text((), count_label(data), 12.0).color(t::TEXT_DIM);
                    });
                });
        });

    // Action row: big Play pill.
    let has_tracks = data.total > 0 || !data.rows.borrow().is_empty();
    let on_play = on_play.clone();
    let rows = data.rows.clone();
    let ctx = data.context_uri.clone();
    s.row(())
        .w(Len::Fill)
        .h_px(t::SP_16)
        .pad_xy(t::SP_6, t::SP_0)
        .gap(t::SP_4)
        .align(Align::Center)
        .child(|a| {
            let mut pill = a.row(());
            pill.w_px(t::SP_14)
                .h_px(t::SP_14)
                .center()
                .color(accent.clone())
                .radius(t::R_FULL);
            if has_tracks {
                pill.hover_opacity(0.85).on_click(move |_| {
                    on_play(make_target(&ctx, &rows, 0));
                });
            } else {
                pill.opacity(0.4);
            }
            pill.child(|p| {
                icons.render(p, Icon::Play, t::ICON_LG, [0.0, 0.0, 0.0, 1.0]);
            });
        });
}

/// Header count label — "Loading…" until tracks land, then "N songs".
fn count_label(data: &PlaylistViewData) -> String {
    if data.total == 0 {
        if data.loading {
            "Loading…".to_string()
        } else {
            "0 songs".to_string()
        }
    } else if data.total == 1 {
        "1 song".to_string()
    } else {
        format!("{} songs", data.total)
    }
}

/// Square cover. Liked Songs has no image — render the signature
/// purple-ish tile with a heart instead.
fn cover_art(
    s: &mut Scene,
    icons: &Rc<IconSet>,
    art: Option<Signal<Option<ImageHandle>>>,
    liked: bool,
) {
    s.col(()).w_px(t::THUMB_2XL).h_px(t::THUMB_2XL).child(|b| {
        if liked {
            b.rect(())
                .abs(0.0, 0.0)
                .w(Len::Fill)
                .h(Len::Fill)
                .rgba(0.36, 0.20, 0.78, 1.0)
                .radius(t::R_LG);
            b.row(())
                .abs(0.0, 0.0)
                .w(Len::Fill)
                .h(Len::Fill)
                .center()
                .child(|c| {
                    icons.render(c, Icon::Heart, t::ICON_XL, t::TEXT);
                });
            return;
        }
        b.rect(())
            .abs(0.0, 0.0)
            .w(Len::Fill)
            .h(Len::Fill)
            .rgba(t::PLACEHOLDER[0], t::PLACEHOLDER[1], t::PLACEHOLDER[2], 1.0)
            .radius(t::R_LG);
        if let Some(sig) = art {
            b.image_bound((), sig)
                .abs(0.0, 0.0)
                .w(Len::Fill)
                .h(Len::Fill)
                .radius(t::R_LG);
        }
    });
}

fn track_row(
    s: &mut Scene,
    r: &PlaylistRow,
    index: u32,
    on_play: &PlayFn,
    context_uri: &Option<String>,
    rows: &RowBuf,
    request_cover: &CoverFn,
) {
    // Lazily fetch this row's cover the first time it materializes (and
    // isn't resolved yet). The consumer gates on inflight/resolved, so
    // repeated materializes are cheap no-ops.
    if let Some(url) = &r.cover_url
        && r.art.as_ref().map(|s| s.get().is_none()).unwrap_or(false)
    {
        request_cover(url.clone());
    }
    let on_play = on_play.clone();
    let rows = rows.clone();
    let ctx = context_uri.clone();
    s.row(())
        .w(Len::Fill)
        .h_px(ROW_H)
        .pad_xy(t::SP_3, t::SP_1)
        .gap(t::SP_3)
        .align(Align::Center)
        .radius(t::R_MD)
        .hover_color(t::HOVER_LIFT_SUBTLE)
        .on_click(move |_| on_play(make_target(&ctx, &rows, index)))
        .child(|row| {
            row.row(()).w_px(t::SP_7).center().child(|c| {
                c.text((), format!("{}", index + 1), 13.0).color(t::TEXT_DIM);
            });
            // Thumb.
            row.col(()).w_px(t::THUMB_SM).h_px(t::THUMB_SM).child(|b| {
                b.rect(())
                    .abs(0.0, 0.0)
                    .w(Len::Fill)
                    .h(Len::Fill)
                    .rgba(t::PLACEHOLDER[0], t::PLACEHOLDER[1], t::PLACEHOLDER[2], 1.0)
                    .radius(t::R_SM);
                if let Some(sig) = r.art.clone() {
                    b.image_bound((), sig)
                        .abs(0.0, 0.0)
                        .w(Len::Fill)
                        .h(Len::Fill)
                        .radius(t::R_SM);
                }
            });
            // Title + artist.
            row.col(())
                .w(Len::Fill)
                .gap(t::SP_0_5)
                .h(Len::Fill)
                .justify(Justify::Center)
                .overflow_x(Overflow::Hidden)
                .child(|m| {
                    m.text((), &r.title, 14.0).color(t::TEXT).max_width_px(360.0);
                    m.text((), &r.artist, 12.0)
                        .color(t::TEXT_DIM)
                        .max_width_px(360.0);
                });
            // Album.
            row.col(())
                .w_px(t::SP_48)
                .h(Len::Fill)
                .justify(Justify::Center)
                .overflow_x(Overflow::Hidden)
                .child(|m| {
                    m.text((), &r.album, 12.0)
                        .color(t::TEXT_DIM)
                        .max_width_px(t::SP_48);
                });
            // Duration.
            row.row(()).w_px(t::SP_12).justify(Justify::End).child(|c| {
                c.text((), &r.duration, 12.0).color(t::TEXT_DIM);
            });
        });
}

/// Placeholder for a not-yet-streamed row — index number + grey bars.
fn skeleton_row(s: &mut Scene, index: u32) {
    s.row(())
        .w(Len::Fill)
        .h_px(ROW_H)
        .pad_xy(t::SP_3, t::SP_1)
        .gap(t::SP_3)
        .align(Align::Center)
        .child(|row| {
            row.row(()).w_px(t::SP_7).center().child(|c| {
                c.text((), format!("{}", index + 1), 13.0).color(t::TEXT_DIM);
            });
            row.rect(())
                .w_px(t::THUMB_SM)
                .h_px(t::THUMB_SM)
                .rgba(t::PLACEHOLDER[0], t::PLACEHOLDER[1], t::PLACEHOLDER[2], 0.6)
                .radius(t::R_SM);
            row.col(())
                .w(Len::Fill)
                .gap(t::SP_1_5)
                .justify(Justify::Center)
                .h(Len::Fill)
                .child(|m| {
                    m.rect(())
                        .w_px(t::SP_40)
                        .h_px(t::SP_2)
                        .rgba(1.0, 1.0, 1.0, 0.08)
                        .radius(t::R_SM);
                    m.rect(())
                        .w_px(t::SP_24)
                        .h_px(t::SP_2)
                        .rgba(1.0, 1.0, 1.0, 0.05)
                        .radius(t::R_SM);
                });
        });
}

/// A short column of skeleton rows shown before the first page lands.
fn skeleton_list(s: &mut Scene) {
    s.col(())
        .w(Len::Fill)
        .h(Len::Fill)
        .pad_ltrb(t::SP_3, t::SP_0, t::SP_3, t::SP_4)
        .child(|c| {
            for i in 0..10 {
                skeleton_row(c, i);
            }
        });
}

/// `ms` → `m:ss`.
pub fn fmt_duration(ms: u64) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}
