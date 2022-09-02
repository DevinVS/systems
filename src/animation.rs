use crate::component::{
    GraphicsComponent as Graphics,
    AnimationComponent as Animation,
};

pub struct AnimationSystem {}

impl AnimationSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn animate<G, A>(
        &self,
        g: &mut Vec<Option<G>>,
        a: &mut Vec<Option<A>>,
    ) where
        G: Graphics,
        A: Animation<G>,
    {
        for i in 0..g.len() {
            if g[i].is_none() || a[i].is_none() { continue; }

            let graphics = g[i].as_mut().unwrap();
            let animation = a[i].as_mut().unwrap();

            if animation.finished() { continue; }
            if animation.ready() {
                *graphics = animation.next()
            }
        }
    }
}
