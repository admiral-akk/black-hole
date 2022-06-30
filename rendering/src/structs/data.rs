use glam::{Vec3, Vec3Swizzles};

use super::polar_coordinates::PolarCoordinates;

#[derive(Clone)]
pub struct Data {
    index: usize,
    vec: Vec3,
}

impl Data {
    pub const fn new(index: usize) -> Data {
        Data {
            index,
            vec: Vec3::ZERO,
        }
    }

    pub fn set_sample(&mut self, index: usize, view_x: f32, view_y: f32) {
        self.index = index;
        self.vec.x = view_x;
        self.vec.y = view_y;
    }

    pub fn get_sample(&self) -> (f32, f32) {
        (self.vec.x, self.vec.y)
    }

    pub fn set_start_dir(&mut self, vec: &Vec3) {
        self.vec = *vec;
    }

    pub fn get_start_dir(&self) -> &Vec3 {
        &self.vec
    }

    pub fn set_polar(&mut self, polar: &PolarCoordinates) {
        self.vec.x = polar.phi;
        self.vec.y = polar.theta;
    }

    pub fn get_polar(&self) -> (f32, f32) {
        (self.vec.x, self.vec.y)
    }

    pub fn set_color(&mut self, c: &[u8; 4]) {
        self.vec.x = c[0] as f32;
        self.vec.y = c[1] as f32;
        self.vec.z = c[2] as f32;
    }

    pub fn get_result(&self) -> (usize, f32, f32, f32) {
        (self.index, self.vec.x, self.vec.y, self.vec.z)
    }
}

pub const DEFAULT_DATA: Data = Data::new(0);
