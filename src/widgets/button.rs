//! Reusable pill button — the bordered, hover-filling, rounded button
//! used for account actions (the settings "Sign out") and the login
//! screen's Back / Reset controls.
//!
//! Auto-sizes to its content (icon + label) so call sites don't hand-pick
//! a width. Two tones: [`ButtonTone::Neutral`] (the default account-action
//! look) and [`ButtonTone::Danger`] (a red scheme for destructive actions
//! like resetting preferences).

use opal_gfx::{Align, EventCtx, Len, Scene};

use crate::widgets::icon::{Icon, IconSet};
use crate::widgets::tokens as t;

/// Visual scheme for a [`pill_button`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonTone {
    /// Neutral outline — white text/border, subtle hover fill.
    Neutral,
    /// Destructive — red text/border, red hover fill.
    Danger,
}

impl ButtonTone {
    /// `(text+icon colour, border colour, hover-fill colour)`.
    fn colors(self) -> ([f32; 4], [f32; 4], [f32; 4]) {
        match self {
            ButtonTone::Neutral => (t::TEXT, t::BORDER, t::BTN_HOVER),
            ButtonTone::Danger => (
                [0.93, 0.46, 0.46, 1.0],
                [0.93, 0.46, 0.46, 0.45],
                [0.85, 0.20, 0.20, 0.18],
            ),
        }
    }
}

/// Build a pill button with an optional leading `icon` and a text `label`.
/// Auto-sized (content + horizontal padding); chain nothing else — pass the
/// click handler in. Mirrors the settings "Sign out" button styling.
pub fn pill_button(
    s: &mut Scene,
    icons: &IconSet,
    label: &str,
    icon: Option<Icon>,
    tone: ButtonTone,
    on_click: impl Fn(&mut EventCtx) + 'static,
) {
    let (fg, border, hover) = tone.colors();
    s.row(())
        .w(Len::Auto)
        .h_px(t::SP_9)
        .pad_xy(t::SP_4, 0.0)
        .gap(t::SP_2)
        .align(Align::Center)
        .radius(t::R_FULL)
        .border(1.0, border)
        .hover_color(hover)
        .on_click(on_click)
        .child(|b| {
            if let Some(ic) = icon {
                icons.render(b, ic, t::ICON_SM, fg);
            }
            b.text((), label, t::TEXT_SM).color(fg);
        });
}
