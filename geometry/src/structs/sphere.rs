use crate::{Ray, Vec3};

pub struct Sphere {
    pos: Vec3,
    rad: f32,
}

impl Sphere {
    pub fn new(pos: Vec3, rad: f32) -> Self {
        Self { pos, rad }
    }

    pub fn is_hit(&self, ray: &Ray) -> bool {
        let diff = &self.pos - &ray.pos;
        let dir = ray.dir.vec3();
        let off = diff.dot(ray.dir.vec3());
        let orthogonal = &diff - &(off * dir);
        orthogonal.dot(&orthogonal) <= self.rad
    }
}
