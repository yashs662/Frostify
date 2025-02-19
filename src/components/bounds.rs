use super::{ComponentPosition, ComponentSize};

#[derive(Debug, Copy, Clone)]
pub struct Bounds {
    pub position: ComponentPosition,
    pub size: ComponentSize,
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
    pub fn new(position: ComponentPosition, size: ComponentSize) -> Self {
        Self { position, size }
    }

    pub fn get_anchor_position(&self, anchor: Anchor) -> (f32, f32) {
        match anchor {
            Anchor::TopLeft => (self.position.x, self.position.y),
            Anchor::Top => (self.position.x + self.size.width / 2.0, self.position.y),
            Anchor::TopRight => (self.position.x + self.size.width, self.position.y),
            Anchor::Left => (self.position.x, self.position.y + self.size.height / 2.0),
            Anchor::Center => (
                self.position.x + self.size.width / 2.0,
                self.position.y + self.size.height / 2.0,
            ),
            Anchor::Right => (
                self.position.x + self.size.width,
                self.position.y + self.size.height / 2.0,
            ),
            Anchor::BottomLeft => (self.position.x, self.position.y + self.size.height),
            Anchor::Bottom => (
                self.position.x + self.size.width / 2.0,
                self.position.y + self.size.height,
            ),
            Anchor::BottomRight => (
                self.position.x + self.size.width,
                self.position.y + self.size.height,
            ),
        }
    }
}
