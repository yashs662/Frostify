use frostify_gfx::{
    Align, Computed, ImageHandle, Justify, Len, Scene, Signal, TextSignal, WindowAction,
};

use crate::api::HomeData;
use crate::ui::icon::{Icon, IconSet};
use crate::ui::theme;

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
}

pub fn build(s: &mut Scene, icons: &IconSet, home: &HomeData, v: &HomeView) {
    s.col("home_root")
        .fill()
        .rgba(theme::BG[0], theme::BG[1], theme::BG[2], 1.0)
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
            top_bar(root, icons);
            root.row(())
                .w(Len::Fill)
                .h(Len::Fill)
                .pad(8.0)
                .gap(8.0)
                .child(|b| {
                    sidebar(b, icons, home);
                    main_area(b, home);
                    now_playing(b, v);
                });
            player_bar(root, icons, v);
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
        .rgba(0.20, 0.20, 0.24, 1.0)
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

fn top_bar(s: &mut Scene, icons: &IconSet) {
    s.row("topbar")
        .w(Len::Fill)
        .h_px(56.0)
        .pad_xy(12.0, 0.0)
        .gap(8.0)
        .align(Align::Center)
        .rgba(0.0, 0.0, 0.0, 0.0)
        .window_action(WindowAction::DragMove)
        .child(|t| {
            icon_btn(t, icons, Icon::Menu);
            icon_btn(t, icons, Icon::ChevronLeft);
            icon_btn(t, icons, Icon::ChevronRight);

            t.row(())
                .w(Len::Fill)
                .h_px(40.0)
                .center()
                .child(|c| {
                    c.row(())
                        .w_px(440.0)
                        .h_px(40.0)
                        .pad_xy(14.0, 0.0)
                        .gap(10.0)
                        .align(Align::Center)
                        .rgba(theme::PANEL_HI[0], theme::PANEL_HI[1], theme::PANEL_HI[2], 1.0)
                        .radius(20.0)
                        .border(1.0, theme::BORDER)
                        .child(|s2| {
                            icons.render(s2, Icon::Search, 16.0, theme::TEXT_DIM);
                            s2.text((), "What do you want to play?", 13.0)
                                .color(theme::TEXT_DIM);
                        });
                });

            icon_btn(t, icons, Icon::Settings);
            icon_btn(t, icons, Icon::Bell);

            chrome_btn(t, icons, Icon::Minimize, WindowAction::Minimize, [1.0, 1.0, 1.0, 0.08], true);
            chrome_btn(t, icons, Icon::Maximize, WindowAction::ToggleMaximize, [1.0, 1.0, 1.0, 0.08], false);
            chrome_btn(t, icons, Icon::Close, WindowAction::Close, theme::CLOSE_HOVER, false);
        });
}

fn icon_btn(s: &mut Scene, icons: &IconSet, icon: Icon) {
    s.row(())
        .w_px(36.0)
        .h_px(36.0)
        .rgba(theme::PANEL[0], theme::PANEL[1], theme::PANEL[2], 1.0)
        .hover_color(theme::PANEL_HI)
        .radius(18.0)
        .center()
        .child(|c| {
            icons.render(c, icon, 18.0, theme::TEXT);
        });
}

fn chrome_btn(s: &mut Scene, icons: &IconSet, icon: Icon, action: WindowAction, hover: [f32; 4], push_end: bool) {
    let mut b = s.row(());
    b.w_px(44.0)
        .h_px(32.0)
        .rgba(0.0, 0.0, 0.0, 0.0)
        .hover_color(hover)
        .radius(6.0)
        .center()
        .window_action(action);
    if push_end {
        b.push_end();
    }
    b.child(|c| {
        icons.render(c, icon, 14.0, theme::TEXT);
    });
}

fn sidebar(s: &mut Scene, icons: &IconSet, home: &HomeData) {
    s.col("sidebar")
        .w_px(320.0)
        .h(Len::Fill)
        .rgba(theme::PANEL[0], theme::PANEL[1], theme::PANEL[2], 0.75)
        .radius(8.0)
        .child(|c| {
            c.row(())
                .w(Len::Fill)
                .h_px(48.0)
                .pad_xy(16.0, 0.0)
                .gap(8.0)
                .align(Align::Center)
                .child(|h| {
                    icons.render(h, Icon::Home, 18.0, theme::TEXT);
                    h.text((), "Your Library", 14.0).color(theme::TEXT);
                    h.row(()).push_end().child(|r| {
                        icons.render(r, Icon::Plus, 18.0, theme::TEXT_DIM);
                    });
                });
            c.col(())
                .w(Len::Fill)
                .h(Len::Fill)
                .pad(8.0)
                .gap(6.0)
                .scroll_y()
                .child(|c| {
                    if home.playlists.is_empty() {
                        for i in 0..8 {
                            playlist_row(c, &format!("Playlist {}", i + 1), "Playlist");
                        }
                    } else {
                        for p in &home.playlists {
                            playlist_row(c, &p.name, "Playlist");
                        }
                    }
                });
        });
}

fn playlist_row(s: &mut Scene, title: &str, subtitle: &str) {
    s.row(())
        .w(Len::Fill)
        .h_px(58.0)
        .pad(6.0)
        .gap(10.0)
        .align(Align::Center)
        .hover_color([1.0, 1.0, 1.0, 0.04])
        .radius(6.0)
        .child(|r| {
            r.rect(())
                .w_px(46.0)
                .h_px(46.0)
                .rgba(0.25, 0.25, 0.30, 1.0)
                .radius(4.0);
            r.col(())
                .gap(2.0)
                .h(Len::Fill)
                .justify(Justify::Center)
                .child(|m| {
                    m.text((), title, 13.0)
                        .color(theme::TEXT)
                        .max_width_px(240.0);
                    m.text((), subtitle, 11.0).color(theme::TEXT_DIM);
                });
        });
}

fn main_area(s: &mut Scene, home: &HomeData) {
    let greeting = match home.profile.as_ref() {
        Some(p) if !p.display_name.is_empty() => format!("Good evening, {}", p.display_name),
        _ => "Good evening".to_string(),
    };
    s.col("main_area")
        .w(Len::Fill)
        .h(Len::Fill)
        .pad(24.0)
        .gap(16.0)
        .rgba(theme::PANEL[0], theme::PANEL[1], theme::PANEL[2], 0.75)
        .radius(8.0)
        .child(|c| {
            c.text((), greeting, 28.0)
                .color(theme::TEXT)
                .max_width_px(520.0);
            c.text((), "Recently played", 16.0).color(theme::TEXT_DIM);
            c.row(())
                .w(Len::Fill)
                .h_px(180.0)
                .gap(12.0)
                .child(|g| {
                    let recent = &home.recent;
                    let n = recent.len().clamp(1, 4);
                    for i in 0..n {
                        let (title, sub) = match recent.get(i) {
                            Some(t) => (t.name.as_str(), t.artist.as_str()),
                            None => ("Album", "Artist"),
                        };
                        recent_card(g, title, sub);
                    }
                    for _ in n..4 {
                        recent_card(g, "Album", "Artist");
                    }
                });
        });
}

fn recent_card(s: &mut Scene, title: &str, sub: &str) {
    s.col(())
        .w(Len::Fill)
        .h(Len::Fill)
        .pad(10.0)
        .gap(8.0)
        .rgba(theme::PANEL_HI[0], theme::PANEL_HI[1], theme::PANEL_HI[2], 1.0)
        .radius(8.0)
        .child(|card| {
            card.rect(())
                .w(Len::Fill)
                .h_px(96.0)
                .rgba(0.25, 0.25, 0.30, 1.0)
                .radius(6.0);
            card.text((), title, 13.0)
                .color(theme::TEXT)
                .max_width_px(96.0);
            card.text((), sub, 11.0)
                .color(theme::TEXT_DIM)
                .max_width_px(96.0);
        });
}

fn now_playing(s: &mut Scene, v: &HomeView) {
    s.col("now_playing")
        .w_px(340.0)
        .h(Len::Fill)
        .pad(16.0)
        .gap(12.0)
        .rgba(theme::PANEL[0], theme::PANEL[1], theme::PANEL[2], 0.75)
        .radius(8.0)
        .child(|c| {
            c.text((), "Now playing", 16.0).color(theme::TEXT);
            // Fixed-size box so the abs crossfade layers have a definite
            // parent to Fill against.
            c.col(())
                .w(Len::Fill)
                .h_px(280.0)
                .child(|b| {
                    crossfaded_art(b, v.backdrop_prev, v.backdrop_curr, v.panel_crossfade_t, 8.0)
                });
            // Reactive labels: updated via text bind on track change, no
            // rebuild.
            c.text_bound((), v.title.clone(), 14.0)
                .color(theme::TEXT)
                .max_width_px(300.0);
            c.text_bound((), v.artist.clone(), 12.0)
                .color(theme::TEXT_DIM)
                .max_width_px(300.0);
        });
}

fn player_bar(s: &mut Scene, icons: &IconSet, v: &HomeView) {
    s.row("playerbar")
        .w(Len::Fill)
        .h_px(80.0)
        .pad_xy(16.0, 8.0)
        .gap(12.0)
        .align(Align::Center)
        .rgba(0.0, 0.0, 0.0, 0.45)
        .child(|c| {
            c.row(())
                .w_px(300.0)
                .h(Len::Fill)
                .gap(10.0)
                .align(Align::Center)
                .child(|l| {
                    l.col(())
                        .w_px(56.0)
                        .h_px(56.0)
                        .child(|b| {
                            crossfaded_art(b, v.backdrop_prev, v.backdrop_curr, v.panel_crossfade_t, 4.0)
                        });
                    l.col(())
                        .gap(2.0)
                        .h(Len::Fill)
                        .justify(Justify::Center)
                        .child(|m| {
                            m.text_bound((), v.title.clone(), 13.0)
                                .color(theme::TEXT)
                                .max_width_px(180.0);
                            m.text_bound((), v.artist.clone(), 11.0)
                                .color(theme::TEXT_DIM)
                                .max_width_px(180.0);
                        });
                    l.row(()).push_end().center().child(|h| {
                        icons.render(h, Icon::Heart, 18.0, theme::TEXT_DIM);
                    });
                });
            c.col(())
                .w(Len::Fill)
                .h(Len::Fill)
                .align(Align::Center)
                .justify(Justify::Center)
                .gap(6.0)
                .child(|ct| {
                    ct.row(()).gap(18.0).align(Align::Center).center().child(|t| {
                        // Shuffle/repeat tint reactively: accent when
                        // active, dim grey when off — a Computed over the
                        // state + accent signals, so toggling (or the
                        // accent cross-fade) updates without a rebuild.
                        let shuffle_tint =
                            Computed::new((v.shuffle.clone(), v.accent.clone()), |(on, acc)| {
                                if on { acc } else { theme::TEXT_DIM }
                            });
                        transport_btn(t, icons, Icon::Shuffle, 18.0, shuffle_tint);
                        transport_btn(t, icons, Icon::SkipBack, 20.0, theme::TEXT);
                        // Play-pill: accent background + a reactive
                        // play/pause glyph (image bind swaps the handle on
                        // is_playing — no rebuild).
                        let play_h = icons.get(Icon::Play);
                        let pause_h = icons.get(Icon::Pause);
                        let play_glyph =
                            Computed::new((v.is_playing.clone(),), move |(playing,)| {
                                Some(if playing { pause_h } else { play_h })
                            });
                        t.row(())
                            .w_px(36.0)
                            .h_px(36.0)
                            .color(v.accent.clone())
                            .hover_opacity(0.85)
                            .radius(18.0)
                            .center()
                            .child(|p| {
                                p.image_bound((), play_glyph)
                                    .w_px(16.0)
                                    .h_px(16.0)
                                    .color([0.0, 0.0, 0.0, 1.0]);
                            });
                        transport_btn(t, icons, Icon::SkipForward, 20.0, theme::TEXT);
                        let repeat_tint =
                            Computed::new((v.repeat_on.clone(), v.accent.clone()), |(on, acc)| {
                                if on { acc } else { theme::TEXT_DIM }
                            });
                        transport_btn(t, icons, Icon::Repeat, 18.0, repeat_tint);
                    });
                    ct.row(())
                        .w(Len::Fill)
                        .h_px(4.0)
                        .pad_xy(40.0, 0.0)
                        .child(|sl| {
                            sl.rect(())
                                .w(Len::Fill)
                                .h_px(4.0)
                                .rgba(1.0, 1.0, 1.0, 0.10)
                                .radius(2.0)
                                .child(|bar| {
                                    // Reactive fill width — tracks playback
                                    // via the % width bind, no rebuild.
                                    //
                                    // Quantized to ~pixel granularity: the
                                    // progress signal is tweened at 60 fps,
                                    // but the bar creeps only a few px/sec,
                                    // so applying every 60 fps tick forces a
                                    // full-window redraw (incl. the costly
                                    // glass sample) for sub-pixel motion.
                                    // The Computed only bumps its version
                                    // when the quantized value changes, so
                                    // the bar redraws a handful of times a
                                    // second — visually identical, fraction
                                    // of the GPU. (Crossfades still 60 fps.)
                                    let progress_q = Computed::new(
                                        (v.progress.clone(),),
                                        |(p,)| (p * PROGRESS_STEPS).round() / PROGRESS_STEPS,
                                    );
                                    bar.rect(())
                                        .width_pct(progress_q)
                                        .h_px(4.0)
                                        .rgba(theme::TEXT[0], theme::TEXT[1], theme::TEXT[2], 1.0)
                                        .radius(2.0);
                                });
                        });
                });
            c.row(())
                .w_px(160.0)
                .h(Len::Fill)
                .gap(8.0)
                .align(Align::Center)
                .justify(Justify::End)
                .child(|r| {
                    icons.render(r, Icon::Volume, 18.0, theme::TEXT);
                    r.rect(())
                        .w_px(96.0)
                        .h_px(4.0)
                        .rgba(1.0, 1.0, 1.0, 0.10)
                        .radius(2.0);
                });
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
) {
    s.row(())
        .w_px(32.0)
        .h_px(32.0)
        .center()
        .child(|c| {
            icons.render(c, icon, size, tint);
        });
}
