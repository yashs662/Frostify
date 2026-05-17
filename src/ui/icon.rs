use std::collections::HashMap;

use frostify_gfx::{App, ImageHandle, Scene};

const RASTER_PX: u32 = 64;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Icon {
    Menu,
    ChevronLeft,
    ChevronRight,
    Settings,
    Bell,
    Play,
    Pause,
    SkipBack,
    SkipForward,
    Shuffle,
    Repeat,
    Volume,
    Minimize,
    Maximize,
    Close,
    Home,
    Search,
    Plus,
    Heart,
}

impl Icon {
    fn svg_bytes(self) -> &'static [u8] {
        match self {
            Icon::Menu => include_bytes!("../../assets/icons/menu.svg"),
            Icon::ChevronLeft => include_bytes!("../../assets/icons/chevron-left.svg"),
            Icon::ChevronRight => include_bytes!("../../assets/icons/chevron-right.svg"),
            Icon::Settings => include_bytes!("../../assets/icons/settings.svg"),
            Icon::Bell => include_bytes!("../../assets/icons/bell.svg"),
            Icon::Play => include_bytes!("../../assets/icons/play.svg"),
            Icon::Pause => include_bytes!("../../assets/icons/pause.svg"),
            Icon::SkipBack => include_bytes!("../../assets/icons/skip-back.svg"),
            Icon::SkipForward => include_bytes!("../../assets/icons/skip-forward.svg"),
            Icon::Shuffle => include_bytes!("../../assets/icons/shuffle.svg"),
            Icon::Repeat => include_bytes!("../../assets/icons/repeat.svg"),
            Icon::Volume => include_bytes!("../../assets/icons/volume.svg"),
            Icon::Minimize => include_bytes!("../../assets/icons/minimize.svg"),
            Icon::Maximize => include_bytes!("../../assets/icons/maximize.svg"),
            Icon::Close => include_bytes!("../../assets/icons/close.svg"),
            Icon::Home => include_bytes!("../../assets/icons/home.svg"),
            Icon::Search => include_bytes!("../../assets/icons/search.svg"),
            Icon::Plus => include_bytes!("../../assets/icons/plus.svg"),
            Icon::Heart => include_bytes!("../../assets/icons/heart.svg"),
        }
    }
}

const ALL: &[Icon] = &[
    Icon::Menu,
    Icon::ChevronLeft,
    Icon::ChevronRight,
    Icon::Settings,
    Icon::Bell,
    Icon::Play,
    Icon::Pause,
    Icon::SkipBack,
    Icon::SkipForward,
    Icon::Shuffle,
    Icon::Repeat,
    Icon::Volume,
    Icon::Minimize,
    Icon::Maximize,
    Icon::Close,
    Icon::Home,
    Icon::Search,
    Icon::Plus,
    Icon::Heart,
];

#[derive(Clone)]
pub struct IconSet {
    handles: HashMap<Icon, ImageHandle>,
}

impl IconSet {
    pub fn get(&self, icon: Icon) -> ImageHandle {
        *self.handles.get(&icon).expect("icon not loaded — extend ALL")
    }

    pub fn render(&self, s: &mut Scene, icon: Icon, size_px: f32, color: [f32; 4]) {
        s.image((), self.get(icon))
            .w_px(size_px)
            .h_px(size_px)
            .color(color);
    }
}

pub fn load_all(app: &mut App) -> IconSet {
    let mut handles = HashMap::with_capacity(ALL.len());
    for &icon in ALL {
        let h = app.stage_image_svg(icon.svg_bytes(), RASTER_PX);
        handles.insert(icon, h);
    }
    IconSet { handles }
}
