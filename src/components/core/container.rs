use super::{
    Anchor, Bounds, Component, ComponentOffset, ComponentPosition, ComponentSize,
    ComponentTransform, RenderPassExt,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexAlign {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

pub struct Container {
    children: Vec<Box<dyn Component>>,
    bounds: Bounds,
    direction: FlexDirection,
    align: FlexAlign,
    justify: FlexAlign,
    padding: f32,
    spacing: f32,
    anchor: Anchor,
    offset: ComponentOffset,
    parent_bounds: Option<Bounds>,
    screen_size: (u32, u32),
}

impl Container {
    pub fn new(
        transform: ComponentTransform,
        parent_bounds: Option<Bounds>,
        direction: FlexDirection,
        align: FlexAlign,
        justify: FlexAlign,
    ) -> Self {
        let ComponentTransform {
            size,
            offset,
            anchor,
        } = transform;
        let (x, y) = if let Some(bounds) = parent_bounds {
            let (anchor_x, anchor_y) = bounds.get_anchor_position(anchor);
            (anchor_x + offset.x, anchor_y + offset.y)
        } else {
            (offset.x, offset.y)
        };

        let position = ComponentPosition { x, y };

        Self {
            children: Vec::new(),
            bounds: Bounds::new(position, size),
            direction,
            align,
            justify,
            padding: 0.0,
            spacing: 0.0,
            anchor,
            offset,
            parent_bounds,
            screen_size: (0, 0),
        }
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    fn layout_children(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) {
        if self.children.is_empty() {
            return;
        }

        let content_width = self.bounds.size.width - (self.padding * 2.0);
        let content_height = self.bounds.size.height - (self.padding * 2.0);

        match self.direction {
            FlexDirection::Row => self.layout_row(queue, device, content_width, content_height),
            FlexDirection::Column => {
                self.layout_column(queue, device, content_width, content_height)
            }
        }
    }

    fn layout_row(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, width: f32, height: f32) {
        // Start with the container's left edge plus padding
        let start_x = match self.anchor {
            Anchor::TopLeft | Anchor::Left | Anchor::BottomLeft => {
                self.bounds.position.x + self.padding
            }
            Anchor::Top | Anchor::Center | Anchor::Bottom => {
                self.bounds.position.x + (self.bounds.size.width - width) / 2.0
            }
            Anchor::TopRight | Anchor::Right | Anchor::BottomRight => {
                self.bounds.position.x - width - self.padding
            }
        };

        // Calculate total width of all children
        let total_children_width: f32 = self
            .children
            .iter()
            .map(|child| child.get_bounds().size.width)
            .sum();

        // Calculate total spacing between elements
        let spacing_count = self.children.len().saturating_sub(1) as f32;
        let total_spacing = self.spacing * spacing_count;

        // Calculate the space between items based on justify
        let available_width_space =
            width - total_children_width - total_spacing - (self.padding * 2.0);

        let item_offset = match self.justify {
            FlexAlign::SpaceBetween => {
                if spacing_count > 0.0 {
                    available_width_space / spacing_count
                } else {
                    0.0
                }
            }
            FlexAlign::SpaceAround => available_width_space / (self.children.len() as f32 + 1.0),
            _ => 0.0,
        };

        // Position each child
        let mut current_x = match self.justify {
            FlexAlign::Start => start_x,
            FlexAlign::Center => start_x + available_width_space / 2.0,
            FlexAlign::End => start_x + available_width_space,
            FlexAlign::SpaceAround => start_x + item_offset,
            FlexAlign::SpaceBetween => start_x,
        };

        let available_height_space = height - (self.padding * 2.0);

        for child in &mut self.children {
            let child_bounds = child.get_bounds();
            let child_y = match self.align {
                FlexAlign::Start | FlexAlign::SpaceBetween => self.bounds.position.y + self.padding,
                FlexAlign::Center | FlexAlign::SpaceAround => {
                    self.bounds.position.y
                        + (available_height_space - child_bounds.size.height) / 2.0
                }
                FlexAlign::End => {
                    self.bounds.position.y + self.bounds.size.height
                        - child_bounds.size.height
                        - self.padding
                }
            };

            child.set_position(
                queue,
                device,
                ComponentPosition {
                    x: current_x,
                    y: child_y,
                },
            );

            current_x += child_bounds.size.width + self.spacing + item_offset;
        }
    }

    fn layout_column(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        width: f32,
        height: f32,
    ) {
        let total_spacing = self.spacing * (self.children.len() - 1) as f32;
        let available_height = height - total_spacing;

        let mut y_offset = match self.justify {
            FlexAlign::Start => self.bounds.position.y + self.padding,
            FlexAlign::Center => {
                self.bounds.position.y + (height - available_height - total_spacing) / 2.0
            }
            FlexAlign::End => self.bounds.position.y + height - available_height - total_spacing,
            FlexAlign::SpaceBetween => self.bounds.position.y + self.padding,
            FlexAlign::SpaceAround => self.bounds.position.y + self.padding,
        };

        for child in &mut self.children {
            let child_bounds = child.get_bounds();
            let x = match self.align {
                FlexAlign::Start => self.bounds.position.x + self.padding,
                FlexAlign::Center => {
                    self.bounds.position.x + (width - child_bounds.size.width) / 2.0
                }
                FlexAlign::End => {
                    self.bounds.position.x + width - child_bounds.size.width - self.padding
                }
                FlexAlign::SpaceBetween | FlexAlign::SpaceAround => {
                    self.bounds.position.x + self.padding
                }
            };

            child.set_position(queue, device, ComponentPosition { x, y: y_offset });
            y_offset += child_bounds.size.height + self.spacing;
        }
    }

    fn resize_bounds(&mut self, width: u32, height: u32) {
        if let Some(bounds) = self.parent_bounds {
            // Update parent bounds with new dimensions
            let parent = Bounds::new(
                bounds.position,
                ComponentSize {
                    width: width as f32,
                    height: height as f32,
                },
            );

            // Recalculate container position based on anchor
            let (anchor_x, anchor_y) = parent.get_anchor_position(self.anchor);
            self.bounds.position.x = anchor_x + self.offset.x;
            self.bounds.position.y = anchor_y + self.offset.y;

            // Store updated parent bounds
            self.parent_bounds = Some(parent);
        }
    }
}

impl Component for Container {
    fn update(&mut self, queue: &wgpu::Queue) {
        for child in &mut self.children {
            child.update(queue);
        }
    }

    fn draw<'a>(&'a self, render_pass: &mut dyn RenderPassExt<'a>) {
        for child in &self.children {
            child.draw(render_pass);
        }
    }

    fn resize(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, width: u32, height: u32) {
        self.screen_size = (width, height);
        self.resize_bounds(width, height);
        self.layout_children(queue, device);

        for child in &mut self.children {
            child.resize(queue, device, width, height);
        }
    }

    fn set_position(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        position: ComponentPosition,
    ) {
        self.bounds.position.x = position.x;
        self.bounds.position.y = position.y;
        self.layout_children(queue, device);
    }

    fn handle_mouse_click(&mut self, x: f32, y: f32) -> bool {
        for child in self.children.iter_mut().rev() {
            if child.handle_mouse_click(x, y) {
                return true;
            }
        }
        false
    }

    fn add_child(&mut self, child: Box<dyn Component>) {
        self.children.push(child);
    }

    fn remove_child(&mut self, index: usize) -> Option<Box<dyn Component>> {
        if index < self.children.len() {
            Some(self.children.remove(index))
        } else {
            None
        }
    }

    fn get_children(&self) -> &Vec<Box<dyn Component>> {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn Component>> {
        &mut self.children
    }

    fn get_bounds(&self) -> Bounds {
        self.bounds
    }
}
