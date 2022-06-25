use crate::{Ray, Vector3};

pub struct Sphere {
    pos: Vector3,
    rad: f32,
}

impl Sphere {
    pub fn new(pos: Vector3, rad: f32) -> Self {
        Self { pos, rad }
    }

    pub fn is_hit(&self, ray: &Ray) -> bool {
        let diff = self.pos - ray.pos;
        let dir = &ray.dir;
        let off = diff.dot(ray.dir);
        let orthogonal = diff - (off * *dir);
        orthogonal.dot(orthogonal) <= self.rad
    }
}