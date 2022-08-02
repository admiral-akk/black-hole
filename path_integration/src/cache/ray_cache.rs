use crate::find_bound;
use crate::{cast_ray_steps, Field, Ray};
use glam::{DVec3, Vec3};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct RayCache {
    pub cache: Vec<RayCachedAnswer>,
    pub max_z: f64,
}

// We're always projecting from (0.0, 0.0, -Z)
pub const RAY_START_DIR: Vec3 = Vec3::new(0.0, 0.0, -1.0);

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct RayCachedAnswer {
    pub z: f32,
    pub final_dir: [f32; 3],
}

const MIN_Z: f64 = -1.0;
const MIN_Z_F64: f64 = MIN_Z as f64;
const POW: i32 = 3;

fn index_to_z(index: usize, size: usize, max_z: f64) -> f64 {
    let r = (index as f64) / ((size - 1) as f64);
    let r = r.powf(1. / POW as f64);
    let z = MIN_Z_F64 + (max_z - MIN_Z_F64) * r;
    z
}

fn z_to_index(z: f32, size: usize, max_z: f64) -> f32 {
    ((size - 1) as f32) * ((z as f64 - MIN_Z) / (max_z - MIN_Z_F64)).powi(POW) as f32
}

impl RayCache {
    pub fn compute_new(size: usize, black_hole_radius: f32, camera_distance: f32) -> Self {
        let mut cache = Vec::new();
        let field = Field::new(black_hole_radius as f64, camera_distance as f64);

        let cache_pos = camera_distance * RAY_START_DIR;

        let max_distance = 2.0 * camera_distance as f64;
        let max_z = find_bound(&cache_pos, &field, 0.0000001, max_distance);

        for i in 0..size {
            let z = index_to_z(i, size, max_z);
            let dir = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
            let ray = Ray::new(cache_pos.as_dvec3(), dir);
            let result = cast_ray_steps(&ray, &field, max_distance, 10.0 * max_distance);
            if result.1.is_none() {
                println!("Caching missed unexpectedly!");
                break;
            } else {
                cache.push(RayCachedAnswer {
                    z: z as f32,
                    final_dir: result.1.unwrap().as_vec3().to_array(),
                })
            }
        }

        Self { cache, max_z }
    }

    pub fn fetch_final_dir(&self, z: f32) -> Option<Vec3> {
        if z > self.max_z as f32 {
            return None;
        }
        let closest_index = z_to_index(z, self.cache.len(), self.max_z) as usize;
        let left = &self.cache[closest_index];
        if closest_index == self.cache.len() - 1 {
            return Some(Vec3::from_array(left.final_dir));
        }
        let right = &self.cache[closest_index + 1];
        let diff = right.z - left.z;

        let lerp = Vec3::lerp(
            Vec3::from_array(left.final_dir),
            Vec3::from_array(right.final_dir),
            (z - left.z) / diff,
        );

        Some(lerp)
    }
}

#[cfg(test)]
mod tests {
    use glam::{DVec3, Vec3};

    use crate::{cache::ray_cache::RayCache, cast_ray_steps, Field, Ray};

    #[test]
    fn ray_cache_absorbed_in_x_plane() {
        let pos = -5.0 * DVec3::Z;
        let r = 1.0;
        let field = Field::new(r, pos.length());
        let cache_size = 100;
        let ray_cache = RayCache::compute_new(cache_size, r as f32, pos.length() as f32);

        let mut false_positive = Vec::new();
        let mut false_negative = Vec::new();

        let iterations = 10;
        for x in (-iterations)..=iterations {
            let x = (x as f64) / (iterations as f64);
            let ray = Ray::new(pos, DVec3::new(x, 0.0, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0).1;
            let approx_dir = ray_cache.fetch_final_dir(ray.dir.z as f32);
            if approx_dir.is_none() != actual_dir.is_none() {
                if approx_dir.is_none() {
                    false_negative.push(ray);
                } else {
                    false_positive.push(ray);
                }
            }
        }

        let fp_10 = &false_positive[0..(usize::min(10, false_positive.len()))];
        assert_eq!(
            false_positive.len(),
            0,
            "First 10 false positives: {:?}",
            fp_10
        );
        let fn_10 = &false_negative[0..(usize::min(10, false_negative.len()))];
        assert_eq!(
            false_negative.len(),
            0,
            "First 10 false negatives: {:?}",
            fn_10
        );
    }

    #[test]
    fn ray_cache_absorbed_in_y_plane() {
        let pos = -5.0 * DVec3::Z;
        let r = 1.0;
        let field = Field::new(r, pos.length());
        let cache_size = 100;
        let ray_cache = RayCache::compute_new(cache_size, r as f32, pos.length() as f32);

        let mut false_positive = Vec::new();
        let mut false_negative = Vec::new();
        let iterations = 10;
        for y in 0..=(iterations + 1) {
            let y = (y as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(pos, DVec3::new(0.0, y, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0).1;
            let approx_dir = ray_cache.fetch_final_dir(ray.dir.z as f32);
            if approx_dir.is_none() != actual_dir.is_none() {
                if approx_dir.is_none() {
                    false_negative.push(ray);
                } else {
                    false_positive.push(ray);
                }
            }
        }

        let fp_10 = &false_positive[0..(usize::min(10, false_positive.len()))];
        assert_eq!(
            false_positive.len(),
            0,
            "First 10 false positives: {:?}",
            fp_10
        );
        let fn_10 = &false_negative[0..(usize::min(10, false_negative.len()))];
        assert_eq!(
            false_negative.len(),
            0,
            "First 10 false negatives: {:?}",
            fn_10
        );
    }

    #[test]
    fn final_dir_in_x_plane() {
        let pos = -5.0 * DVec3::Z;
        let r = 1.0;
        let field = Field::new(r, pos.length());
        let cache_size = 100;
        let ray_cache = RayCache::compute_new(cache_size, r as f32, pos.length() as f32);

        let mut max_error = 0.0;
        let iterations = 10;
        let mut worst_case = DVec3::new(0.0, 0.0, -10.0);
        let mut approx = Vec3::new(0.0, 0.0, -10.0);
        let mut actual = Vec3::new(0.0, 0.0, -10.0);
        let mut index = 0;
        for x in 0..=(iterations + 1) {
            let t = x;
            let x = (x as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(pos, DVec3::new(x, 0.0, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0).1;
            if actual_dir.is_some() {
                let approximate_dir = ray_cache.fetch_final_dir(ray.dir.z as f32).unwrap();
                let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0).1.unwrap();
                let actual_dir = Vec3::new(
                    actual_dir.x as f32,
                    actual_dir.y as f32,
                    actual_dir.z as f32,
                );
                let error = (approximate_dir - actual_dir).length();
                if error > max_error {
                    index = t;
                    max_error = error;
                    worst_case = ray.dir;
                    approx = approximate_dir;
                    actual = actual_dir;
                }
            }
        }

        println!(
            "Cache size: {}\nIteration size: {}\nMax Error: {}\nInitial Dir: {:?}\nApprox: {:?}Actual: {:?}\nIndex: {}",
            cache_size, iterations, max_error, worst_case, approx, actual, index
        );
    }

    #[test]
    fn final_dir_in_y_plane() {
        let pos = -5.0 * DVec3::Z;
        let r = 1.0;
        let field = Field::new(r, pos.length());
        let cache_size = 100;
        let ray_cache = RayCache::compute_new(cache_size, r as f32, pos.length() as f32);

        let iterations = 10;
        let mut max_error = 0.0;
        let mut worst_case = DVec3::new(0.0, 0.0, -10.0);
        let mut approx = Vec3::new(0.0, 0.0, -10.0);
        let mut actual = Vec3::new(0.0, 0.0, -10.0);
        let mut index = 0;
        for y in 0..=(iterations + 1) {
            let t = y;
            let y = (y as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(pos, DVec3::new(0.0, y, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0).1;
            if actual_dir.is_some() {
                let approximate_dir = ray_cache.fetch_final_dir(ray.dir.z as f32).unwrap();
                let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0).1.unwrap();
                let actual_dir = Vec3::new(
                    actual_dir.x as f32,
                    actual_dir.y as f32,
                    actual_dir.z as f32,
                );
                let error = (approximate_dir - actual_dir).length();
                if error > max_error {
                    index = t;
                    max_error = error;
                    worst_case = ray.dir;
                    approx = approximate_dir;
                    actual = actual_dir;
                }
            }
        }

        println!(
            "Cache size: {}\nIteration size: {}\nMax Error: {}\nInitial Dir: {:?}\nApprox: {:?}Actual: {:?}\nIndex: {}",
            cache_size, iterations, max_error, worst_case, approx, actual, index
        );
    }

    #[test]
    fn serialization() {
        let pos = -5.0 * DVec3::Z;
        let r = 1.0;
        let cache_size = 5;
        let ray_cache = RayCache::compute_new(cache_size, r as f32, pos.length() as f32);
        let serialized = serde_json::to_string(&ray_cache);
        assert!(serialized.is_ok());
        let deserialized: Result<RayCache, serde_json::Error> =
            serde_json::from_str(serialized.unwrap().as_str());
        assert!(deserialized.is_ok());
        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized, ray_cache);
    }
}
