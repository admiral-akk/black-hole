use crate::{Ray, Vector3};

pub struct Disc {
    pos: Vector3,
    up: Vector3,
    outer_rad: f32,
    inner_rad: f32,
}

impl Disc {
    pub fn new(pos: Vector3, up: Vector3, outer_rad: f32, inner_rad: f32) -> Self {
        Self {
            pos,
            up,
            outer_rad,
            inner_rad,
        }
    }

    pub fn is_hit(&self, ray: &Ray) -> bool {
        let up_dist = (self.pos - ray.pos).dot(self.up);
        let plane_point = ray.pos + (up_dist * ray.dir);
        let plane_dist = (self.pos - plane_point).length();

        plane_dist >= self.inner_rad && plane_dist <= self.outer_rad
    }
}
