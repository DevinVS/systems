use crate::rect::Rect;

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
    fn hitbox(&self) -> &Rect;
}

pub trait DepthComponent {
    fn depth(&self) -> f32;
}

pub trait GraphicsComponent {
    fn texture(&self) -> f32;
    fn renderbox(&self) -> &Rect;
    fn srcbox(&self) -> Option<(u32, u32, u32, u32)> { None }
}

pub trait AnimationComponent {
    fn next(&mut self) -> dyn GraphicsComponent;
    fn ready(&self) -> bool;
    fn finished(&self) -> bool { false }
}
