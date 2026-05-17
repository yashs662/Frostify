use frostify_gfx::{Align, Justify, Len, Scene, WindowAction};

use crate::api::HomeData;
use crate::ui::icon::{Icon, IconSet};
use crate::ui::theme;

pub fn build(s: &mut Scene, icons: &IconSet, home: &HomeData) {
    s.col("home_root")
        .fill()
        .rgba(theme::BG[0], theme::BG[1], theme::BG[2], 1.0)
        .child(|root| {
            top_bar(root, icons);
            root.row(())
                .w(Len::Fill)
                .h(Len::Fill)
                .pad(8.0)
                .gap(8.0)
                .child(|b| {
                    sidebar(b, icons, home);
                    main_area(b, home);
                    now_playing(b, home);
                });
            player_bar(root, icons);
        });
}

fn top_bar(s: &mut Scene, icons: &IconSet) {
    s.row("topbar")
        .w(Len::Fill)
        .h_px(56.0)
        .pad_xy(12.0, 0.0)
        .gap(8.0)
        .align(Align::Center)
        .rgba(theme::BG[0], theme::BG[1], theme::BG[2], 1.0)
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
        .rgba(theme::PANEL[0], theme::PANEL[1], theme::PANEL[2], 1.0)
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
        .rgba(theme::PANEL[0], theme::PANEL[1], theme::PANEL[2], 1.0)
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
                    let n = recent.len().min(4).max(1);
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

fn now_playing(s: &mut Scene, home: &HomeData) {
    let last = home.recent.first();
    let (title, artist) = match last {
        Some(t) => (t.name.as_str(), t.artist.as_str()),
        None => ("Nothing playing", "\u{2014}"),
    };
    s.col("now_playing")
        .w_px(340.0)
        .h(Len::Fill)
        .pad(16.0)
        .gap(12.0)
        .rgba(theme::PANEL[0], theme::PANEL[1], theme::PANEL[2], 1.0)
        .radius(8.0)
        .child(|c| {
            c.text((), "Chill", 16.0).color(theme::TEXT);
            c.rect(())
                .w(Len::Fill)
                .h_px(280.0)
                .rgba(0.20, 0.20, 0.24, 1.0)
                .radius(8.0);
            c.text((), title, 14.0)
                .color(theme::TEXT)
                .max_width_px(300.0);
            c.text((), artist, 12.0)
                .color(theme::TEXT_DIM)
                .max_width_px(300.0);
        });
}

fn player_bar(s: &mut Scene, icons: &IconSet) {
    s.row("playerbar")
        .w(Len::Fill)
        .h_px(80.0)
        .pad_xy(16.0, 8.0)
        .gap(12.0)
        .align(Align::Center)
        .rgba(theme::BG[0], theme::BG[1], theme::BG[2], 1.0)
        .child(|c| {
            c.row(())
                .w_px(300.0)
                .h(Len::Fill)
                .gap(10.0)
                .align(Align::Center)
                .child(|l| {
                    l.rect(())
                        .w_px(56.0)
                        .h_px(56.0)
                        .rgba(0.25, 0.25, 0.30, 1.0)
                        .radius(4.0);
                    l.col(())
                        .gap(2.0)
                        .h(Len::Fill)
                        .justify(Justify::Center)
                        .child(|m| {
                            m.text((), "Song name", 13.0)
                                .color(theme::TEXT)
                                .max_width_px(180.0);
                            m.text((), "Artist", 11.0)
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
                        transport_btn(t, icons, Icon::Shuffle, 18.0, theme::TEXT_DIM);
                        transport_btn(t, icons, Icon::SkipBack, 20.0, theme::TEXT);
                        t.row(())
                            .w_px(36.0)
                            .h_px(36.0)
                            .color(theme::TEXT)
                            .hover_opacity(0.85)
                            .radius(18.0)
                            .center()
                            .child(|p| {
                                icons.render(p, Icon::Play, 16.0, [0.0, 0.0, 0.0, 1.0]);
                            });
                        transport_btn(t, icons, Icon::SkipForward, 20.0, theme::TEXT);
                        transport_btn(t, icons, Icon::Repeat, 18.0, theme::TEXT_DIM);
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
                                .radius(2.0);
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

fn transport_btn(s: &mut Scene, icons: &IconSet, icon: Icon, size: f32, color: [f32; 4]) {
    s.row(())
        .w_px(32.0)
        .h_px(32.0)
        .center()
        .child(|c| {
            icons.render(c, icon, size, color);
        });
}
