use glam::DVec3;
use path_integration::Field;

pub struct BlackHole {
    pub field: Field,
}

impl BlackHole {
    pub fn new(radius: f64, camera_pos: &DVec3) -> Self {
        let field = Field::new(radius, &camera_pos);
        Self { field }
    }
}
