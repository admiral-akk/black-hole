use crate::find_bound;
use crate::{cast_ray_steps, Field, Ray};
use glam::{DVec3, Vec3};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct FixedDistanceRayCache {
    pub distance: f32,
    pub max_z: f32,
    pub cache: Vec<RayCachedAnswer>,
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
const POW: i32 = 2;

fn index_to_z(index: usize, size: usize, max_z: f64) -> f64 {
    let r = (index as f64) / ((size - 1) as f64);
    let r = r.powf(1. / POW as f64);
    let z = MIN_Z_F64 + (max_z - MIN_Z_F64) * r;
    z
}

fn z_to_index(z: f32, size: usize, max_z: f64) -> f32 {
    ((size - 1) as f32) * ((z as f64 - MIN_Z) / (max_z - MIN_Z_F64)).powi(POW) as f32
}
const Z_EPSILON: f64 = 0.000000001;

impl FixedDistanceRayCache {
    pub fn compute_new(cache_size: usize, black_hole_radius: f32, distance: f32) -> Self {
        let cache_pos = distance * RAY_START_DIR;
        let field = Field::new(black_hole_radius as f64, distance as f64);
        let ray_travel_distance_limit = 2.0 * distance as f64;
        let max_z = find_bound(&cache_pos, &field, Z_EPSILON, ray_travel_distance_limit);
        let mut cache = Vec::new();
        for i in 0..cache_size {
            let z = index_to_z(i, cache_size, max_z);
            let dir = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
            let ray = Ray::new(cache_pos.as_dvec3(), dir);
            let result = cast_ray_steps(
                &ray,
                &field,
                2. * distance as f64,
                10.0 * ray_travel_distance_limit,
            );
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

        Self {
            distance,
            max_z: max_z as f32,
            cache,
        }
    }

    pub fn fetch_final_dir(&self, z: f32) -> Option<Vec3> {
        if z > self.max_z as f32 {
            return None;
        }
        let closest_index = z_to_index(z, self.cache.len(), self.max_z as f64) as usize;
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

    use crate::{
        cache::fixed_distance_ray_cache::FixedDistanceRayCache, cast_ray_steps, Field, Ray,
    };

    #[test]
    fn ray_cache_absorbed_in_x_plane_dist() {
        let cache_size = 512;
        let black_hole_radius = 1.0;
        let distance_bounds = (5.0, 20.0);

        let iterations = 100;
        for i in 0..iterations {
            let mut false_positive = Vec::new();
            let mut false_negative = Vec::new();
            let mut true_negatives_count = 0;
            let mut true_positives_count = 0;
            let distance = distance_bounds.0 as f64
                + (distance_bounds.1 - distance_bounds.0) as f64 * i as f64 / (100 - 1) as f64;
            let ray_cache =
                FixedDistanceRayCache::compute_new(cache_size, black_hole_radius, distance as f32);
            let pos = -distance * DVec3::Z;
            let field = Field::new(black_hole_radius as f64, distance);
            for x in (-iterations)..=iterations {
                let x = (x as f64) / (iterations as f64);
                let ray = Ray::new(pos, DVec3::new(x, 0.0, 1.0));
                let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0).1;
                let approx_dir = ray_cache.fetch_final_dir(ray.dir.z as f32);
                if actual_dir.is_none() {
                    true_negatives_count += 1;
                } else {
                    true_positives_count += 1;
                }
                if approx_dir.is_none() != actual_dir.is_none() {
                    if approx_dir.is_none() {
                        false_negative.push(ray);
                    } else {
                        false_positive.push(ray);
                    }
                }
            }
            let fp_10 = &false_positive[0..(usize::min(10, false_positive.len()))];
            let fp_rate = false_positive.len() as f32 / true_positives_count as f32;
            if true_positives_count > 0 || false_positive.len() > 0 {
                assert!(
                fp_rate < 0.01,
                "\nFalse positive rate is >= 1%\nDistance: {}\nTrue positive count: {}\nFalse positive count: {}\nRate: {}\nExamples: {:?}",
                distance,
                true_positives_count,
                false_positive.len(),
                fp_rate,
                fp_10
            );
            }
            if true_negatives_count > 0 || false_positive.len() > 0 {
                let fn_10 = &false_negative[0..(usize::min(10, false_negative.len()))];
                let fn_rate = false_negative.len() as f32 / true_negatives_count as f32;
                assert!(
                fn_rate < 0.01,
                "\nFalse negative rate is >= 1%\nDistance: {}\nTrue negative count: {}\nFalse negative count: {}\n Rate: {}\nExamples: {:?}",
                distance,
                true_negatives_count,
                false_negative.len(),
                fn_rate < 0.01,
                fn_10
            );
            }
        }
    }

    #[test]
    fn ray_cache_absorbed_in_x_plane() {
        let _cache_dimensions = (10, 100);
        let pos = -5.0 * DVec3::Z;
        let r = 1.0;
        let field = Field::new(r, pos.length());
        let _distance_bounds = (5.0, 20.0);
        let ray_cache = FixedDistanceRayCache::compute_new(100, r as f32, pos.length() as f32);

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
        let ray_cache =
            FixedDistanceRayCache::compute_new(cache_size, r as f32, pos.length() as f32);

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
        let ray_cache =
            FixedDistanceRayCache::compute_new(cache_size, r as f32, pos.length() as f32);

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
        let ray_cache =
            FixedDistanceRayCache::compute_new(cache_size, r as f32, pos.length() as f32);

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
        let ray_cache =
            FixedDistanceRayCache::compute_new(cache_size, r as f32, pos.length() as f32);
        let serialized = serde_json::to_string(&ray_cache);

        assert!(serialized.is_ok());

        let deserialized: Result<FixedDistanceRayCache, serde_json::Error> =
            serde_json::from_str(serialized.unwrap().as_str());

        assert!(deserialized.is_ok());

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized, ray_cache);
    }
}
