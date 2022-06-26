use glam::DVec3;
use path_integration::Ray;

use super::dimensions::Dimensions;

pub struct Camera {
    dimensions: Dimensions,
    pub pos: DVec3,
    pub vertical_fov: f64,
    out: Vec<u8>,
}

impl Camera {
    pub fn new(dimensions: Dimensions, pos: DVec3, vertical_fov: f64) -> Self {
        let out = dimensions.get_buffer();
        Self {
            dimensions,
            pos,
            vertical_fov,
            out,
        }
    }

    fn get_ray(&self, x: usize, y: usize) -> Ray {
        let y_size = f64::tan(self.vertical_fov * std::f64::consts::PI / 360.0);
        let x_size = y_size * self.dimensions.aspect_ratio();
        let (half_width, half_height) = (
            (self.dimensions.width / 2) as f64,
            (self.dimensions.height / 2) as f64,
        );
        let view_x = x_size * ((x as f64) - half_width) / half_width;
        let view_y = y_size * ((y as f64) - half_height) / half_height;

        let viewport = DVec3::new(view_x, view_y, 1.0);
        Ray::new(DVec3::new(self.pos.x, self.pos.y, self.pos.z), viewport)
    }

    pub fn get_rays(&self) -> Vec<Ray> {
        let mut rays = Vec::new();
        let dimensions = &self.dimensions;
        for y in (0..dimensions.height).rev() {
            for x in 0..dimensions.width {
                rays.push(self.get_ray(x, y));
            }
        }

        rays
    }

    pub fn write_color(&mut self, index: usize, color: &[u8; 4]) {
        let c: &mut [u8] = &mut self.out[(4 * index)..(4 * index + 4)];
        c[0] = color[0];
        c[1] = color[1];
        c[2] = color[2];
        c[3] = color[3];
    }

    pub fn get_colors(&self) -> &[u8] {
        &self.out
    }

    pub fn get_dimensions(&self) -> &Dimensions {
        &self.dimensions
    }
}
