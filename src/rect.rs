use crate::component::PositionComponent;

/// Rectangle which exists inside the game world
#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect {x, y, w, h}
    }

    /// Check if this rectangle intersects in any way with another retangle
    pub fn has_intersection(&self, other: &Rect) -> bool {
        let left = self.x;
        let right = self.x + self.w as f32;
        let top = self.y;
        let bottom = self.y + self.h as f32;

        let o_left = other.x;
        let o_right = other.x + other.w as f32;
        let o_top = other.y;
        let o_bottom = other.y + other.h as f32;

        if right <= o_left || o_right <= left {
            return false;
        }

        if top >= o_bottom || o_top >= bottom {
            return false;
        }

        return true;
    }

    /// Create a new rectangle that has the offset of a position component
    pub fn after_position<P: PositionComponent>(mut self, pos: &P) -> Rect {
        self.x += pos.x();
        self.y += pos.y();

        self
    }
}
