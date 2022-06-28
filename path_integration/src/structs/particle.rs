use glam::DVec3;

use crate::{Field, Ray};

pub struct Particle {
    pub p: DVec3,
    pub v: DVec3,
}

impl Particle {
    // We use the ray/field here to calculate initial velocity, such that |v| = 1 at the Schwarzchild radius.
    // This ends up allowing us to avoid any
    pub fn new(ray: &Ray, field: &Field) -> Self {
        let p = DVec3::new(ray.pos.x as f64, ray.pos.y as f64, ray.pos.z as f64);
        let v = field.initial_speed(&p)
            * DVec3::new(ray.dir.x as f64, ray.dir.y as f64, ray.dir.z as f64);
        Self { p, v }
    }
}
