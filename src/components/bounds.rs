#[derive(Debug, Copy, Clone)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Copy, Clone)]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Bounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn get_anchor_position(&self, anchor: Anchor) -> (f32, f32) {
        match anchor {
            Anchor::TopLeft => (self.x, self.y),
            Anchor::Top => (self.x + self.width/2.0, self.y),
            Anchor::TopRight => (self.x + self.width, self.y),
            Anchor::Left => (self.x, self.y + self.height/2.0),
            Anchor::Center => (self.x + self.width/2.0, self.y + self.height/2.0),
            Anchor::Right => (self.x + self.width, self.y + self.height/2.0),
            Anchor::BottomLeft => (self.x, self.y + self.height),
            Anchor::Bottom => (self.x + self.width/2.0, self.y + self.height),
            Anchor::BottomRight => (self.x + self.width, self.y + self.height),
        }
    }
}
