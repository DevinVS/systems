use crate::rect::Rect;
use crate::graphics::Texture;

pub trait PositionComponent {
    fn x(&self) -> f32;
    fn y(&self) -> f32;

    fn set_x(&mut self, x: f32);
    fn set_y(&mut self, y: f32);
}

pub trait VelocityComponent {
    fn x(&self) -> f32;
    fn y(&self) -> f32;

    fn set_x(&mut self, x: f32);
    fn set_y(&mut self, y: f32,);
}

pub trait PhysicsComponent {
    fn hitbox(&self) -> Rect<f32>;
    fn set_x_collision(&mut self, with: Option<Rect<f32>>);
    fn set_y_collision(&mut self, with: Option<Rect<f32>>);
}

pub trait GraphicsComponent {
    fn texture(&self) -> Texture;
    fn renderbox(&self) -> Rect<f32>;
}

pub trait AnimationComponent<G: GraphicsComponent> {
    fn next(&mut self) -> G;
    fn ready(&self) -> bool;
    fn finished(&self) -> bool { false }
}
