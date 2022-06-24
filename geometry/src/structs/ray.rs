use crate::Vector3;

pub struct Ray {
    pub pos: Vector3,
    pub dir: Vector3,
}

impl Ray {
    pub fn new(pos: Vector3, dir: Vector3) -> Self {
        Self { pos, dir }
    }
}
