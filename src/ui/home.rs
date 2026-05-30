use std::collections::HashMap;

use frostify_gfx::{
    Align, Computed, ImageHandle, Justify, Len, Overflow, Overlay, Scene, Signal, TextSignal,
    WindowAction,
};

use crate::album_art;
use crate::api::{AlbumRef, HomeData};
use crate::ui::icon::{Icon, IconSet};
use crate::ui::tokens as t;

/// Per-URL reactive cover handles for Home tiles. Keyed by cache_key
/// (trailing CDN hex). `None` until the worker resolves the fetch.
pub type ArtMap = HashMap<String, Signal<Option<ImageHandle>>>;

/// (title, subtitle, resolved cover signal) — pre-baked per tile by
/// [`tile_row`] from the source item + the shared `ArtMap`.
type TileEntry = (String, String, Option<Signal<Option<ImageHandle>>>);

/// Incoming-layer tint: white with alpha rising 0 → 1 as the crossfade
/// advances, so the new cover fades in *over* the outgoing one.
///
/// The outgoing layer underneath stays fully opaque (a plain
/// `[1,1,1,1]` literal, no bind). Crucially this is NOT a symmetric
/// dual fade: if both layers cross-faded (prev `1-t`, curr `t`) their
/// combined coverage dips to ~75% at the midpoint and the dark glass
/// backdrop bleeds through — a murky mid-transition, most visible on the
/// slow backdrop fade. Holding the outgoing layer opaque keeps full
/// coverage throughout, so it's a clean A→B dissolve. Painter order
/// (outgoing declared first) guarantees incoming draws on top.
fn fade_in_alpha(crossfade_t: &Signal<f32>) -> Computed<[f32; 4]> {
    Computed::new((crossfade_t.clone(),), move |(t,)| {
        [1.0, 1.0, 1.0, t.clamp(0.0, 1.0)]
    })
}

/// Fully-opaque white tint for the outgoing (under) crossfade layer.
const OPAQUE_TINT: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

/// Progress-bar quantization steps across its full width. The fill only
/// redraws when `round(progress * STEPS)` changes — ~1 step ≈ 1 px on a
/// ~600 px bar, so the bar redraws a few times a second instead of 60,
/// without any visible stepping (it advances only a few px/sec).
const PROGRESS_STEPS: f32 = 600.0;

/// Reactive height for sidebar header + chip rows. Collapses to 0 in
/// icon-only mode (combined with `.overflow_y(Hidden)` so children get
/// clipped out of the 0-px box instead of painting past).
fn collapsed_height(sidebar_w: &Signal<f32>, expanded: f32) -> Computed<f32> {
    Computed::new((sidebar_w.clone(),), move |(w,)| {
        if w < t::SIDEBAR_COLLAPSE_THRESHOLD {
            0.0
        } else {
            expanded
        }
    })
}

/// Reactive spacer between thumb and text in the playlist row. This is
/// the gap-replacement: the row uses `gap=0` so the spacer can shrink
/// to 0 px when collapsed, putting the thumb against `pad_x` with NO
/// trailing dead space. With a real `gap()` the spacing is fixed at
/// layout time and the thumb ends up left-aligned with the gap still
/// reserving room next to it.
fn collapsed_spacer(sidebar_w: &Signal<f32>) -> Computed<f32> {
    Computed::new((sidebar_w.clone(),), move |(w,)| {
        if w < t::SIDEBAR_COLLAPSE_THRESHOLD {
            0.0
        } else {
            t::SIDEBAR_TEXT_SPACER
        }
    })
}

/// Reactive width for the playlist-row text column. Goes to 0 in
/// collapsed mode; otherwise allocates whatever's left after the row
/// chrome (nested paddings + thumb + spacer). `SIDEBAR_TEXT_CHROME`
/// already encodes the full arithmetic — see its docs in `tokens.rs`.
fn collapsed_text_width(sidebar_w: &Signal<f32>) -> Computed<f32> {
    Computed::new((sidebar_w.clone(),), move |(w,)| {
        if w < t::SIDEBAR_COLLAPSE_THRESHOLD {
            0.0
        } else {
            (w - t::SIDEBAR_TEXT_CHROME).max(t::SP_8)
        }
    })
}

/// A transport intent raised by a player-bar button click. The consumer
/// (main.rs) maps these to optimistic signal flips + worker commands;
/// the UI layer stays ignorant of tokens and the Web API.
#[derive(Debug, Clone, Copy)]
pub enum PlayerAction {
    PlayPause,
    Next,
    Prev,
    ToggleShuffle,
    CycleRepeat,
}

/// All the reactive state the Home view binds to. Bundled so the whole
/// view updates without scene rebuilds — track changes drive these
/// signals and the lib's bind registry pushes the new values into the
/// tree (`crossfade_t`/`panel_crossfade_t` animate via the timeline).
pub struct HomeView<'a> {
    pub backdrop_prev: &'a Signal<Option<ImageHandle>>,
    pub backdrop_curr: &'a Signal<Option<ImageHandle>>,
    /// Slow backdrop + accent crossfade progress.
    pub crossfade_t: &'a Signal<f32>,
    /// Faster foreground cover/thumb crossfade progress.
    pub panel_crossfade_t: &'a Signal<f32>,
    pub accent: &'a Signal<[f32; 4]>,
    pub title: &'a TextSignal,
    pub artist: &'a TextSignal,
    pub is_playing: &'a Signal<bool>,
    pub shuffle: &'a Signal<bool>,
    pub repeat_on: &'a Signal<bool>,
    /// Playback progress 0.0..=1.0 (fraction of track elapsed).
    pub progress: &'a Signal<f32>,
    /// Resizable panel widths (driven by splitters via `width_px_bind`).
    pub sidebar_w: &'a Signal<f32>,
    pub now_playing_w: &'a Signal<f32>,
    /// Called by the splitters after every committed width change.
    /// Wired by the consumer to debounced prefs persistence.
    pub mark_dirty: std::rc::Rc<dyn Fn()>,
    /// Dispatches a transport intent from a player-bar button. Wired by
    /// the consumer to optimistic UI flips + worker playback commands.
    pub on_action: std::rc::Rc<dyn Fn(PlayerAction)>,
    /// The settings modal widget. The view opens it from the gear button
    /// and renders it (the `Overlay` owns the scrim, fade, and dismissal).
    pub settings: &'a Overlay,
    /// Canvas-visibility toggle state, bound by the settings switch.
    pub show_canvas: &'a Signal<bool>,
    /// Persist after the canvas toggle flips.
    pub on_canvas_change: std::rc::Rc<dyn Fn()>,
    /// Clear the stored token and return to Login.
    pub sign_out: std::rc::Rc<dyn Fn()>,
}

pub fn build(s: &mut Scene, icons: &IconSet, home: &HomeData, art: &ArtMap, v: &HomeView) {
    s.col("home_root")
        .fill()
        .rgba(t::BG[0], t::BG[1], t::BG[2], 1.0)
        .child(|root| {
            // Outgoing layer: previous cover, held fully opaque so the
            // incoming layer dissolves over solid coverage (no background
            // bleed at the midpoint — see `fade_in_alpha`). Bound to the
            // signal via `image_bound`, so `promote_backdrop` swaps the
            // handle with no scene rebuild; `None` renders nothing (the
            // first track has no previous cover).
            root.image_bound((), v.backdrop_prev.clone())
                .abs(0.0, 0.0)
                .w(Len::Fill)
                .h(Len::Fill)
                .blur_source()
                .color(OPAQUE_TINT);
            // Incoming layer: current cover, alpha rising 0 → 1. The
            // handle tracks the signal (rebuild-free); the timeline writes
            // new t every 16 ms and `process_binds` snaps both the handle
            // and the alpha into the tree. `blur_source` so its handle/
            // alpha changes (the crossfade) are the *only* thing that
            // re-runs the full-window blur.
            root.image_bound((), v.backdrop_curr.clone())
                .abs(0.0, 0.0)
                .w(Len::Fill)
                .h(Len::Fill)
                .blur_source()
                .color(fade_in_alpha(v.crossfade_t));
            // Frosted-glass overlay: heavy blur + dark tint = the dimmed
            // ambient look. Always present in Home — before any art it
            // just blurs the dark BG (reads the same), and keeping it
            // unconditional means the first cover appears *under* the
            // glass without needing a rebuild to introduce it.
            root.glass(())
                .abs(0.0, 0.0)
                .w(Len::Fill)
                .h(Len::Fill)
                .blur(80.0)
                .rgba(0.0, 0.0, 0.0, 0.25);
            top_bar(root, icons, v.settings.clone());
            root.row(())
                .w(Len::Fill)
                .h(Len::Fill)
                .pad(t::SP_2)
                .gap(t::SP_0)
                .child(|b| {
                    sidebar(b, icons, home, art, v.sidebar_w, v.accent);
                    crate::ui::splitter::splitter(
                        b,
                        crate::ui::splitter::SplitterProps {
                            name: "split_sidebar",
                            width: v.sidebar_w.clone(),
                            side: crate::ui::splitter::PanelSide::Left,
                            min: t::SIDEBAR_MIN,
                            max: t::SIDEBAR_MAX,
                            collapsed: t::SIDEBAR_COLLAPSED,
                            on_change: v.mark_dirty.clone(),
                        },
                    );
                    main_area(b, icons, home, art, v.accent);
                    crate::ui::splitter::splitter(
                        b,
                        crate::ui::splitter::SplitterProps {
                            name: "split_now_playing",
                            width: v.now_playing_w.clone(),
                            side: crate::ui::splitter::PanelSide::Right,
                            min: t::NOW_PLAYING_MIN,
                            max: t::NOW_PLAYING_MAX,
                            collapsed: t::SP_0,
                            on_change: v.mark_dirty.clone(),
                        },
                    );
                    now_playing(b, v);
                });
            player_bar(root, icons, v);
            // Settings modal — rendered last (layers on top). The Overlay
            // owns the scrim, fade, input-blocking and dismissal; we just
            // hand it the panel interior. Skipped entirely when closed.
            v.settings.render(root, t::SCRIM, |panel| {
                crate::ui::settings::panel(
                    panel,
                    icons,
                    crate::ui::settings::SettingsProps {
                        overlay: v.settings.clone(),
                        profile: home.profile.as_ref(),
                        show_canvas: v.show_canvas,
                        accent: v.accent,
                        sign_out: v.sign_out.clone(),
                        on_canvas_change: v.on_canvas_change.clone(),
                    },
                )
            });
        });
}

/// Two stacked album-art layers that crossfade on track change, sized to
/// fill the parent box. Reuses the backdrop's `crossfade_t` + prev/curr
/// handles so panel art dissolves in lockstep with the ambient backdrop
/// instead of snapping. Dim placeholder when neither handle resolves.
/// Both layers are `abs(0,0)` so they overlap — the parent must have a
/// definite size for `Fill` to resolve against.
fn crossfaded_art(
    c: &mut Scene,
    prev: &Signal<Option<ImageHandle>>,
    curr: &Signal<Option<ImageHandle>>,
    crossfade_t: &Signal<f32>,
    radius: f32,
) {
    // Layer 0: dim placeholder shown until any cover resolves (the
    // image layers above render nothing while their signal is None).
    c.rect(())
        .abs(0.0, 0.0)
        .w(Len::Fill)
        .h(Len::Fill)
        .rgba(t::PLACEHOLDER[0], t::PLACEHOLDER[1], t::PLACEHOLDER[2], 1.0)
        .radius(radius);
    // Layer 1: outgoing cover, opaque. Layer 2: incoming, fading in.
    // Both bound to the shared backdrop signals — swap rebuild-free.
    c.image_bound((), prev.clone())
        .abs(0.0, 0.0)
        .w(Len::Fill)
        .h(Len::Fill)
        .radius(radius)
        .color(OPAQUE_TINT);
    c.image_bound((), curr.clone())
        .abs(0.0, 0.0)
        .w(Len::Fill)
        .h(Len::Fill)
        .radius(radius)
        .color(fade_in_alpha(crossfade_t));
}

fn top_bar(s: &mut Scene, icons: &IconSet, settings: Overlay) {
    s.row("topbar")
        .w(Len::Fill)
        .h(Len::Auto)
        .pad_ltrb(t::SP_2, t::SP_2, t::SP_2, t::SP_0)
        .gap(t::SP_2)
        .align(Align::Center)
        .rgba(0.0, 0.0, 0.0, 0.0)
        .window_action(WindowAction::DragMove)
        .child(|t_row| {
            topbar_icon_btn(t_row, icons, Icon::Menu);
            topbar_icon_btn(t_row, icons, Icon::ChevronLeft);
            topbar_icon_btn(t_row, icons, Icon::ChevronRight);

            t_row
                .row(())
                .w(Len::Fill)
                .h_px(t::SEARCH_H)
                .center()
                .child(|c| {
                    c.row(())
                        .w_px(t::SEARCH_W)
                        .h_px(t::SEARCH_H)
                        .pad_xy(t::SP_3_5, t::SP_0)
                        .gap(t::SP_2_5)
                        .align(Align::Center)
                        .rgba(t::PANEL_HI[0], t::PANEL_HI[1], t::PANEL_HI[2], 1.0)
                        .radius(t::R_FULL)
                        .border(1.0, t::BORDER)
                        .child(|s2| {
                            icons.render(s2, Icon::Search, t::ICON_SM, t::TEXT_DIM);
                            s2.text((), "What do you want to play?", 13.0)
                                .color(t::TEXT_DIM);
                        });
                });

            topbar_icon_btn_click(t_row, icons, Icon::Settings, move |ctx| {
                settings.open(ctx.timeline, ctx.now)
            });
            topbar_icon_btn(t_row, icons, Icon::Bell);

            chrome_btn(
                t_row,
                icons,
                Icon::Minimize,
                WindowAction::Minimize,
                t::BTN_HOVER,
                true,
            );
            chrome_btn(
                t_row,
                icons,
                Icon::Maximize,
                WindowAction::ToggleMaximize,
                t::BTN_HOVER,
                false,
            );
            chrome_btn(
                t_row,
                icons,
                Icon::Close,
                WindowAction::Close,
                t::CLOSE_HOVER,
                false,
            );
        });
}

/// Top-bar pill button with a click handler (e.g. the settings gear).
/// The handler receives the full `EventCtx` so it can start a timeline
/// tween (the settings fade) at click time.
fn topbar_icon_btn_click(
    s: &mut Scene,
    icons: &IconSet,
    icon: Icon,
    on_click: impl Fn(&mut frostify_gfx::EventCtx) + 'static,
) {
    s.row(())
        .w_px(t::TOPBAR_BTN)
        .h_px(t::TOPBAR_BTN)
        .rgba(t::PANEL[0], t::PANEL[1], t::PANEL[2], 1.0)
        .hover_color(t::PANEL_HI)
        .radius(t::R_FULL)
        .center()
        .on_click(on_click)
        .child(|c| {
            icons.render(c, icon, t::ICON_MD, t::TEXT);
        });
}

fn topbar_icon_btn(s: &mut Scene, icons: &IconSet, icon: Icon) {
    s.row(())
        .w_px(t::TOPBAR_BTN)
        .h_px(t::TOPBAR_BTN)
        .rgba(t::PANEL[0], t::PANEL[1], t::PANEL[2], 1.0)
        .hover_color(t::PANEL_HI)
        .radius(t::R_FULL)
        .center()
        .child(|c| {
            icons.render(c, icon, t::ICON_MD, t::TEXT);
        });
}

fn chrome_btn(
    s: &mut Scene,
    icons: &IconSet,
    icon: Icon,
    action: WindowAction,
    hover: [f32; 4],
    push_end: bool,
) {
    let mut b = s.row(());
    b.w_px(t::SP_11)
        .h_px(t::SP_8)
        .rgba(0.0, 0.0, 0.0, 0.0)
        .hover_color(hover)
        .radius(t::R_MD)
        .center()
        .window_action(action);
    if push_end {
        b.push_end();
    }
    b.child(|c| {
        icons.render(c, icon, t::ICON_XS, t::TEXT);
    });
}

fn sidebar(
    s: &mut Scene,
    icons: &IconSet,
    home: &HomeData,
    art: &ArtMap,
    w: &Signal<f32>,
    accent: &Signal<[f32; 4]>,
) {
    s.col("sidebar")
        .width_px_bind(w.clone())
        .h(Len::Fill)
        .rgba(t::PANEL[0], t::PANEL[1], t::PANEL[2], 0.75)
        .radius(t::R_LG)
        // Clip horizontally so the fixed-width children (thumb, header
        // text, chips) don't paint past the panel edge when the
        // splitter drags the width below their natural size.
        .overflow_x(Overflow::Hidden)
        .child(|c| {
            // Header row — collapses to 0 height in icon-only mode so
            // the text "Your Library" doesn't truncate.
            c.row(())
                .w(Len::Fill)
                .height_px_bind(collapsed_height(w, t::ROW_H_LG))
                .pad_xy(t::SP_4, t::SP_0)
                .gap(t::SP_2)
                .align(Align::Center)
                .overflow_y(Overflow::Hidden)
                .child(|h| {
                    icons.render(h, Icon::Home, t::ICON_MD, t::TEXT);
                    h.text((), "Your Library", 14.0).color(t::TEXT);
                    h.row(()).push_end().child(|r| {
                        icons.render(r, Icon::Plus, t::ICON_MD, t::TEXT_DIM);
                    });
                });
            // Library filter chips — same collapse behavior.
            c.row(())
                .w(Len::Fill)
                .height_px_bind(collapsed_height(w, t::SP_11))
                .pad_xy(t::SP_3, t::SP_0)
                .gap(t::SP_2)
                .align(Align::Center)
                .overflow_y(Overflow::Hidden)
                .child(|chips| {
                    chip(chips, "Playlists", true, accent);
                    chip(chips, "Artists", false, accent);
                    chip(chips, "Albums", false, accent);
                });
            c.col(())
                .w(Len::Fill)
                .h(Len::Fill)
                .pad_xy(t::SP_1_5, t::SP_1_5)
                .gap(t::SP_1)
                .scroll_y()
                // Auto-hide so the right edge of the collapsed sidebar
                // reads as a clean panel border, not a reserved scroll
                // gutter. Reappears on hover/drag like Spotify.
                .scrollbar(|s| s.auto_hide(true).margin(t::SP_0_5).thickness(t::SP_1))
                .child(|c| {
                    for p in &home.playlists {
                        let sig = p
                            .image_url
                            .as_ref()
                            .and_then(|u| art.get(&album_art::cache_key(u)).cloned());
                        playlist_row(c, &p.name, "Playlist", sig, w);
                    }
                });
        });
}

fn playlist_row(
    s: &mut Scene,
    title: &str,
    subtitle: &str,
    art: Option<Signal<Option<ImageHandle>>>,
    sidebar_w: &Signal<f32>,
) {
    s.row(())
        .w(Len::Fill)
        .h_px(t::SP_16)
        .pad_xy(t::SP_1_5, t::SP_1_5)
        .gap(t::SP_0)
        .align(Align::Center)
        .hover_color(t::HOVER_LIFT_SUBTLE)
        .radius(t::R_MD)
        .child(|r| {
            thumb(r, art, t::THUMB_LG, t::R_SM);
            // Reactive spacer — replaces `gap` so the trailing space
            // vanishes when collapsed. `gap()` is fixed at layout time
            // and would leave dead space next to the thumb otherwise.
            r.rect(())
                .width_px_bind(collapsed_spacer(sidebar_w))
                .h_px(t::SP_PX)
                .rgba(0.0, 0.0, 0.0, 0.0);
            // Text col — width also collapses to 0 so the row's natural
            // size matches `SIDEBAR_COLLAPSED` exactly. Overflow_x clips
            // the still-laid-out glyphs out of the 0-width box.
            r.col(())
                .gap(t::SP_0_5)
                .h(Len::Fill)
                .justify(Justify::Center)
                .width_px_bind(collapsed_text_width(sidebar_w))
                .overflow_x(Overflow::Hidden)
                .child(|m| {
                    m.text((), title, 13.0).color(t::TEXT).max_width_px(240.0);
                    m.text((), subtitle, 11.0).color(t::TEXT_DIM);
                });
        });
}

/// Fixed-size square thumbnail. Renders the resolved cover when the
/// signal carries `Some(handle)` (overlaid on a dim placeholder so the
/// pre-resolve frame doesn't pop). `None` (no signal or unresolved) =
/// placeholder only.
fn thumb(s: &mut Scene, art: Option<Signal<Option<ImageHandle>>>, size: f32, radius: f32) {
    s.col(()).w_px(size).h_px(size).child(|b| {
        // Placeholder backdrop — always present so the tile keeps a
        // visible thumb shape while art is loading.
        b.rect(())
            .abs(0.0, 0.0)
            .w(Len::Fill)
            .h(Len::Fill)
            .rgba(t::PLACEHOLDER[0], t::PLACEHOLDER[1], t::PLACEHOLDER[2], 1.0)
            .radius(radius);
        if let Some(sig) = art {
            b.image_bound((), sig)
                .abs(0.0, 0.0)
                .w(Len::Fill)
                .h(Len::Fill)
                .radius(radius);
        }
    });
}

/// Content filter tabs shown across the top of the main pane.
const FILTERS: &[&str] = &["All", "Music", "Podcasts", "Audiobooks"];

fn main_area(
    s: &mut Scene,
    icons: &IconSet,
    home: &HomeData,
    art: &ArtMap,
    accent: &Signal<[f32; 4]>,
) {
    let greeting = match home.profile.as_ref() {
        Some(p) if !p.display_name.is_empty() => format!("Good evening, {}", p.display_name),
        _ => "Good evening".to_string(),
    };
    let made_for = match home.profile.as_ref() {
        Some(p) if !p.display_name.is_empty() => format!("Made For {}", p.display_name),
        _ => "Made For You".to_string(),
    };
    s.col("main_area")
        .w(Len::Fill)
        .h(Len::Fill)
        .rgba(t::PANEL[0], t::PANEL[1], t::PANEL[2], 0.75)
        .radius(t::R_LG)
        .clip()
        .child(|outer| {
            // Filter chips pinned at the top of the pane.
            outer
                .row(())
                .w(Len::Fill)
                .h(Len::Auto)
                .pad_ltrb(t::SP_6, t::SP_4, t::SP_6, t::SP_0)
                .gap(t::SP_2)
                .align(Align::Center)
                .child(|chips| {
                    for (i, label) in FILTERS.iter().enumerate() {
                        chip(chips, label, i == 0, accent);
                    }
                });
            // Scrolling content body — all sections hit real endpoints.
            outer
                .col(())
                .w(Len::Fill)
                .h(Len::Fill)
                .pad_xy(t::SP_6, t::SP_2)
                .gap(t::SP_5)
                .scroll_y()
                .child(|c| {
                    c.text((), greeting, 26.0)
                        .color(t::TEXT)
                        .max_width_px(520.0);

                    // Spotlit new release (newest album from #1 top artist).
                    if let Some(rel) = home.latest_release.as_ref() {
                        section_header(c, &format!("New release from {}", rel.artist));
                        new_release_card(c, icons, rel, art, accent);
                    }

                    section_header(c, "Recently played");
                    tile_row(c, home.recent.iter().take(5), art, |t| {
                        (t.name.clone(), t.artist.clone(), t.album_image_url.clone())
                    });

                    section_header(c, "Your top artists");
                    tile_row(c, home.top_artists.iter().take(5), art, |a| {
                        (a.name.clone(), "Artist".to_string(), a.image_url.clone())
                    });

                    section_header(c, "Your top tracks");
                    tile_row(c, home.top_tracks.iter().take(5), art, |t| {
                        (t.name.clone(), t.artist.clone(), t.album_image_url.clone())
                    });

                    section_header(c, &made_for);
                    tile_row(c, home.playlists.iter().take(5), art, |p| {
                        (p.name.clone(), "Playlist".to_string(), p.image_url.clone())
                    });
                });
        });
}

/// Horizontal strip of up to 5 tiles.
fn tile_row<T>(
    s: &mut Scene,
    items: impl Iterator<Item = T>,
    art: &ArtMap,
    label: impl Fn(&T) -> (String, String, Option<String>),
) {
    let entries: Vec<TileEntry> = items
        .map(|t| {
            let (title, sub, url) = label(&t);
            let sig = url
                .as_ref()
                .and_then(|u| art.get(&album_art::cache_key(u)).cloned());
            (title, sub, sig)
        })
        .collect();
    s.row(())
        .w(Len::Fill)
        .h_px(t::SP_56)
        .gap(t::SP_3_5)
        .scroll_x()
        .child(|g| {
            if entries.is_empty() {
                for _ in 0..8 {
                    tile(g, "—", "", None);
                }
            } else {
                for (title, sub, art) in &entries {
                    tile(g, title, sub, art.clone());
                }
            }
        });
}

/// Pill-shaped content filter. Selected chip uses the live accent
/// colour (derived from the current album art) on a dark text foreground
/// — pulls the album palette into the chrome the same way the play pill
/// does. Unselected chips sit on the panel-highlight colour.
fn chip(s: &mut Scene, label: &str, selected: bool, accent: &Signal<[f32; 4]>) {
    let mut row = s.row(());
    row.h_px(t::CHIP_H)
        .pad_xy(t::SP_3_5, t::SP_0)
        .center()
        .radius(t::R_FULL);
    if selected {
        row.color(accent.clone()).hover_opacity(0.9).child(|c| {
            c.text((), label, 13.0).color(t::BG);
        });
    } else {
        row.color(t::PANEL_HI).hover_opacity(0.8).child(|c| {
            c.text((), label, 13.0).color(t::TEXT);
        });
    }
}

/// Section title plus a dim "Show all" affordance on the right.
fn section_header(s: &mut Scene, title: &str) {
    s.row(())
        .w(Len::Fill)
        .h_px(t::SP_7)
        .align(Align::Center)
        .child(|h| {
            h.text((), title, 18.0).color(t::TEXT);
            h.row(()).push_end().child(|r| {
                r.text((), "Show all", 12.0).color(t::TEXT_DIM);
            });
        });
}

fn tile(s: &mut Scene, title: &str, sub: &str, art: Option<Signal<Option<ImageHandle>>>) {
    s.col(())
        .w_px(t::TILE_W)
        .h(Len::Fill)
        .pad(t::SP_2_5)
        .gap(t::SP_2)
        .rgba(t::PANEL_HI[0], t::PANEL_HI[1], t::PANEL_HI[2], 1.0)
        .hover_color(t::HOVER_LIFT)
        .radius(t::R_LG)
        .child(|card| {
            card.col(())
                .w_px(t::TILE_THUMB)
                .h_px(t::TILE_THUMB)
                .child(|b| {
                    b.rect(())
                        .abs(0.0, 0.0)
                        .w(Len::Fill)
                        .h(Len::Fill)
                        .rgba(t::PLACEHOLDER[0], t::PLACEHOLDER[1], t::PLACEHOLDER[2], 1.0)
                        .radius(t::R_MD);
                    if let Some(sig) = art {
                        b.image_bound((), sig)
                            .abs(0.0, 0.0)
                            .w(Len::Fill)
                            .h(Len::Fill)
                            .radius(t::R_MD);
                    }
                });
            card.text((), title, 13.0)
                .color(t::TEXT)
                .max_width_px(t::TILE_TEXT_MAX);
            card.text((), sub, 11.0)
                .color(t::TEXT_DIM)
                .max_width_px(t::TILE_TEXT_MAX);
        });
}

/// Wide spotlight card: large art + title/artist + an accent play pill.
fn new_release_card(
    s: &mut Scene,
    icons: &IconSet,
    album: &AlbumRef,
    art: &ArtMap,
    accent: &Signal<[f32; 4]>,
) {
    let art_sig = album
        .image_url
        .as_ref()
        .and_then(|u| art.get(&album_art::cache_key(u)).cloned());
    s.row(())
        .w(Len::Fill)
        .h_px(t::SP_32)
        .pad(t::SP_3_5)
        .gap(t::SP_4)
        .align(Align::Center)
        .rgba(t::PANEL_HI[0], t::PANEL_HI[1], t::PANEL_HI[2], 1.0)
        .hover_color(t::HOVER_LIFT)
        .radius(t::R_2XL)
        .child(|c| {
            thumb(c, art_sig, t::THUMB_XL, t::R_MD);
            c.col(())
                .h(Len::Fill)
                .gap(t::SP_1)
                .justify(Justify::Center)
                .child(|m| {
                    m.text((), &album.release_date, 11.0).color(t::TEXT_DIM);
                    m.text((), &album.name, 20.0)
                        .color(t::TEXT)
                        .max_width_px(360.0);
                    m.text((), &album.artist, 12.0)
                        .color(t::TEXT_DIM)
                        .max_width_px(360.0);
                });
            c.row(())
                .push_end()
                .w_px(t::BTN_H_LG)
                .h_px(t::BTN_H_LG)
                .center()
                .color(accent.clone())
                .hover_opacity(0.85)
                .radius(t::R_FULL)
                .child(|p| {
                    icons.render(p, Icon::Play, t::ICON_MD, [0.0, 0.0, 0.0, 1.0]);
                });
        });
}

fn now_playing(s: &mut Scene, v: &HomeView) {
    s.col("now_playing")
        .width_px_bind(v.now_playing_w.clone())
        .h(Len::Fill)
        .pad(t::SP_4)
        .gap(t::SP_3)
        .rgba(t::PANEL[0], t::PANEL[1], t::PANEL[2], 0.75)
        .radius(t::R_LG)
        .overflow_x(Overflow::Hidden)
        .child(|c| {
            c.text((), "Now playing", 16.0).color(t::TEXT);
            // Square art container — `.square()` is an aspect-ratio
            // constraint enforced by the layout: width is whatever flex
            // resolves (here `Fill` against the panel's inner width),
            // height is rewritten to match. Stays square at every
            // splitter position, no per-axis Computed needed.
            c.col(()).w(Len::Fill).square().child(|b| {
                crossfaded_art(
                    b,
                    v.backdrop_prev,
                    v.backdrop_curr,
                    v.panel_crossfade_t,
                    t::R_LG,
                )
            });
            c.text_bound((), v.title.clone(), 14.0)
                .color(t::TEXT)
                .max_width_px(300.0);
            c.text_bound((), v.artist.clone(), 12.0)
                .color(t::TEXT_DIM)
                .max_width_px(300.0);
        });
}

fn player_bar(s: &mut Scene, icons: &IconSet, v: &HomeView) {
    s.row("playerbar")
        .w(Len::Fill)
        .h_px(t::PLAYER_H)
        .pad_xy(t::SP_4, t::SP_2)
        .gap(t::SP_3)
        .align(Align::Center)
        .rgba(0.0, 0.0, 0.0, 0.45)
        .child(|c| {
            c.row(())
                .w_px(t::SP_80 - t::SP_5)
                .h(Len::Fill)
                .gap(t::SP_2_5)
                .align(Align::Center)
                .child(|l| {
                    l.col(()).w_px(t::SP_14).h_px(t::SP_14).child(|b| {
                        crossfaded_art(
                            b,
                            v.backdrop_prev,
                            v.backdrop_curr,
                            v.panel_crossfade_t,
                            t::R_SM,
                        )
                    });
                    l.col(())
                        .gap(t::SP_0_5)
                        .h(Len::Fill)
                        .justify(Justify::Center)
                        .child(|m| {
                            m.text_bound((), v.title.clone(), 13.0)
                                .color(t::TEXT)
                                .max_width_px(180.0);
                            m.text_bound((), v.artist.clone(), 11.0)
                                .color(t::TEXT_DIM)
                                .max_width_px(180.0);
                        });
                    l.row(()).push_end().center().child(|h| {
                        icons.render(h, Icon::Heart, t::ICON_MD, t::TEXT_DIM);
                    });
                });
            c.col(())
                .w(Len::Fill)
                .h(Len::Fill)
                .align(Align::Center)
                .justify(Justify::Center)
                .gap(t::SP_1_5)
                .child(|ct| {
                    ct.row(())
                        .gap(t::SP_4 + t::SP_0_5)
                        .align(Align::Center)
                        .center()
                        .child(|tr| {
                            let shuffle_tint = Computed::new(
                                (v.shuffle.clone(), v.accent.clone()),
                                |(on, acc)| {
                                    if on { acc } else { t::TEXT_DIM }
                                },
                            );
                            transport_btn(tr, icons, Icon::Shuffle, t::ICON_MD, shuffle_tint, {
                                let act = v.on_action.clone();
                                move || act(PlayerAction::ToggleShuffle)
                            });
                            transport_btn(tr, icons, Icon::SkipBack, t::ICON_LG, t::TEXT, {
                                let act = v.on_action.clone();
                                move || act(PlayerAction::Prev)
                            });
                            let play_h = icons.get(Icon::Play);
                            let pause_h = icons.get(Icon::Pause);
                            let play_glyph =
                                Computed::new((v.is_playing.clone(),), move |(playing,)| {
                                    Some(if playing { pause_h } else { play_h })
                                });
                            let play_act = v.on_action.clone();
                            tr.row(())
                                .w_px(t::SP_9)
                                .h_px(t::SP_9)
                                .color(v.accent.clone())
                                .hover_opacity(0.85)
                                .radius(t::R_FULL)
                                .center()
                                .on_click(move |_| play_act(PlayerAction::PlayPause))
                                .child(|p| {
                                    p.image_bound((), play_glyph)
                                        .w_px(t::SP_4)
                                        .h_px(t::SP_4)
                                        .color([0.0, 0.0, 0.0, 1.0]);
                                });
                            transport_btn(tr, icons, Icon::SkipForward, t::ICON_LG, t::TEXT, {
                                let act = v.on_action.clone();
                                move || act(PlayerAction::Next)
                            });
                            let repeat_tint = Computed::new(
                                (v.repeat_on.clone(), v.accent.clone()),
                                |(on, acc)| {
                                    if on { acc } else { t::TEXT_DIM }
                                },
                            );
                            transport_btn(tr, icons, Icon::Repeat, t::ICON_MD, repeat_tint, {
                                let act = v.on_action.clone();
                                move || act(PlayerAction::CycleRepeat)
                            });
                        });
                    ct.row(())
                        .w(Len::Fill)
                        .h_px(t::SP_1)
                        .pad_xy(t::SP_10, t::SP_0)
                        .child(|sl| {
                            sl.rect(())
                                .w(Len::Fill)
                                .h_px(t::SP_1)
                                .rgba(1.0, 1.0, 1.0, 0.10)
                                .radius(t::R_SM / 2.0)
                                .child(|bar| {
                                    let progress_q = Computed::new((v.progress.clone(),), |(p,)| {
                                        (p * PROGRESS_STEPS).round() / PROGRESS_STEPS
                                    });
                                    bar.rect(())
                                        .width_pct(progress_q)
                                        .h_px(t::SP_1)
                                        .rgba(t::TEXT[0], t::TEXT[1], t::TEXT[2], 1.0)
                                        .radius(t::R_SM / 2.0);
                                });
                        });
                });
            c.row(())
                .w_px(t::SP_80 - t::SP_5)
                .h(Len::Fill)
                .gap(t::SP_3)
                .align(Align::Center)
                .justify(Justify::End)
                .child(|r| {
                    lossless_badge(r);
                    icon_only(r, icons, Icon::Queue);
                    icon_only(r, icons, Icon::Devices);
                    icons.render(r, Icon::Volume, t::ICON_MD, t::TEXT);
                    r.rect(())
                        .w_px(t::SP_24)
                        .h_px(t::SP_1)
                        .rgba(1.0, 1.0, 1.0, 0.10)
                        .radius(t::R_SM / 2.0);
                });
        });
}

/// Small "LOSSLESS" capability pill shown in the player bar.
fn lossless_badge(s: &mut Scene) {
    s.row(())
        .h_px(t::SP_5)
        .pad_xy(t::SP_2, t::SP_0)
        .center()
        .rgba(1.0, 1.0, 1.0, 0.10)
        .radius(t::R_SM)
        .border(1.0, t::BORDER)
        .child(|b| {
            b.text((), "LOSSLESS", 10.0).color(t::TEXT_DIM);
        });
}

/// Bare icon button (no background pill) for the player-bar utilities.
fn icon_only(s: &mut Scene, icons: &IconSet, icon: Icon) {
    s.row(()).w_px(t::SP_7).h_px(t::SP_7).center().child(|c| {
        icons.render(c, icon, t::ICON_MD, t::TEXT_DIM);
    });
}

/// Transport icon button. `tint` accepts a static colour, a `Signal`, or
/// a `Computed` (via `Into<Bind>`), so active-toggle states route a
/// reactive tint and update without a rebuild.
fn transport_btn(
    s: &mut Scene,
    icons: &IconSet,
    icon: Icon,
    size: f32,
    tint: impl Into<frostify_gfx::Bind<[f32; 4]>>,
    on_click: impl Fn() + 'static,
) {
    s.row(())
        .w_px(t::SP_8)
        .h_px(t::SP_8)
        .center()
        .hover_opacity(0.7)
        .on_click(move |_| on_click())
        .child(|c| {
            icons.render(c, icon, size, tint);
        });
}
