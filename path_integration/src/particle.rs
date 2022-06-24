use geometry::Vec3;

pub struct Particle {
    pub p: Vec3,
    pub v: Vec3,
}

impl Particle {
    pub fn new(p: Vec3, v: Vec3) -> Self {
        Self { p, v }
    }
}
