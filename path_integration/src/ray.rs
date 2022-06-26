use glam::DVec3;
#[derive(Debug, Clone)]
pub struct Ray {
    pub pos: DVec3,
    pub dir: DVec3,
}

impl Ray {
    pub fn new(pos: DVec3, dir: DVec3) -> Self {
        Self {
            pos,
            dir: dir.normalize(),
        }
    }
}
