use cgmath::Matrix4;
use crate::rect::Rect;

pub struct Camera {
    pub rect: Rect<i32>,
    pub player_box: Rect<u32>,
    zoom: f32,
    logical_size: (f32, f32),
    physical_size: (u32, u32),
    scaling_factor: f32,
    pixel_size: u32,
    x_offset: f32,
    y_offset: f32
}

impl Camera {
    pub fn new(
        rect: Rect<i32>,
        player_box: Rect<u32>,
        zoom: f32,
        physical_size: (u32, u32),
        logical_size: (f32, f32),
        scaling_factor: f32
    ) -> Camera {
        let mut cam = Camera {
            rect,
            player_box,
            scaling_factor,
            logical_size,
            physical_size,
            zoom,
            pixel_size: 5,
            x_offset: 0.0,
            y_offset: 0.0
        };

        cam.calculate_pixel_width();
        cam
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        // Converts game pixel values into coordinate values
        let width_factor =  self.pixel_size as f32 * self.zoom / self.logical_size.0  as f32 * 2.0;
        let height_factor = self.pixel_size as f32 * self.zoom / self.logical_size.1 as f32 * 2.0;

        // Convert ingame camera position to coordinate position
        let camera_x = width_factor*self.rect.x as f32;
        let camera_y = height_factor*self.rect.y as f32;

        // Convert offset to coordinate value
        let x_offset = self.x_offset / self.logical_size.0 * 2.0;
        let y_offset = self.y_offset / self.logical_size.1 * 2.0;

        // Column Major!
        Matrix4::from_cols(
            [width_factor, 0.0, 0.0, 0.0].into(),
            [0.0, height_factor, 0.0, 0.0].into(),
            [0.0, 0.0, 1.0, 0.0].into(),
            [-camera_x-1.0+x_offset, -camera_y-1.0+y_offset, 0.0, 1.0].into()
        )
    }

    pub fn update_window(&mut self, physical_size: (u32, u32), logical_size: (f32, f32), scaling_factor: f32) {
        self.physical_size = physical_size;
        self.logical_size = logical_size;
        self.scaling_factor = scaling_factor;
        self.calculate_pixel_width();
    }

    pub fn viewport(&self) -> ([u32; 2], [u32; 2]) {
        let origin = [
            self.x_offset as u32 * self.scaling_factor as u32,
            self.y_offset as u32 * self.scaling_factor as u32
        ];
        let dimensions = [
            self.rect.w as u32 * self.pixel_size * self.scaling_factor as u32,
            self.rect.h as u32 * self.pixel_size * self.scaling_factor as u32
        ];

        (origin, dimensions)
    }

    fn calculate_pixel_width(&mut self) {
        self.pixel_size = 5;

        while self.logical_size.0 < (self.rect.w as u32*self.pixel_size) as f32 ||
            self.logical_size.1 < (self.rect.h as u32*self.pixel_size) as f32
        {
            self.pixel_size-=1;
        }

        self.x_offset = (self.logical_size.0 - (self.rect.w as u32*self.pixel_size) as f32) / 2.0;
        self.y_offset = (self.logical_size.1 - (self.rect.h as u32*self.pixel_size) as f32) / 2.0;
    }

    pub fn pan_to(&mut self, rect: &Rect<f32>) {
        // Camera dimensions in pixels
        let cam_width = self.rect.w;
        let cam_height = self.rect.h;
    
        // Focus area dimensions in pixels
        let focus_width = cam_width / 3;
        let focus_height = cam_height / 3;

        // Offset of the focus area from the camera edge
        let x_offset = (cam_width - focus_width) / 2;
        let y_offset = (cam_height - focus_height) / 2;

        // Sides of the focus area
        let left = x_offset as f32;
        let right = (cam_width - x_offset) as f32;
        let top = (y_offset) as f32;
        let bottom = (cam_height - y_offset) as f32;

        // Sides of the rectangle
        let rect_left = rect.x - self.rect.x as f32;
        let rect_right = rect_left + rect.w as f32;
        let rect_top = rect.y - self.rect.y as f32;
        let rect_bottom = rect_top + rect.h as f32;

        if rect_left < left {
            self.rect.x -= (left - rect_left) as i32;
        }

        if rect_right > right {
            self.rect.x += (rect_right - right) as i32;
        }

        if rect_top < top {
            self.rect.y -= (top - rect_top) as i32;
        }

        if rect_bottom > bottom {
            self.rect.y += (rect_bottom - bottom) as i32;
        }
    }
}
