use super::{Anchor, Bounds};
use crate::{
    color::Color,
    components::core::ComponentPosition,
    text_renderer::{OptionalTextUpdateData, TextHandler},
};
use log::trace;
use uuid::Uuid;

pub struct TextComponent {
    pub id: Uuid,
}

impl TextComponent {
    pub fn new(
        text: String,
        font_size: f32,
        line_height_multiplier: f32,
        color: Color,
        bounds: Bounds,
        text_handler: &mut TextHandler,
        anchor: Anchor,
    ) -> Self {
        let id = Uuid::new_v4();

        trace!(
            "Registering text with id: {}, text: {}, color: {:?}",
            id, text, color
        );

        // use the anchor to calculate the position
        let anchor_adjusted_pos = bounds.get_anchor_position(anchor);
        let mut x = anchor_adjusted_pos.0;
        let mut y = anchor_adjusted_pos.1;
        // register with the text handler
        text_handler.register_text(
            id,
            text.clone(),
            font_size,
            line_height_multiplier,
            bounds,
            color,
        );

        if let Some(text_size) = text_handler.measure_text(id) {
            let (adj_x, adj_y) = match anchor {
                Anchor::Top => (x - text_size.width / 2.0, y),
                Anchor::TopRight => (x - text_size.width, y),
                Anchor::Left => (x, y - text_size.height / 2.0),
                Anchor::Center => (x - text_size.width / 2.0, y - text_size.height / 2.0),
                Anchor::Right => (x - text_size.width, y - text_size.height / 2.0),
                Anchor::BottomLeft => (x, y - text_size.height),
                Anchor::Bottom => (x - text_size.width / 2.0, y - text_size.height),
                Anchor::BottomRight => (x - text_size.width, y - text_size.height),
                _ => (x, y),
            };
            x = adj_x;
            y = adj_y;
        }
        let new_position = ComponentPosition { x, y };
        let new_bounds = Bounds::new(new_position, bounds.size);

        text_handler.update((id, OptionalTextUpdateData::new().with_bounds(new_bounds)));

        Self { id }
    }
}
