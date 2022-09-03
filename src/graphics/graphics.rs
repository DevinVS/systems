use crate::graphics::vulkan::VulkanState;
use winit::event_loop::EventLoop;
use super::atlas::Atlas;
use crate::component::{GraphicsComponent, PositionComponent, PhysicsComponent};
use crate::graphics::Camera;
use crate::rect::Rect;
use super::atlas::Texture;

struct ZeroPositionComponent {}
impl PositionComponent for ZeroPositionComponent {
    fn x(&self) -> f32 { 0.0 }
    fn y(&self) -> f32 { 0.0 }
    fn set_x(&mut self, _: f32) {}
    fn set_y(&mut self, _: f32) {}
}

pub struct GraphicsSystem<C: Camera> {
    vulkan_state: VulkanState,
    atlas: Atlas,
    pub camera: C,
}

impl<C: Camera> GraphicsSystem<C> {
    pub fn new(
        event_loop: &EventLoop<()>,
        atlas: Atlas,
        cam_rect: Rect<f32>,
        player_box: Rect<u32>,
        zoom: f32
    ) -> GraphicsSystem<C> {
        let vulkan_state = VulkanState::new::<C>(event_loop, &atlas);
        let camera = C::new(
            cam_rect,
            player_box,
            zoom,
            vulkan_state.physical_size(),
            vulkan_state.scale_factor()
        );

        Self {
            vulkan_state,
            atlas,
            camera,
        }
    }

    pub fn handle_resize(&mut self) {
        self.vulkan_state.recreate_swapchain = true;
    }

    pub fn get_texture(&self, path: &str) -> Option<Texture> {
        self.atlas.get(path)
    }

    pub fn transfer<P, G, GH, GPH>(
        &mut self,
        bgs: &Vec<GH>,
        p: &Vec<Option<P>>, g: &Vec<Option<G>>,
        overlays: &Vec<GPH>
    )
    where
        P: PositionComponent,
        G: GraphicsComponent,
        GH: GraphicsComponent,
        GPH: GraphicsComponent,
    {
        let mut rects = Vec::with_capacity(p.len());
        let mut vertices = Vec::with_capacity(p.len() * 4);
        let mut indices = Vec::with_capacity(p.len() * 6);

        // Create vec of all entity rectangles
        rects.extend(
            (0..p.len())
                .filter(|i| p[*i].is_some() && g[*i].is_some())
                .map(|i| {
                    let pos = p[i].as_ref().unwrap();
                    (i, g[i].as_ref().unwrap().renderbox().after_position(pos))
                })
        );

        // Sort by bottom of rectangle
        rects.sort_unstable_by(|a, b| (a.1.y+a.1.h).partial_cmp(&(b.1.y+b.1.h)).unwrap());

        let mut vert_index = 0;

        // Add backgrounds
        for bg in bgs {
            let bg_rect = bg.renderbox().after_position(&ZeroPositionComponent{});
            let bg_tex = bg.texture();
            vertices.extend_from_slice(&bg_rect.vertices(&bg_tex));
            indices.extend_from_slice(&bg_rect.indices(vert_index));
            vert_index += 4;
        }

        // Add ordinary objects
        for i in 0..rects.len() {
            vertices.extend_from_slice(&rects[i].1.vertices(&g[rects[i].0].as_ref().unwrap().texture()));
            indices.extend_from_slice(&rects[i].1.indices(vert_index));
            vert_index += 4;
        }

        // Add overlays
        for ov in overlays {
            let ov_rect = ov.renderbox().after_position(&ZeroPositionComponent {});
            let ov_tex = ov.texture();
            vertices.extend_from_slice(&ov_rect.vertices(&ov_tex));
            indices.extend_from_slice(&ov_rect.indices(vert_index));
            vert_index += 4;
        }

        self.vulkan_state.transfer_object_data(vertices, indices);
    }

    pub fn draw(&mut self) {
        self.vulkan_state.draw(&mut self.camera);
    }

    pub fn pan_to<P: PositionComponent, PH: PhysicsComponent> (&mut self, p: &P, ph: Option<&PH>) {
        if let Some(ph) = ph {
            self.camera.pan_to(&ph.hitbox().after_position(p))
        } else {
            self.camera.pan_to(&Rect {
                x: p.x(),
                y: p.y(),
                w: 0.0,
                h: 0.0
            });
        }
    }
}
