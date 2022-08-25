use std::f32::consts::TAU;

use glam::Vec2;

#[derive(Clone, Copy)]
pub struct RenderParams {
    pub black_hole_radius: f32,
    pub fov_degrees: f32,
}

impl RenderParams {
    pub fn view_coord_to_vec(&self, coord: f32) -> Vec2 {
        let view_width = (self.fov_degrees * TAU / 360.).tan();
        Vec2::new(view_width * coord, 1.).normalize()
    }
}
