use systems::component::PositionComponent;
use systems::component::GraphicsComponent;
use systems::graphics::Atlas;
use systems::graphics::Texture;
use systems::graphics::GraphicsSystem;
use systems::Rect;
use systems::graphics::camera::FixedSizeCamera;

use winit::event_loop::EventLoop;
use winit::event_loop::ControlFlow;
use winit::event::Event;
use winit::event::WindowEvent;

#[derive(Copy, Clone)]
struct Position {
    x: f32,
    y: f32,
}

impl PositionComponent for Position {
    fn x(&self) -> f32 { self.x }
    fn y(&self) -> f32 { self.y }
    fn set_x(&mut self, x: f32) { self.x = x }
    fn set_y(&mut self, y: f32) { self.y = y }
}

#[derive(Copy, Clone)]
struct Graphics {
    tex: Texture,
    renderbox: Rect<f32>
}

impl GraphicsComponent for Graphics {
    fn texture(&self) -> Texture {
        self.tex
    }

    fn renderbox(&self) -> Rect<f32> {
        self.renderbox
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let atlas = Atlas::new("./examples/square/square.atlas");

    let rect_pos = Position { x: 240.0-16.0, y: 0.0 };
    let rect_g = Graphics { tex: atlas.get("square.png").unwrap(), renderbox: Rect::new(0.0, 0.0, 16.0, 16.0) };

    let mut sys = GraphicsSystem::<FixedSizeCamera>::new(
        &event_loop,
        atlas,
        Rect::new(0.0, 0.0, 240.0, 180.0),
        Rect::new(0, 0, 200, 140),
        1.0,
    );

    event_loop.run(move |ev, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match ev {
            Event::WindowEvent { event: WindowEvent::CloseRequested, ..} => {
                std::process::exit(0);
            }
            Event::WindowEvent { event: WindowEvent::Resized(_), ..} => {
                sys.handle_resize();
            }
            Event::MainEventsCleared => {
                sys.transfer(&Vec::<Graphics>::new(), &vec![Some(rect_pos)], &vec![Some(rect_g)], &Vec::<Graphics>::new());
                sys.draw();
            }
            _ => {}
        }
    });
}
