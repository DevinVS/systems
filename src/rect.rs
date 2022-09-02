use std::ops::Add;
use crate::component::PositionComponent;

use crate::graphics::{Texture, Vertex};

/// Rectangle which exists inside the game world
#[derive(Debug, Copy, Clone)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T
}

impl <T> Rect<T> {
    /// Create new rectangle
    pub fn new(x: T, y: T, w: T, h: T) -> Rect<T> {
        Rect { x, y, w, h }
    }
}

impl Rect<i32> {
    /// Create a new rectangle that has the offset of a position component
    pub fn after_position<P: PositionComponent>(mut self, pos: &P) -> Rect<i32> {
        self.x += pos.x() as i32;
        self.y += pos.y() as i32;

        self
    }
}

impl Rect<u32> {
    /// Create a new rectangle that has the offset of a position component
    pub fn after_position<P: PositionComponent>(mut self, pos: &P) -> Rect<u32> {
        self.x += pos.x() as u32;
        self.y += pos.y() as u32;

        self
    }
}

impl Rect<f32> {
    /// Create a new rectangle that has the offset of a position component
    pub fn after_position<P: PositionComponent>(mut self, pos: &P) -> Rect<f32> {
        self.x += pos.x();
        self.y += pos.y();

        self
    }

    /// Create vertices for upload to gpu
    pub fn vertices(&self, tex: &Texture) -> [Vertex; 4] {
        [
            Vertex {
                position: [self.x, self.y],
                tex_coords: tex.nw()
            },
            Vertex {
                position: [self.x + self.w, self.y],
                tex_coords: tex.ne()
            },
            Vertex {
                position: [self.x + self.w, self.y + self.h],
                tex_coords: tex.se()
            },
            Vertex {
                position: [self.x, self.y + self.h],
                tex_coords: tex.sw()
            }
        ]
    }

    pub fn indices(&self, vert_index: u16) -> [u16; 6] {
        [vert_index, vert_index+1, vert_index+2, vert_index+2, vert_index+3, vert_index]
    }
}

impl <T> Rect<T>
where
    T: Add<Output = T> + PartialOrd + Copy
{
    /// Check if this rectangle intersects in any way with another retangle
    pub fn has_intersection(&self, other: &Rect<T>) -> bool {
        let left = self.x;
        let right = self.x + self.w;
        let top = self.y;
        let bottom = self.y + self.h;

        let o_left = other.x;
        let o_right = other.x + other.w;
        let o_top = other.y;
        let o_bottom = other.y + other.h;

        if right <= o_left || o_right <= left {
            return false;
        }

        if top >= o_bottom || o_top >= bottom {
            return false;
        }

        return true;
    }
}
