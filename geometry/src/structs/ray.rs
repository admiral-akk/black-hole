use super::{unit_vec3::UnitVec3, vec3::Vec3};

pub struct Ray {
    pub pos: Vec3,
    pub dir: UnitVec3,
}

impl Ray {
    pub fn new(pos: Vec3, dir: UnitVec3) -> Self {
        Self { pos, dir }
    }

    pub fn dir(&self) -> &Vec3 {
        self.dir.vec3()
    }
}
