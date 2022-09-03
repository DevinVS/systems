use cgmath::Matrix4;
use crate::rect::Rect;

pub trait Camera {
    fn new(rect: Rect<f32>, player_box: Rect<u32>, zoom: f32, physical_size: (u32, u32), scale_factor: f32) -> Self where Self: Sized;
    fn matrix(&self) -> Matrix4<f32>;
    fn handle_resize(&mut self, physical_size: (u32, u32), scale_factor: f32);
    fn viewport(&self) -> ([u32; 2], [u32; 2]);
    fn pan_to(&mut self, rect: &Rect<f32>);
}

#[derive(Debug)]
pub struct FixedHeightCamera {
    pub rect: Rect<f32>,
    pub player_box: Rect<u32>,
    zoom: f32,
    physical_size: (u32, u32),
    logical_size: (f32, f32),
    scale_factor: f32,
    pixel_size: f32,
}

impl FixedHeightCamera {

    fn calculate_pixel_width(&mut self) {
        self.pixel_size = self.physical_size.1 as f32 / (self.scale_factor * self.rect.h as f32);
        self.rect.w = self.pixel_size * self.physical_size.0 as f32;
    }
}

impl Camera for FixedHeightCamera {
    fn new(rect: Rect<f32>, player_box: Rect<u32>, zoom: f32, physical_size: (u32, u32), scale_factor: f32) -> Self {
        let logical_size = (physical_size.0 as f32 / scale_factor, physical_size.1 as f32 / scale_factor);
        let mut cam = Self {
            rect,
            player_box,
            zoom,
            physical_size,
            logical_size,
            pixel_size: 0.0,
            scale_factor
        };
        
        cam.calculate_pixel_width();
        cam
    }

    fn matrix(&self) -> Matrix4<f32> {
        // Converts game pixel values into coordinate values
        let width_factor =  self.pixel_size * self.zoom / self.logical_size.0  as f32 * 2.0;
        let height_factor = self.pixel_size * self.zoom / self.logical_size.1 as f32 * 2.0;

        // Convert ingame camera position to coordinate position
        let camera_x = width_factor*self.rect.x;
        let camera_y = height_factor*self.rect.y;

        // Column Major!
        Matrix4::from_cols(
            [width_factor, 0.0, 0.0, 0.0].into(),
            [0.0, height_factor, 0.0, 0.0].into(),
            [0.0, 0.0, 1.0, 0.0].into(),
            [-camera_x-1.0, -camera_y-1.0, 0.0, 1.0].into()
        )
    }

    fn handle_resize(&mut self, physical_size: (u32, u32), scale_factor: f32) {
        self.physical_size = physical_size;
        self.logical_size = (physical_size.0 as f32 / scale_factor, physical_size.1 as f32 / scale_factor);
        self.scale_factor = scale_factor;
        self.calculate_pixel_width();
    }

    fn viewport(&self) -> ([u32; 2], [u32; 2]) {
        let origin = [0, 0];
        let dimensions = [
            (self.rect.w as f32 * self.pixel_size * self.scale_factor) as u32,
            (self.rect.h as f32 * self.pixel_size * self.scale_factor) as u32
        ];

        (origin, dimensions)
    }


    fn pan_to(&mut self, rect: &Rect<f32>) {
        let left = self.rect.x as f32 + self.player_box.x as f32;
        let right = left + self.player_box.w as f32;

        // Sides of the rectangle
        let rect_left = rect.x;
        let rect_right = rect_left + rect.w;

        if rect_left < left {
            self.rect.x -= left - rect_left;
        }

        if rect_right > right {
            self.rect.x += rect_right - right;
        }
    }
}

#[derive(Debug)]
pub struct FixedSizeCamera {
    pub rect: Rect<f32>,
    pub player_box: Rect<u32>,
    zoom: f32,
    logical_size: (f32, f32),
    physical_size: (u32, u32),
    scale_factor: f32,
    pixel_size: u32,
    x_offset: f32,
    y_offset: f32
}

impl FixedSizeCamera {
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
}

impl Camera for FixedSizeCamera {
    fn new(
        rect: Rect<f32>,
        player_box: Rect<u32>,
        zoom: f32,
        physical_size: (u32, u32),
        scale_factor: f32
    ) -> FixedSizeCamera {
        let logical_size = (physical_size.0 as f32 / scale_factor, physical_size.1 as f32 / scale_factor);

        let mut cam = FixedSizeCamera {
            rect,
            player_box,
            scale_factor,
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

    fn matrix(&self) -> Matrix4<f32> {
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

    fn handle_resize(&mut self, physical_size: (u32, u32), scale_factor: f32) {
        self.physical_size = physical_size;
        self.logical_size = (physical_size.0 as f32 / scale_factor, physical_size.1 as f32 / scale_factor);
        self.scale_factor = scale_factor;
        self.calculate_pixel_width();
    }

    fn viewport(&self) -> ([u32; 2], [u32; 2]) {
        let origin = [
            self.x_offset as u32 * self.scale_factor as u32,
            self.y_offset as u32 * self.scale_factor as u32
        ];
        let dimensions = [
            self.rect.w as u32 * self.pixel_size * self.scale_factor as u32,
            self.rect.h as u32 * self.pixel_size * self.scale_factor as u32
        ];

        (origin, dimensions)
    }

    fn pan_to(&mut self, rect: &Rect<f32>) {
        let left = self.rect.x + self.player_box.x as f32;
        let right = left + self.player_box.w as f32;
        let top = self.rect.y + self.player_box.y as f32;
        let bottom = top + self.player_box.h as f32;

        // Sides of the rectangle
        let rect_left = rect.x;
        let rect_right = rect_left + rect.w;
        let rect_top = rect.y;
        let rect_bottom = rect_top + rect.h;

        if rect_left < left {
            self.rect.x -= left - rect_left;
        }

        if rect_right > right {
            self.rect.x += rect_right - right;
        }

        if rect_top < top {
            self.rect.y -= top - rect_top;
        }

        if rect_bottom > bottom {
            self.rect.y += rect_bottom - bottom;
        }
    }
}
