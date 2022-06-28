use glam::{DVec3, Vec3};

use crate::{structs::ray_cache::RayCache, Field};

pub struct BlackHole {
    cache: RayCache,
}

impl BlackHole {
    pub fn new(radius: f64, camera_distance: f64) -> Self {
        let camera_pos = -camera_distance * DVec3::Z;
        let field = Field::new(radius, &camera_pos);
        let cache = RayCache::compute_new(10000, &field, &camera_pos);
        Self { cache }
    }
    pub fn fetch_final_dir(&self, z: f32) -> Option<Vec3> {
        self.cache.fetch_final_dir(z)
    }
}
