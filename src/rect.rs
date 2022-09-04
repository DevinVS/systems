use std::ops::Add;
use crate::component::PositionComponent;
use crate::graphics::Texture;
use crate::graphics::vulkan::Vertex;
use crate::graphics::Atlas;

pub trait IntoF32 {
    fn to_f32(self) -> f32;
}

impl IntoF32 for i32 {
    fn to_f32(self) -> f32 { self as f32 }
}

impl IntoF32 for u32 {
    fn to_f32(self) -> f32 { self as f32 }
}

impl IntoF32 for f32 {
    fn to_f32(self) -> f32 { self as f32 }
}

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

impl<T> Rect<T>
where T: IntoF32
{
    /// Create a new rectangle that has the offset of a position component
    pub fn after_position<P: PositionComponent>(self, pos: &P) -> Rect<f32> {
        Rect {
            x: self.x.to_f32() + pos.x(),
            y: self.y.to_f32() + pos.y(),
            w: self.w.to_f32(),
            h: self.h.to_f32()
        }
    }
}
impl Rect<f32> {
    /// Create vertices for upload to gpu
    pub fn vertices(&self, tex: &Texture, atlas: &Atlas) -> [Vertex; 4] {
        let nw = tex.nw();
        let ne = tex.ne();
        let sw = tex.sw();
        let se = tex.se();

        [
            Vertex {
                position: [self.x, self.y],
                tex_coords: [
                    nw[0] as f32 / atlas.width as f32,
                    nw[1] as f32 / atlas.height as f32
                ]
            },
            Vertex {
                position: [self.x + self.w, self.y],
                tex_coords: [
                    ne[0] as f32 / atlas.width as f32,
                    ne[1] as f32 / atlas.height as f32
                ]
            },
            Vertex {
                position: [self.x + self.w, self.y + self.h],
                tex_coords: [
                    se[0] as f32 / atlas.width as f32,
                    se[1] as f32 / atlas.height as f32
                ]
            },
            Vertex {
                position: [self.x, self.y + self.h],
                tex_coords: [
                    sw[0] as f32 / atlas.width as f32,
                    sw[1] as f32 / atlas.height as f32
                ]
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
