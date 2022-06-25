use geometry::{DVec3, Vec3};

pub struct Particle {
    pub p: DVec3,
    pub v: DVec3,
}

impl Particle {
    pub fn new(p: Vec3, v: Vec3) -> Self {
        Self {
            p: DVec3::new(p.x as f64, p.y as f64, p.z as f64),
            v: DVec3::new(v.x as f64, v.y as f64, v.z as f64),
        }
    }
}
