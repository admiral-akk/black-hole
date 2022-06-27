use glam::DVec3;

use crate::{Field, Ray};

use super::ray_cache::RayCache;

pub struct BlackHole {
    cache: RayCache,
}

impl BlackHole {
    pub fn new(radius: f64, camera_distance: f64, fov_radians: f64) -> Self {
        let camera_pos = -camera_distance * DVec3::Z;
        let field = Field::new(radius, &camera_pos);
        let cache = RayCache::compute_new(10000, &field, &camera_pos, fov_radians);
        Self { cache }
    }

    pub fn final_dir(&self, ray: &Ray) -> Option<DVec3> {
        self.cache.final_dir(ray)
    }
}
