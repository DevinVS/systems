use crate::component::PositionComponent;

use std::ops::Add;

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
    #[cfg(feature = "sdl2")]
    pub fn new(x: T, y: T, w: T, h: T) -> Rect<T> {
        Rect { x, y, w, h }
    }
}

impl Rect<i32> {
    /// Turn into sdl2 rectangle
    #[cfg(feature = "sdl2")]
    pub fn sdl2(&self) -> sdl2::rect::Rect {
        sdl2::rect::Rect::new(
            self.x,
            self.y,
            self.w as u32,
            self.h as u32
        )
    }

    /// Create a new rectangle that has the offset of a position component
    pub fn after_position<P: PositionComponent>(mut self, pos: &P) -> Rect<i32> {
        self.x += pos.x() as i32;
        self.y += pos.y() as i32;

        self
    }
}

impl Rect<u32> {
    /// Turn into sdl2 rectangle
    #[cfg(feature = "sdl2")]
    pub fn sdl2(&self) -> sdl2::rect::Rect {
        sdl2::rect::Rect::new(
            self.x as i32,
            self.y as i32,
            self.w,
            self.h
        )
    }

    /// Create a new rectangle that has the offset of a position component
    pub fn after_position<P: PositionComponent>(mut self, pos: &P) -> Rect<u32> {
        self.x += pos.x() as u32;
        self.y += pos.y() as u32;

        self
    }
}

impl Rect<f32> {
    /// Turn into sdl2 rectangle
    #[cfg(feature = "sdl2")]
    pub fn sdl2(&self) -> sdl2::rect::Rect {
        sdl2::rect::Rect::new(self.x as i32, self.y as i32, self.w as u32, self.h as u32)
    }

    /// Create a new rectangle that has the offset of a position component
    pub fn after_position<P: PositionComponent>(mut self, pos: &P) -> Rect<f32> {
        self.x += pos.x();
        self.y += pos.y();

        self
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
