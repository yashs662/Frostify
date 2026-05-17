use frostify_gfx::{Align, Len, Scene, WindowAction};

use crate::ui::icon::{Icon, IconSet};
use crate::ui::theme;

pub fn title_bar(s: &mut Scene, icons: &IconSet, title: &str) {
    s.row(())
        .w(Len::Fill)
        .h_px(36.0)
        .pad_xy(10.0, 0.0)
        .gap(8.0)
        .align(Align::Center)
        .rgba(theme::PANEL[0], theme::PANEL[1], theme::PANEL[2], 1.0)
        .window_action(WindowAction::DragMove)
        .child(|t| {
            t.text((), title, 13.0).color(theme::TEXT_DIM);

            chrome_btn(t, icons, Icon::Minimize, WindowAction::Minimize, theme::BTN_HOVER, true);
            chrome_btn(t, icons, Icon::Maximize, WindowAction::ToggleMaximize, theme::BTN_HOVER, false);
            chrome_btn(t, icons, Icon::Close, WindowAction::Close, theme::CLOSE_HOVER, false);
        });
}

fn chrome_btn(s: &mut Scene, icons: &IconSet, icon: Icon, action: WindowAction, hover: [f32; 4], push_end: bool) {
    let mut b = s.row(());
    b.w_px(44.0)
        .h_px(36.0)
        .rgba(0.0, 0.0, 0.0, 0.0)
        .hover_color(hover)
        .center()
        .window_action(action);
    if push_end {
        b.push_end();
    }
    b.child(|c| {
        icons.render(c, icon, 14.0, theme::TEXT);
    });
}
