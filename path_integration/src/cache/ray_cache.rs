use glam::Vec3;
use serde::{Deserialize, Serialize};

use super::fixed_distance_ray_cache::FixedDistanceRayCache;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct RayCache {
    pub caches: Vec<FixedDistanceRayCache>,
}
fn index_to_distance(index: usize, size: usize, distance_bounds: (f32, f32)) -> f32 {
    (index as f32) * (distance_bounds.1 - distance_bounds.0) / ((size - 1) as f32)
        + distance_bounds.0
}

fn distance_to_index(distance: f32, size: usize, distance_bounds: (f32, f32)) -> usize {
    (((size - 1) as f32) * (distance - distance_bounds.0) / (distance_bounds.1 - distance_bounds.0))
        as usize
}

impl RayCache {
    pub fn compute_new(
        cache_dimensions: (usize, usize),
        black_hole_radius: f32,
        distance_bounds: (f32, f32),
    ) -> Self {
        let mut caches = Vec::new();
        for x in 0..cache_dimensions.0 {
            let camera_distance = distance_bounds.0
                + (distance_bounds.1 - distance_bounds.0) * (x as f32)
                    / (cache_dimensions.0 - 1) as f32;
            caches.push(FixedDistanceRayCache::compute_new(
                cache_dimensions.1,
                black_hole_radius,
                camera_distance,
            ));
        }

        Self { caches }
    }

    pub fn fetch_final_dir(&self, distance: f32, z: f32) -> Option<Vec3> {
        let distance_bounds = (
            self.caches.first().unwrap().distance,
            self.caches.last().unwrap().distance,
        );
        let left_index = distance_to_index(distance, self.caches.len(), distance_bounds);
        let left = &self.caches[left_index];
        if left_index == self.caches.len() - 1 {
            return left.fetch_final_dir(z);
        }
        let right = &self.caches[left_index + 1];
        let diff = right.distance - left.distance;
        let left_distance = left.distance;

        let left = left.fetch_final_dir(z);
        let right = right.fetch_final_dir(z);

        // if the left avoids the black hole, the right definitely avoids it.
        if left.is_some() {
            return Some(Vec3::lerp(
                left.unwrap(),
                right.unwrap(),
                (distance - left_distance) / diff,
            ));
        }

        // if the right hits the black hole, the left definitely hits it.
        if right.is_none() {
            return None;
        }

        // if they disagree on whether it hits, use the closer cache.
        if distance - left_distance < diff / 2. {
            return left;
        } else {
            return right;
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::{DVec3, Vec3};

    use crate::{cache::ray_cache::RayCache, cast_ray_steps, Field, Ray};

    #[test]
    fn ray_caches_absorbed_in_x_plane() {
        let cache_dimensions = (100, 512);
        let black_hole_radius = 1.0;
        let distance_bounds = (5.0, 20.0);
        let ray_cache = RayCache::compute_new(cache_dimensions, black_hole_radius, distance_bounds);

        let iterations = 100;
        for i in 0..iterations {
            let mut false_positive = Vec::new();
            let mut false_negative = Vec::new();
            let mut true_negatives_count = 0;
            let mut true_positives_count = 0;
            let distance = distance_bounds.0 as f64
                + (distance_bounds.1 - distance_bounds.0) as f64 * i as f64 / (100 - 1) as f64;
            let pos = -distance * DVec3::Z;
            let field = Field::new(black_hole_radius as f64, distance);
            for x in (-iterations)..=iterations {
                let x = (x as f64) / (iterations as f64);
                let ray = Ray::new(pos, DVec3::new(x, 0.0, 1.0));
                let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0).1;
                let approx_dir = ray_cache.fetch_final_dir(distance as f32, ray.dir.z as f32);
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
            let fp_rate = (false_positive.len() as f32 / true_positives_count as f32);
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
                let fn_rate = (false_negative.len() as f32 / true_negatives_count as f32);
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
    fn final_dir_in_x_plane() {
        let mut max_error = 0.0;
        let iterations = 10;
        let mut worst_case = DVec3::new(0.0, 0.0, -10.0);
        let mut approx = Vec3::new(0.0, 0.0, -10.0);
        let mut actual = Vec3::new(0.0, 0.0, -10.0);
        let mut index = 0;
        let mut false_negatives = 0;
        let mut false_positives = 0;

        let cache_dimensions = (100, 100);
        let black_hole_radius = 1.0;
        let distance_bounds = (5.0, 20.0);
        let ray_cache = RayCache::compute_new(cache_dimensions, black_hole_radius, distance_bounds);

        for i in 0..100 {
            let distance = distance_bounds.0 as f64
                + (distance_bounds.1 - distance_bounds.0) as f64 * i as f64 / (100 - 1) as f64;
            let pos = -distance * DVec3::Z;
            let r = 1.0;
            let field = Field::new(r, pos.length());
            for x in 0..=(iterations + 1) {
                let t = x;
                let x = (x as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
                let ray = Ray::new(pos, DVec3::new(x, 0.0, 1.0));
                let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0).1;
                if actual_dir.is_some() {
                    let approximate_dir =
                        ray_cache.fetch_final_dir(distance as f32, ray.dir.z as f32);
                    if approximate_dir.is_none() {
                        false_negatives += 1;
                        continue;
                    }
                    let approximate_dir = approximate_dir.unwrap();
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
                } else {
                    let approximate_dir =
                        ray_cache.fetch_final_dir(distance as f32, ray.dir.z as f32);
                    if approximate_dir.is_some() {
                        false_positives += 1;
                    }
                }
            }
        }

        println!(
            "Cache size: {:?}\nIteration size: {}\nMax Error: {}\nInitial Dir: {:?}\nApprox: {:?}Actual: {:?}\nIndex: {}\nFalse Negatives: {}\nFalse Positives: {}",
            cache_dimensions, iterations, max_error, worst_case, approx, actual, index,false_negatives,false_positives
        );
    }

    #[test]
    fn serialization() {
        let cache_dimensions = (10, 10);
        let black_hole_radius = 1.0;
        let distance_bounds = (5.0, 20.0);
        let ray_cache = RayCache::compute_new(cache_dimensions, black_hole_radius, distance_bounds);
        let serialized = serde_json::to_string(&ray_cache);

        assert!(serialized.is_ok());

        let deserialized: Result<RayCache, serde_json::Error> =
            serde_json::from_str(serialized.unwrap().as_str());

        assert!(deserialized.is_ok());

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized, ray_cache);
    }
}
