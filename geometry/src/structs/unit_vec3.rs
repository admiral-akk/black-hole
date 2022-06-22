use super::vec3::Vec3;

pub struct UnitVec3 {
    vec: Vec3,
}

impl UnitVec3 {
    pub fn new(vec: Vec3) -> Self {
        Self { vec }
    }

    pub fn xyz(&self) -> (f32, f32, f32) {
        self.vec.xyz()
    }
}
