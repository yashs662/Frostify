#[derive(Debug, Clone, Copy)]
pub enum Color {
    AliceBlue,
    AntiqueWhite,
    Aquamarine,
    Azure,
    Beige,
    Bisque,
    Black,
    Blue,
    Crimson,
    Cyan,
    DarkGray,
    DarkGreen,
    Fuchsia,
    Gold,
    Gray,
    Green,
    Indigo,
    LimeGreen,
    Maroon,
    MidnightBlue,
    Navy,
    Olive,
    Orange,
    OrangeRed,
    Pink,
    Purple,
    Red,
    Salmon,
    SeaGreen,
    Silver,
    Teal,
    Tomato,
    Turquoise,
    Violet,
    White,
    Yellow,
    YellowGreen,
    Custom([f32; 4]),
}

impl Color {
    pub fn value(&self) -> [f32; 4] {
        match *self {
            Color::AliceBlue => [0.94, 0.97, 1.0, 1.0],
            Color::AntiqueWhite => [0.98, 0.92, 0.84, 1.0],
            Color::Aquamarine => [0.49, 1.0, 0.83, 1.0],
            Color::Azure => [0.94, 1.0, 1.0, 1.0],
            Color::Beige => [0.96, 0.96, 0.86, 1.0],
            Color::Bisque => [1.0, 0.89, 0.77, 1.0],
            Color::Black => [0.0, 0.0, 0.0, 1.0],
            Color::Blue => [0.0, 0.0, 1.0, 1.0],
            Color::Crimson => [0.86, 0.08, 0.24, 1.0],
            Color::Cyan => [0.0, 1.0, 1.0, 1.0],
            Color::DarkGray => [0.25, 0.25, 0.25, 1.0],
            Color::DarkGreen => [0.0, 0.5, 0.0, 1.0],
            Color::Fuchsia => [1.0, 0.0, 1.0, 1.0],
            Color::Gold => [1.0, 0.84, 0.0, 1.0],
            Color::Gray => [0.5, 0.5, 0.5, 1.0],
            Color::Green => [0.0, 1.0, 0.0, 1.0],
            Color::Indigo => [0.29, 0.0, 0.51, 1.0],
            Color::LimeGreen => [0.2, 0.8, 0.2, 1.0],
            Color::Maroon => [0.5, 0.0, 0.0, 1.0],
            Color::MidnightBlue => [0.1, 0.1, 0.44, 1.0],
            Color::Navy => [0.0, 0.0, 0.5, 1.0],
            Color::Olive => [0.5, 0.5, 0.0, 1.0],
            Color::Orange => [1.0, 0.65, 0.0, 1.0],
            Color::OrangeRed => [1.0, 0.27, 0.0, 1.0],
            Color::Pink => [1.0, 0.08, 0.58, 1.0],
            Color::Purple => [0.5, 0.0, 0.5, 1.0],
            Color::Red => [1.0, 0.0, 0.0, 1.0],
            Color::Salmon => [0.98, 0.5, 0.45, 1.0],
            Color::SeaGreen => [0.18, 0.55, 0.34, 1.0],
            Color::Silver => [0.75, 0.75, 0.75, 1.0],
            Color::Teal => [0.0, 0.5, 0.5, 1.0],
            Color::Tomato => [1.0, 0.39, 0.28, 1.0],
            Color::Turquoise => [0.25, 0.88, 0.82, 1.0],
            Color::Violet => [0.93, 0.51, 0.93, 1.0],
            Color::White => [1.0, 1.0, 1.0, 1.0],
            Color::Yellow => [1.0, 1.0, 0.0, 1.0],
            Color::YellowGreen => [0.6, 0.8, 0.2, 1.0],
            Color::Custom(color) => color,
        }
    }

    pub fn with_alpha(&self, alpha: f32) -> Color {
        let [r, g, b, _] = self.value();
        Color::Custom([r, g, b, alpha])
    }

    pub fn darken(&self, factor: f32) -> Color {
        let [r, g, b, a] = self.value();
        Color::Custom([r * factor, g * factor, b * factor, a])
    }

    pub fn lighten(&self, factor: f32) -> Color {
        let [r, g, b, a] = self.value();
        Color::Custom([
            r + (1.0 - r) * factor,
            g + (1.0 - g) * factor,
            b + (1.0 - b) * factor,
            a,
        ])
    }

    pub fn to_rgb_0_255(self) -> [u8; 4] {
        let [r, g, b, a] = self.value();
        [
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        ]
    }

    pub fn to_glyphon_color(self) -> glyphon::Color {
        let [r, g, b, a] = self.to_rgb_0_255();
        glyphon::Color::rgba(r, g, b, a)
    }
}
