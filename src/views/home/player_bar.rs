//! Player bar — the bottom transport strip, a [`Component`].
//!
//! Reads its own `backdrop` (thumb crossfade + accent) and `player`
//! (title/artist + transport toggles + progress) slices directly, and
//! raises transport intents through `on_action`. The UI layer stays
//! ignorant of tokens/Web API — `on_action` is wired by the host to the
//! optimistic signal flips + worker commands.

use std::rc::Rc;

use frostify_gfx::{Align, Computed, CursorIcon, Justify, Len, Scene};

use crate::model::{BackdropModel, PlayerModel};
use crate::widgets::component::Component;
use crate::views::home::PlayerAction;
use crate::widgets::color::accent_fg;
use crate::widgets::crossfade::crossfaded_art;
use crate::widgets::icon::{Icon, IconSet};
use crate::widgets::tokens as t;

/// Progress-bar quantization steps across its full width. The fill only
/// redraws when `round(progress * STEPS)` changes — ~1 step ≈ 1 px on a
/// ~600 px bar, so it redraws a few times a second instead of 60, with no
/// visible stepping (it advances only a few px/sec).
const PROGRESS_STEPS: f32 = 600.0;

pub struct PlayerBar<'a> {
    pub backdrop: &'a BackdropModel,
    pub player: &'a PlayerModel,
    pub on_action: Rc<dyn Fn(PlayerAction)>,
    /// `&Rc<IconSet>` (not `&IconSet`) so the host can pass `&icons`
    /// directly; deref-coercion handles the `render`/`get`/helper calls.
    pub icons: &'a Rc<IconSet>,
}

impl Component for PlayerBar<'_> {
    fn view(&self, s: &mut Scene) {
        let icons = self.icons;
        s.row("playerbar")
            .w(Len::Fill)
            .h_px(t::PLAYER_H)
            .pad_xy(t::SP_4, t::SP_2)
            .gap(t::SP_3)
            .align(Align::Center)
            .rgba(0.0, 0.0, 0.0, 0.45)
            .child(|c| {
                // Left: thumb + title/artist + like.
                c.row(())
                    .w_px(t::SP_80 - t::SP_5)
                    .h(Len::Fill)
                    .gap(t::SP_2_5)
                    .align(Align::Center)
                    .child(|l| {
                        l.col(()).w_px(t::SP_14).h_px(t::SP_14).child(|b| {
                            crossfaded_art(
                                b,
                                &self.backdrop.prev,
                                &self.backdrop.curr,
                                &self.backdrop.panel_t,
                                t::R_SM,
                            )
                        });
                        l.col(())
                            .gap(t::SP_0_5)
                            .h(Len::Fill)
                            .justify(Justify::Center)
                            .child(|m| {
                                m.text_bound((), self.player.title.clone(), 13.0)
                                    .color(t::TEXT)
                                    .max_width_px(180.0);
                                m.text_bound((), self.player.artist.clone(), 11.0)
                                    .color(t::TEXT_DIM)
                                    .max_width_px(180.0);
                            });
                        l.row(()).push_end().center().child(|h| {
                            icons.render(h, Icon::Heart, t::ICON_MD, t::TEXT_DIM);
                        });
                    });
                // Centre: transport controls + progress.
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
                                    (self.player.shuffle.clone(), self.backdrop.accent.clone()),
                                    |(on, acc)| if on { acc } else { t::TEXT_DIM },
                                );
                                transport_btn(tr, icons, Icon::Shuffle, t::ICON_MD, shuffle_tint, {
                                    let act = self.on_action.clone();
                                    move || act(PlayerAction::ToggleShuffle)
                                });
                                transport_btn(tr, icons, Icon::SkipBack, t::ICON_LG, t::TEXT, {
                                    let act = self.on_action.clone();
                                    move || act(PlayerAction::Prev)
                                });
                                let play_h = icons.get(Icon::Play);
                                let pause_h = icons.get(Icon::Pause);
                                let play_glyph =
                                    Computed::new((self.player.is_playing.clone(),), move |(playing,)| {
                                        Some(if playing { pause_h } else { play_h })
                                    });
                                let play_act = self.on_action.clone();
                                tr.row(())
                                    .w_px(t::SP_9)
                                    .h_px(t::SP_9)
                                    .color(self.backdrop.accent.clone())
                                    .hover_opacity(0.85)
                                    .radius(t::R_FULL)
                                    .center()
                                    .on_click(move |_| play_act(PlayerAction::PlayPause))
                                    .child(|p| {
                                        p.image_bound((), play_glyph)
                                            .w_px(t::SP_4)
                                            .h_px(t::SP_4)
                                            .color(accent_fg(&self.backdrop.accent));
                                    });
                                transport_btn(tr, icons, Icon::SkipForward, t::ICON_LG, t::TEXT, {
                                    let act = self.on_action.clone();
                                    move || act(PlayerAction::Next)
                                });
                                let repeat_tint = Computed::new(
                                    (self.player.repeat_on.clone(), self.backdrop.accent.clone()),
                                    |(on, acc)| if on { acc } else { t::TEXT_DIM },
                                );
                                transport_btn(tr, icons, Icon::Repeat, t::ICON_MD, repeat_tint, {
                                    let act = self.on_action.clone();
                                    move || act(PlayerAction::CycleRepeat)
                                });
                            });
                        // Scrubbable progress bar. The lane is taller than
                        // the visible track so it's easy to grab; the
                        // interactive node maps the cursor to a fraction via
                        // its own rect (`on_drag` = press/scrub, click-to-set
                        // included; `on_hover_move` = un-pressed preview).
                        self.seek_bar(ct);
                    });
                // Right: lossless badge + queue/devices/volume.
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
}

impl PlayerBar<'_> {
    /// The scrubbable progress bar. A tall transparent lane (easy to grab)
    /// holds the thin visible track; the lane maps the cursor to a fraction
    /// via its own rect. `on_drag` handles press (click-to-seek, fired once
    /// at press) + scrub; `on_hover_move` drives the un-pressed timestamp
    /// preview. The fill follows the cursor while dragging, the live
    /// position otherwise. The commit (Web API seek) fires on release from
    /// the frame loop (`PlayerModel::tick_seek`).
    fn seek_bar(&self, ct: &mut Scene) {
        let seek_drag = self.player.seek_handle();
        let seek_hover = self.player.seek_handle();
        // `seeking` tracks the held scrub via on_drag(true)+on_drag_end(false)
        // — NOT on_press, which would flip false when the cursor drags off
        // the bar (committing early). on_drag fires while captured anywhere.
        let seeking_drag = self.player.seeking.clone();
        let seeking_end = self.player.seeking.clone();
        // Elapsed (left) + total (right) timestamps flank the bar. Fixed
        // width so the bar doesn't shift as the labels change width.
        ct.row(())
            .w(Len::Fill)
            .h_px(t::SP_4)
            .gap(t::SP_2)
            .align(Align::Center)
            .child(|sl| {
                sl.row(())
                    .w_px(t::SP_10)
                    .justify(Justify::End)
                    .child(|c| {
                        c.text_bound((), self.player.elapsed_label.clone(), 10.0)
                            .color(t::TEXT_DIM);
                    });
                sl.row("seekbar")
                    .w(Len::Fill)
                    .h(Len::Fill)
                    .align(Align::Center)
                    .cursor(CursorIcon::Pointer)
                    .on_hover(self.player.bar_hovered.clone())
                    .on_drag(move |ctx| {
                        // ctx is physical px; the tooltip's composite offset
                        // is logical, so convert by the display scale.
                        let s = ctx.scale.max(1.0);
                        seek_drag.set_at((ctx.current[0] - ctx.rect[0]) / s, ctx.rect[2] / s);
                        seeking_drag.set(true);
                    })
                    .on_drag_end(move |_| {
                        seeking_end.set(false);
                    })
                    .on_hover_move(move |ctx| {
                        let s = ctx.scale.max(1.0);
                        seek_hover.set_at((ctx.pos[0] - ctx.rect[0]) / s, ctx.rect[2] / s);
                    })
                    .child(|lane| {
                        // Tooltip: a pill above the bar, centred on the cursor.
                        // It's promoted to a composite layer driven by
                        // `layer_offset_x` (the cursor's px along the bar), so
                        // following the cursor is a composite-only translate —
                        // no relayout / re-flatten of the scene. The pill is
                        // centred inside a fixed-width box, and the box's left
                        // edge sits half a box-width left of the bar origin, so
                        // `offset = cursor_px` lands the pill *centred* on the
                        // cursor. Visibility is a composite opacity bind.
                        let tip_opacity = Computed::new(
                            (self.player.bar_hovered.clone(), self.player.seeking.clone()),
                            |(h, s)| if h || s { 1.0 } else { 0.0 },
                        );
                        lane.row(())
                            .abs(-TIP_W / 2.0, -t::SP_6)
                            .w_px(TIP_W)
                            .h_px(t::SP_5)
                            .center()
                            .opacity_bind(tip_opacity)
                            .layer_offset_x(self.player.seek_preview_px.clone())
                            .child(|tip| {
                                tip.row(())
                                    .h_px(t::SP_5)
                                    .pad_xy(t::SP_2, t::SP_0)
                                    .center()
                                    .rgba(0.0, 0.0, 0.0, 0.9)
                                    .radius(t::R_SM)
                                    .border(1.0, t::BORDER)
                                    .child(|c| {
                                        c.text_bound((), self.player.seek_label.clone(), 11.0)
                                            .color(t::TEXT);
                                    });
                            });
                        // Visible track + fill (cursor while dragging, else
                        // live progress; quantized so it doesn't redraw 60/s).
                        let fill = Computed::new(
                            (
                                self.player.seeking.clone(),
                                self.player.seek_preview.clone(),
                                self.player.progress.clone(),
                            ),
                            |(s, prev, prog)| {
                                let f = if s { prev } else { prog };
                                (f * PROGRESS_STEPS).round() / PROGRESS_STEPS
                            },
                        );
                        lane.rect(())
                            .w(Len::Fill)
                            .h_px(t::SP_1)
                            .rgba(1.0, 1.0, 1.0, 0.10)
                            .radius(t::R_SM / 2.0)
                            .child(|bar| {
                                bar.rect(())
                                    .width_pct(fill)
                                    .h_px(t::SP_1)
                                    .rgba(t::TEXT[0], t::TEXT[1], t::TEXT[2], 1.0)
                                    .radius(t::R_SM / 2.0);
                            });
                    });
                sl.row(())
                    .w_px(t::SP_10)
                    .justify(Justify::Start)
                    .child(|c| {
                        c.text_bound((), self.player.total_label.clone(), 10.0)
                            .color(t::TEXT_DIM);
                    });
            });
    }
}

/// Fixed width (logical px) of the seek tooltip's centring box. The pill is
/// centred inside it; must exceed the widest "M:SS"/"MM:SS" pill.
const TIP_W: f32 = 96.0;

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

/// Transport icon button. `tint` accepts a static colour, a `Signal`, or a
/// `Computed` (via `Into<Bind>`), so active-toggle states route a reactive
/// tint and update without a rebuild.
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
