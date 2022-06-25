use geometry::{DVec3, Ray, Vec3};

use super::dimensions::Dimensions;

pub struct Camera {
    dimensions: Dimensions,
    pos: DVec3,
    dir: DVec3,
    vertical_fov: f32,
    out: Vec<u8>,
}

impl Camera {
    pub fn new(dimensions: Dimensions, pos: DVec3, dir: DVec3, vertical_fov: f32) -> Self {
        let out = dimensions.get_buffer();
        Self {
            dimensions,
            pos,
            dir,
            vertical_fov,
            out,
        }
    }

    fn get_ray(&self, x: usize, y: usize) -> Ray {
        let y_size = f32::tan(self.vertical_fov * std::f32::consts::PI / 360.0);
        let x_size = y_size * self.dimensions.aspect_ratio();
        let (half_width, half_height) = (
            (self.dimensions.width / 2) as f32,
            (self.dimensions.height / 2) as f32,
        );
        let view_x = x_size * ((x as f32) - half_width) / half_width;
        let view_y = y_size * ((y as f32) - half_height) / half_height;

        let viewport = Vec3::new(view_x, view_y, 1.0);
        Ray::new(
            Vec3::new(self.pos.x as f32, self.pos.y as f32, self.pos.z as f32),
            viewport,
        )
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
