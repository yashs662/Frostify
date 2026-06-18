//! Right-click context-menu slice.
//!
//! Holds the open state, the logical-px position to anchor the menu at
//! (the cursor), and the right-clicked track's actionable data. Opening
//! requests a scene rebuild (like the other popups), so the menu renders
//! at the new position with the new target's actions; dismissing closes
//! it the same way.

use std::cell::{Cell, RefCell};

/// The right-clicked track's data the menu acts on.
#[derive(Clone, Default)]
pub struct MenuTarget {
    /// `spotify:track:…` URI — Add to queue.
    pub uri: String,
    /// Album id — "Go to album" (empty hides the item).
    pub album_id: String,
    /// First-artist id — "Go to artist" (empty hides the item).
    pub artist_id: String,
}

pub struct MenuModel {
    pub open: Cell<bool>,
    /// Anchor position in **logical px** (cursor at right-click time).
    pub pos: Cell<[f32; 2]>,
    pub target: RefCell<MenuTarget>,
}

impl MenuModel {
    pub fn new() -> Self {
        Self {
            open: Cell::new(false),
            pos: Cell::new([0.0; 2]),
            target: RefCell::default(),
        }
    }

    pub fn show(&self, target: MenuTarget, pos: [f32; 2]) {
        *self.target.borrow_mut() = target;
        self.pos.set(pos);
        self.open.set(true);
    }

    pub fn close(&self) {
        self.open.set(false);
    }
}

impl Default for MenuModel {
    fn default() -> Self {
        Self::new()
    }
}
