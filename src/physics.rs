use std::time::Instant;

use crate::rect::Rect;
use crate::component::{
    PositionComponent as Position,
    VelocityComponent as Velocity,
    PhysicsComponent as Physics,
};

pub struct PhysicsSystem {
    last_tick: Instant
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
        ph: &Vec<Option<PH>>,
        )
    where
        P: Position,
        V: Velocity,
        PH: Physics
    {
        let t = self.last_tick.elapsed().as_secs_f32();
        self.last_tick = Instant::now();
        Self::apply_collision(p, v, ph, t);
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

    fn apply_collision<P, V, PH>(
        p: &mut Vec<Option<P>>,
        v: &mut Vec<Option<V>>,
        ph: &Vec<Option<PH>>,
        t: f32,
    )
    where
        P: Position,
        V: Velocity,
        PH: Physics,
    {
        for i in 0..p.len() {
            if p[i].is_none() || v[i].is_none() || ph[i].is_none() { continue; }

            let phy = ph[i].as_ref().unwrap();
            let vel = v[i].as_mut().unwrap();

            let (irect, after_x, after_y) = {
                let pos = p[i].as_mut().unwrap();
                let irect = phy.hitbox().after_position(pos);
                let mut after_x = phy.hitbox().after_position(pos);
                let mut after_y = phy.hitbox().after_position(pos);

                after_x.w += vel.x() * t;
                after_y.h += vel.y() * t;

                (irect, after_x, after_y)
            };

            let (x_delta, y_delta) = Self::handle_collision(p, ph, i, vel, &irect, &after_x, &after_y);

            let pos = p[i].as_mut().unwrap();
            pos.set_x(pos.x() + x_delta.unwrap_or(0.0));
            pos.set_y(pos.y() + y_delta.unwrap_or(0.0));
        }
    }

    fn handle_collision<P, V, PH>(
        p: &Vec<Option<P>>,
        ph: &Vec<Option<PH>>,
        i: usize,
        vel: &mut V,
        irect: &Rect,
        after_x: &Rect,
        after_y: &Rect,
        ) -> (Option<f32>, Option<f32>)
    where
        P: Position,
        V: Velocity,
        PH: Physics
    {

        let mut x_delta: Option<f32> = None;
        let mut y_delta: Option<f32> = None;

        // For every other entity, check whether the new hitbox after x and y components
        // of the velocity intersects
        for j in 0..p.len() {
            if p[i].is_none() || ph[i].is_none() { continue; }

            let jrect = ph[j].as_ref().unwrap().hitbox().after_position(p[j].as_ref().unwrap());

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
                    }
                } else {
                    x_delta = Some(dist);
                }

                // Zero out velocity
                vel.set_x(0.0);
            }

            if jrect.has_intersection(&after_y) {
                // Distance from edge of irect to edge of jrect
                let dist = if vel.y().signum() >= 0.0 {
                    jrect.y - (jrect.y + jrect.h)
                } else {
                    (jrect.y + jrect.h) - irect.y
                };

                // Store smallest distance
                if let Some(d) = y_delta {
                    if dist.abs() < d.abs() {
                        y_delta = Some(dist);
                    }
                } else {
                    y_delta = Some(dist);
                }

                // Zero out velocity
                vel.set_y(0.0);
            }
        }

        (x_delta, y_delta)
    }
}
