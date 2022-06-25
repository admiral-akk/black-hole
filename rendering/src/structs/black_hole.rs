use geometry::DVec3;
use geometry::Ray;
use path_integration::cast_ray_steps;
use path_integration::Field;

pub struct BlackHole {
    field: Field,
}

impl BlackHole {
    pub fn new(pos: DVec3, radius: f64) -> Self {
        let field = Field::new(pos, radius, &DVec3::ZERO);
        Self { field }
    }

    pub fn raycast(&self, ray: &Ray) -> Option<DVec3> {
        cast_ray_steps(ray, &self.field, 40.0)
    }
}