//! The Login / Splash view — the pre-auth screen. Owns its single
//! callback (start OAuth) and renders the login chrome; `checking` (Splash
//! state) shows a "checking saved credentials" message instead of the
//! button while the stored token is being validated.

use std::rc::Rc;

use opal_gfx::{Len, Scene};

use crate::app::AppState;
use crate::views::View;
use crate::widgets::icon::IconSet;
use crate::widgets::{chrome, tokens};
use crate::worker::Worker;

/// The Login view controller — owns the OAuth-start callback and renders
/// the login screen, reading Splash-vs-Login from the router.
pub struct LoginView {
    state: Rc<AppState>,
    icons: Rc<IconSet>,
    on_login: Rc<dyn Fn()>,
}

impl LoginView {
    pub fn new(state: Rc<AppState>, worker: Rc<Worker>, icons: Rc<IconSet>) -> Self {
        let on_login: Rc<dyn Fn()> = Rc::new(move || worker.start_oauth());
        Self { state, icons, on_login }
    }

    pub fn build(&self, s: &mut Scene) {
        // Splash = still validating the stored token → show "checking".
        let checking = matches!(self.state.router.view.get(), View::Splash);
        render(s, &self.icons, self.on_login.clone(), checking);
    }
}

fn render(s: &mut Scene, icons: &IconSet, on_login: Rc<dyn Fn()>, checking: bool) {
    s.col(())
        .fill()
        .rgba(tokens::BG[0], tokens::BG[1], tokens::BG[2], 1.0)
        .child(|root| {
            chrome::title_bar(root, icons, "Opal");

            root.col(())
                .w(Len::Fill)
                .h(Len::Fill)
                .center()
                .gap(20.0)
                .child(|c| {
                    c.text((), "Opal", 36.0).color(tokens::TEXT);
                    c.text((), "An unofficial Spotify desktop client.", 14.0)
                        .color(tokens::TEXT_DIM);

                    if checking {
                        c.text((), "Checking saved credentials...", 13.0)
                            .color(tokens::TEXT_DIM);
                    } else {
                        let cb = on_login.clone();
                        c.row(())
                            .w_px(240.0)
                            .h_px(48.0)
                            .color(tokens::ACCENT)
                            .hover_color(tokens::ACCENT_HOVER)
                            .radius(24.0)
                            .center()
                            .on_click(move |_| cb())
                            .child(|b| {
                                b.text((), "Log in with Spotify", 14.0).color([1.0, 1.0, 1.0, 1.0]);
                            });
                    }
                });
        });
}
