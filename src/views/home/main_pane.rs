//! Centre pane — a [`Component`] that wraps a constant panel around a
//! slide/fade transition layer whose content swaps between the Home feed
//! and a playlist page. The panel itself never transitions — only its
//! inner content — so a nav change reads as the content sliding up +
//! fading in, not the whole pane flickering.

use std::rc::Rc;

use frostify_gfx::{Align, Computed, ImageHandle, Justify, Len, Scene, Signal};

use crate::album_art;
use crate::api::{AlbumRef, HomeData};
use crate::views::MainNav;
use crate::widgets::component::Component;
use crate::views::home::{ArtMap, NavFn, PlayFn};
use crate::widgets::chip::chip;
use crate::widgets::color::accent_fg;
use crate::widgets::thumb::thumb;
use crate::widgets::icon::{Icon, IconSet};
use crate::views::home::playlist::{self, PlaylistViewData};
use crate::widgets::tokens as t;

/// (title, subtitle, resolved cover signal) — pre-baked per tile.
type TileEntry = (String, String, Option<Signal<Option<ImageHandle>>>);

/// Content filter tabs shown across the top of the pane.
const FILTERS: &[&str] = &["All", "Music", "Podcasts", "Audiobooks"];

pub struct MainPane<'a> {
    pub icons: &'a Rc<IconSet>,
    pub home: &'a HomeData,
    pub art: &'a ArtMap,
    pub accent: &'a Signal<[f32; 4]>,
    /// What the pane shows (Home feed vs a playlist page).
    pub nav: &'a MainNav,
    /// View data for the open playlist (`Some` when `nav` is a Playlist).
    pub playlist: Option<&'a PlaylistViewData>,
    /// 0 → 1 entrance transition progress on nav change.
    pub main_t: &'a Signal<f32>,
    pub on_play: PlayFn,
    pub on_navigate: NavFn,
}

impl Component for MainPane<'_> {
    fn view(&self, s: &mut Scene) {
        s.col("main_area")
            .w(Len::Fill)
            .h(Len::Fill)
            .rgba(t::PANEL[0], t::PANEL[1], t::PANEL[2], 0.75)
            .radius(t::R_LG)
            .clip()
            .child(|outer| {
                // Transition wrapper — abs-fill so the slide offset doesn't
                // disturb flow. `main_t` 0→1 drives a subtle upward slide +
                // opacity fade-in on every nav change; steady state parks at
                // offset 0, fully opaque.
                let slide = Computed::new((self.main_t.clone(),), |(tt,)| {
                    [0.0, (1.0 - tt.clamp(0.0, 1.0)) * 14.0]
                });
                let fade = Computed::new((self.main_t.clone(),), |(tt,)| tt.clamp(0.0, 1.0));
                outer
                    .col("main_content")
                    .pos(slide)
                    .w(Len::Fill)
                    .h(Len::Fill)
                    .opacity_bind(fade)
                    .child(|content| match self.nav {
                        MainNav::Home => self.home_feed(content),
                        MainNav::Playlist { .. } => {
                            if let Some(pv) = self.playlist {
                                playlist::view(
                                    content,
                                    self.icons,
                                    pv,
                                    self.accent,
                                    self.on_play.clone(),
                                    self.on_navigate.clone(),
                                );
                            }
                        }
                    });
            });
    }
}

impl MainPane<'_> {
    fn home_feed(&self, content: &mut Scene) {
        let icons = self.icons;
        let home = self.home;
        let art = self.art;
        let accent = self.accent;
        let greeting = match home.profile.as_ref() {
            Some(p) if !p.display_name.is_empty() => format!("Good evening, {}", p.display_name),
            _ => "Good evening".to_string(),
        };
        let made_for = match home.profile.as_ref() {
            Some(p) if !p.display_name.is_empty() => format!("Made For {}", p.display_name),
            _ => "Made For You".to_string(),
        };
        // Filter chips pinned at the top of the pane.
        content
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
        content
            .col(())
            .w(Len::Fill)
            .h(Len::Fill)
            .pad_xy(t::SP_6, t::SP_2)
            .gap(t::SP_5)
            .scroll_y()
            // Compositor scroll layer: the feed body rasters once into a
            // content-sized texture; scrolling recomposites the window.
            .layer()
            .child(|c| {
                c.text((), greeting, 26.0).color(t::TEXT).max_width_px(520.0);

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
    }
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
            let sig = url.as_ref().and_then(|u| art.get(&album_art::cache_key(u)).cloned());
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
                    tile(g, "\u{2014}", "", None);
                }
            } else {
                for (title, sub, art) in &entries {
                    tile(g, title, sub, art.clone());
                }
            }
        });
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
            card.text((), title, 13.0).color(t::TEXT).max_width_px(t::TILE_TEXT_MAX);
            card.text((), sub, 11.0).color(t::TEXT_DIM).max_width_px(t::TILE_TEXT_MAX);
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
                    m.text((), &album.name, 20.0).color(t::TEXT).max_width_px(360.0);
                    m.text((), &album.artist, 12.0).color(t::TEXT_DIM).max_width_px(360.0);
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
                    icons.render(p, Icon::Play, t::ICON_MD, accent_fg(accent));
                });
        });
}
