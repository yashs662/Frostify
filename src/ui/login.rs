use std::rc::Rc;

use frostify_gfx::{Len, Scene};

use crate::ui::icon::IconSet;
use crate::ui::{chrome, theme};

pub fn build(s: &mut Scene, icons: &IconSet, on_login: Rc<dyn Fn()>, checking: bool) {
    s.col(())
        .fill()
        .rgba(theme::BG[0], theme::BG[1], theme::BG[2], 1.0)
        .child(|root| {
            chrome::title_bar(root, icons, "Frostify");

            root.col(())
                .w(Len::Fill)
                .h(Len::Fill)
                .center()
                .gap(20.0)
                .child(|c| {
                    c.text((), "Frostify", 36.0).color(theme::TEXT);
                    c.text((), "An unofficial Spotify desktop client.", 14.0)
                        .color(theme::TEXT_DIM);

                    if checking {
                        c.text((), "Checking saved credentials...", 13.0)
                            .color(theme::TEXT_DIM);
                    } else {
                        let cb = on_login.clone();
                        c.row(())
                            .w_px(240.0)
                            .h_px(48.0)
                            .color(theme::ACCENT)
                            .hover_color(theme::ACCENT_HOVER)
                            .radius(24.0)
                            .center()
                            .on_click(move |_| cb())
                            .child(|b| {
                                b.text((), "Log in with Spotify", 14.0)
                                    .color([1.0, 1.0, 1.0, 1.0]);
                            });
                    }
                });
        });
}
