use glam::DVec3;

use crate::{Field, Ray, RayCache};

pub struct BlackHole {
    field: Field,
    cache: RayCache,
}

impl BlackHole {
    pub fn new(radius: f64, camera_pos: &DVec3, fov_radians: f64) -> Self {
        let field = Field::new(radius, camera_pos);
        let cache = RayCache::compute_new(10000, &field, camera_pos, fov_radians);
        Self { field, cache }
    }

    pub fn final_dir(&self, ray: &Ray) -> Option<DVec3> {
        self.cache.final_dir(ray, &self.field)
    }
}
