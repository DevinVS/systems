use std::time::Instant;

use crate::rect::Rect;
use crate::component::{
    PositionComponent as Position,
    VelocityComponent as Velocity,
    PhysicsComponent as Physics,
};

pub trait CollisionMap {
    fn test(&self, rect: &Rect<f32>) -> bool;
}

pub struct PhysicsSystem {
    last_tick: Instant
}

pub struct Dummy {}
impl CollisionMap for Dummy {
    fn test(&self, _: &Rect<f32>) -> bool { false }
}

impl PhysicsSystem {
    pub fn new() -> PhysicsSystem {
        PhysicsSystem {
            last_tick: Instant::now()
        }
    }

    pub fn velocity<P: Position, V: Velocity>(
        &mut self,
        p: &mut Vec<Option<P>>,
        v: &Vec<Option<V>>
    ) {
        let t = self.last_tick.elapsed().as_secs_f32();
        self.last_tick = Instant::now();
        Self::apply_velocity(p, v, t);
    }

    pub fn collision<P, V, PH>(
        &mut self,
        p: &mut Vec<Option<P>>,
        v: &mut Vec<Option<V>>,
        ph: &mut Vec<Option<PH>>
    )
    where
        P: Position,
        V: Velocity,
        PH: Physics,
    {
        let t = self.last_tick.elapsed().as_secs_f32();
        let map = Dummy{};
        self.last_tick = Instant::now();
        Self::apply_collision(p, v, ph, Some(&map), t);
        Self::apply_velocity(p, v, t);
    }


    pub fn collision_map<P, V, PH, CM>(
        &mut self,
        p: &mut Vec<Option<P>>,
        v: &mut Vec<Option<V>>,
        ph: &mut Vec<Option<PH>>,
        map: Option<&CM>
        )
    where
        P: Position,
        V: Velocity,
        PH: Physics,
        CM: CollisionMap,
    {
        let t = self.last_tick.elapsed().as_secs_f32();
        self.last_tick = Instant::now();
        Self::apply_collision(p, v, ph, map, t);
        Self::apply_velocity(p, v, t);
    }

    fn apply_velocity<P: Position, V: Velocity>(
        p: &mut Vec<Option<P>>,
        v: &Vec<Option<V>>,
        t: f32,
    ) {

        for i in 0..p.len() {
            if p[i].is_none() { continue; }
            if v[i].is_none() { continue; }

            let pos = p[i].as_mut().unwrap();
            let vel = v[i].as_ref().unwrap();

            pos.set_x(pos.x() + vel.x() * t);
            pos.set_y(pos.y() + vel.y() * t);
        }
    }

    fn apply_collision<P, V, PH, CM>(
        p: &mut Vec<Option<P>>,
        v: &mut Vec<Option<V>>,
        ph: &mut Vec<Option<PH>>,
        map: Option<&CM>,
        t: f32,
    )
    where
        P: Position,
        V: Velocity,
        PH: Physics,
        CM: CollisionMap,
    {
        for i in 0..p.len() {
            if p[i].is_none() || v[i].is_none() || ph[i].is_none() { continue; }

            let vel = v[i].as_mut().unwrap();

            let (irect, mut after_x, mut after_y) = {
                let phy = ph[i].as_ref().unwrap();
                let pos = p[i].as_mut().unwrap();
                let irect = phy.hitbox().after_position(pos);
                let mut after_x = phy.hitbox().after_position(pos);
                let mut after_y = phy.hitbox().after_position(pos);

                after_x.w += vel.x().abs() * t;
                after_y.h += vel.y().abs() * t;

                if vel.x() <= 0.0 {
                    after_x.x -= vel.x().abs() * t;
                }

                if vel.y() <= 0.0 {
                    after_y.y -= vel.y().abs() * t;
                }

                (irect, after_x, after_y)
            };

            let (x_delta, x_coll, y_delta, y_coll) = Self::handle_collision(p, ph, map, i, vel, &irect, &mut after_x, &mut after_y);

            let pos = p[i].as_mut().unwrap();
            let phy = ph[i].as_mut().unwrap();
            pos.set_x(pos.x() + x_delta.unwrap_or(0.0));
            pos.set_y(pos.y() + y_delta.unwrap_or(0.0));
            phy.set_x_collision(x_coll);
            phy.set_y_collision(y_coll);
        }
    }

    fn handle_collision<P, V, PH, CM>(
        p: &Vec<Option<P>>,
        ph: &Vec<Option<PH>>,
        map: Option<&CM>,
        i: usize,
        vel: &mut V,
        irect: &Rect<f32>,
        after_x: &mut Rect<f32>,
        after_y: &mut Rect<f32>,
    ) -> (Option<f32>, Option<Rect<f32>>, Option<f32>, Option<Rect<f32>>)
    where
        P: Position,
        V: Velocity,
        PH: Physics,
        CM: CollisionMap,
    {

        let mut x_delta: Option<f32> = None;
        let mut x_coll: Option<Rect<f32>> = None;
        let mut y_delta: Option<f32> = None;
        let mut y_coll: Option<Rect<f32>> = None;

        // Check map collisions if applicable
        if let Some(map) = map {
            // For each axis do a binary search sort of thing to figure out
            // how much the rect is allowed to move
            if map.test(after_x) {
                vel.set_x(0.0);
            }

            if map.test(after_y) {
                vel.set_y(0.0);
            }
        }

        // For every other entity, check whether the new hitbox after x and y components
        // of the velocity intersects
        for j in 0..p.len() {
            if i==j || p[j].is_none() || ph[j].is_none() { continue; }

            let jrect = ph[j].as_ref().unwrap().hitbox()
                .after_position(p[j].as_ref().unwrap());

            if jrect.has_intersection(&after_x) {
                // Distance from edge of irect to edge of jrect
                let dist = if vel.x().signum() >= 0.0 {
                     jrect.x - (irect.x + irect.w)
                } else {
                     (jrect.x + jrect.w) - irect.x
                };
                
                // Store smallest distance (magnitude)
                if let Some(d) = x_delta {
                    if dist.abs() < d.abs() {
                        x_delta = Some(dist);
                        x_coll = Some(jrect);
                    }
                } else {
                    x_delta = Some(dist);
                    x_coll = Some(jrect);
                }

                // Zero out velocity
                vel.set_x(0.0);
            }

            if jrect.has_intersection(&after_y) {
                // Distance from edge of irect to edge of jrect
                let dist = if vel.y().signum() >= 0.0 {
                    jrect.y - (irect.y + irect.h)
                } else {
                    (jrect.y + jrect.h) - irect.y
                };

                // Store smallest distance
                if let Some(d) = y_delta {
                    if dist.abs() < d.abs() {
                        y_delta = Some(dist);
                        y_coll = Some(jrect);
                    }
                } else {
                    y_delta = Some(dist);
                    y_coll = Some(jrect);
                }

                // Zero out velocity
                vel.set_y(0.0);
            }
        }

        (x_delta, x_coll, y_delta, y_coll)
    }
}
