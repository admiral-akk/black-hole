use glam::{DVec3, Vec3};

use crate::{cast_ray_steps, Field, Ray};

pub struct RayCache {
    cache: Vec<RayCachedAnswer>,
}

#[derive(Debug)]
struct RayCachedAnswer {
    pub z: f32,
    pub final_dir: Vec3,
}

// Finds the element with largest val.z such that val.z < z
// Assumes that cache[0].z <= z
fn binary_search(cache: &[RayCachedAnswer], z: f32) -> usize {
    let mut low = 0;
    let mut high = cache.len();
    while high > low + 1 {
        let mid = (high + low) / 2;
        if cache[mid].z <= z {
            low = mid;
        } else {
            high = mid;
        }
    }

    low
}

fn find_bound(camera_pos: &DVec3, field: &Field, epsilon: f64) -> f64 {
    let (mut miss_z, mut hit_z) = (-1.0, 1.0);
    while hit_z - miss_z > epsilon {
        let z = 0.5 * (hit_z + miss_z);
        let test = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
        let ray = Ray::new(*camera_pos, test);
        let final_dir = cast_ray_steps(&ray, field, 100.0);
        if final_dir.is_none() {
            // hit the black hole
            hit_z = test.z;
        } else {
            miss_z = test.z;
        }
    }
    miss_z
}

// want to skew sample ([0,1]) to 1 values, as they're closer to the boundary.
fn rescale(r: f64) -> f64 {
    r.powf(1.0 / 3.0)
}

impl RayCache {
    pub fn compute_new(size: usize, field: &Field, camera_pos: &DVec3) -> Self {
        let mut cache = Vec::new();

        // We're always projecting from (0.0, 0.0, -Z)
        let cache_pos = -camera_pos.length() * DVec3::Z;

        let min_z = -1.0;
        let max_z = find_bound(&cache_pos, field, 0.0000001);

        for i in 0..size {
            let r = (i as f64) / ((size - 1) as f64);
            let r = rescale(r);
            let z = min_z + (max_z - min_z) * r;
            let dir = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
            let ray = Ray::new(cache_pos, dir);
            let result = cast_ray_steps(&ray, &field, 100.0);
            if result.is_none() {
                println!("Caching missed unexpectedly!");
                break;
            } else {
                let result = result.unwrap();
                cache.push(RayCachedAnswer {
                    z: ray.dir.z as f32,
                    final_dir: Vec3::new(result.x as f32, result.y as f32, result.z as f32),
                })
            }
        }

        Self { cache }
    }

    pub fn fetch_final_dir(&self, z: f32) -> Option<Vec3> {
        let cache_last = self.cache.len() - 1;
        if z > self.cache[cache_last].z {
            return None;
        }

        let closest_index = binary_search(&self.cache, z);
        let left = &self.cache[closest_index];
        if closest_index == cache_last {
            return Some(left.final_dir);
        }
        let right = &self.cache[closest_index + 1];
        let diff = right.z - left.z;

        let lerp = Vec3::lerp(left.final_dir, right.final_dir, (z - left.z) / diff);

        Some(lerp)
    }
}

#[cfg(test)]
mod tests {
    use glam::{DVec3, Vec3};

    use crate::{cast_ray_steps, structs::ray_cache::RayCache, Field, Ray};

    use super::{binary_search, RayCachedAnswer};

    #[test]
    fn binary_search_all() {
        let cache = [
            RayCachedAnswer {
                z: -10.0,
                final_dir: Vec3::ZERO,
            },
            RayCachedAnswer {
                z: -2.0,
                final_dir: Vec3::ZERO,
            },
            RayCachedAnswer {
                z: 0.0,
                final_dir: Vec3::ZERO,
            },
        ];

        assert_eq!(binary_search(&cache, -10.0), 0);
        assert_eq!(binary_search(&cache, -5.0), 0);
        assert_eq!(binary_search(&cache, -2.0), 1);
        assert_eq!(binary_search(&cache, -1.0), 1);
        assert_eq!(binary_search(&cache, 0.0), 2);
        assert_eq!(binary_search(&cache, 1.0), 2);
    }

    #[test]
    fn ray_cache_absorbed_in_x_plane() {
        let start = -5.0 * DVec3::Z;
        let r = 1.0;
        let field = Field::new(r, &start);
        let ray_cache = RayCache::compute_new(10000, &field, &start);

        let mut false_positive = Vec::new();
        let mut false_negative = Vec::new();

        let iterations = 100000;
        for x in (-iterations)..=iterations {
            let x = (x as f64) / (iterations as f64);
            let ray = Ray::new(start, DVec3::new(x, 0.0, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 100.0);
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
        let start = -5.0 * DVec3::Z;
        let r = 1.0;
        let field = Field::new(r, &start);
        let ray_cache = RayCache::compute_new(10000, &field, &start);

        let mut false_positive = Vec::new();
        let mut false_negative = Vec::new();
        let iterations = 100000;
        for y in 0..=(iterations + 1) {
            let y = (y as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(start, DVec3::new(0.0, y, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 100.0);
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
        let start = -5.0 * DVec3::Z;
        let r = 1.0;
        let field = Field::new(r, &start);
        let cache_size = 10000;
        let ray_cache = RayCache::compute_new(cache_size, &field, &start);

        let mut max_error = 0.0;
        let iterations = 100000;
        let mut worst_case = DVec3::new(0.0, 0.0, -10.0);
        let mut approx = Vec3::new(0.0, 0.0, -10.0);
        let mut actual = Vec3::new(0.0, 0.0, -10.0);
        let mut index = 0;
        for x in 0..=(iterations + 1) {
            let t = x;
            let x = (x as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(start, DVec3::new(x, 0.0, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 100.0);
            if actual_dir.is_some() {
                let approximate_dir = ray_cache.fetch_final_dir(ray.dir.z as f32).unwrap();
                let actual_dir = cast_ray_steps(&ray, &field, 100.0).unwrap();
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
        let start = -5.0 * DVec3::Z;
        let r = 1.0;
        let field = Field::new(r, &start);
        let cache_size = 10000;
        let ray_cache = RayCache::compute_new(cache_size, &field, &start);

        let iterations = 100000;
        let mut max_error = 0.0;
        let mut worst_case = DVec3::new(0.0, 0.0, -10.0);
        let mut approx = Vec3::new(0.0, 0.0, -10.0);
        let mut actual = Vec3::new(0.0, 0.0, -10.0);
        let mut index = 0;
        for y in 0..=(iterations + 1) {
            let t = y;
            let y = (y as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(start, DVec3::new(0.0, y, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 100.0);
            if actual_dir.is_some() {
                let approximate_dir = ray_cache.fetch_final_dir(ray.dir.z as f32).unwrap();
                let actual_dir = cast_ray_steps(&ray, &field, 100.0).unwrap();
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
}
