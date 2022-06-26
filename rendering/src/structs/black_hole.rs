use glam::DVec3;
use path_integration::cast_ray_steps;
use path_integration::Field;
use path_integration::Ray;

pub struct BlackHole {
    pub field: Field,
}

impl BlackHole {
    pub fn new_zero(radius: f64, camera_pos: &DVec3) -> Self {
        let field = Field::zero_new(radius, &camera_pos);
        Self { field }
    }

    pub fn new(pos: DVec3, radius: f64) -> Self {
        let field = Field::new(pos, radius, &DVec3::ZERO);
        Self { field }
    }

    pub fn raycast(&self, ray: &Ray) -> Option<DVec3> {
        cast_ray_steps(ray, &self.field, 40.0)
    }
}
