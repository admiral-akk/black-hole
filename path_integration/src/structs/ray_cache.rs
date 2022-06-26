use glam::DVec3;

use crate::{cast_ray_steps, Field, Ray};

pub struct RayCache {
    cache: Vec<RayCachedAnswer>,
}

#[derive(Debug)]
struct RayCachedAnswer {
    pub x: f64,
    pub final_dir: DVec3,
}

fn to_canonical_form(ray: &Ray) -> f64 {
    -(ray.dir.x.powi(2) + ray.dir.y.powi(2)).sqrt()
}

fn from_canonical_form(dir: &DVec3, original_ray: &Ray) -> DVec3 {
    let angle = -f64::atan2(original_ray.dir.y, -original_ray.dir.x);
    let (cos_angle, sin_angle) = (f64::cos(angle), f64::sin(angle));

    DVec3::new(
        dir.x * cos_angle - dir.y * sin_angle,
        dir.x * sin_angle + dir.y * cos_angle,
        dir.z,
    )
}

// Finds the element with largest val.x such that val.x <= x
// Assumes that cache[0].x <= x
fn binary_search(cache: &[RayCachedAnswer], x: f64) -> usize {
    let mut low = 0;
    let mut high = cache.len();
    while high > low + 1 {
        let mid = (high + low) / 2;
        if cache[mid].x <= x {
            low = mid;
        } else {
            high = mid;
        }
    }

    low
}

fn find_bound(camera_pos: &DVec3, field: &Field, epsilon: f64, left_dir: &DVec3) -> f64 {
    let mut left = left_dir.clone();
    let mut right = DVec3::Z;
    while right.x - left.x > epsilon {
        let center = 0.5 * (left + right);
        let ray = Ray::new(*camera_pos, center);
        let final_dir = cast_ray_steps(&ray, field, 100.0);
        if final_dir.is_none() {
            // hit the black hole
            right = center;
        } else {
            left = center;
        }
    }
    left.x
}

impl RayCache {
    pub fn compute_new(size: usize, field: &Field, camera_pos: &DVec3, fov_radians: f64) -> Self {
        let mut cache = Vec::new();
        // TODO: need to figure out right constant to multiply by.
        // Too large ruins the sample effeciency.
        // Too small results in missing some outer rays.
        let left_dir = -4.0 * f64::tan(fov_radians / 2.0) * DVec3::X + DVec3::Z;

        // left_x gets close but misses.
        let left_x = find_bound(camera_pos, field, 0.0000001, &left_dir);
        let right_dir = left_x * DVec3::X + DVec3::Z;

        for i in 0..size {
            let r = (i as f64) / ((size - 1) as f64);
            let dir = (1.0 - r) * left_dir + r * right_dir;
            let ray = Ray::new(camera_pos.clone(), dir);
            let result = cast_ray_steps(&ray, &field, 100.0);
            if result.is_none() {
                break;
            } else {
                cache.push(RayCachedAnswer {
                    x: ray.dir.x,
                    final_dir: result.unwrap(),
                })
            }
        }

        Self { cache }
    }

    pub fn final_dir(&self, ray: &Ray, field: &Field) -> Option<DVec3> {
        let x = to_canonical_form(&ray);
        if x > self.cache[self.cache.len() - 1].x {
            return None;
        }

        let closest_index = binary_search(&self.cache, x);
        let left = &self.cache[closest_index];
        let right = &self.cache[closest_index + 1];
        let diff = right.x - left.x;

        let lerp = DVec3::lerp(left.final_dir, right.final_dir, (x - left.x) / diff);

        Some(from_canonical_form(&lerp, ray))
    }
}

#[cfg(test)]
mod tests {
    use glam::DVec3;

    use crate::{cast_ray_steps, structs::ray_cache::RayCache, Field, Ray};

    use super::{binary_search, from_canonical_form, to_canonical_form, RayCachedAnswer};

    #[test]
    fn canonical_form_idempotent() {
        let start = -5.0 * DVec3::Z;
        let iterations = 100;
        let epsilon = 0.0000001;

        for x in (-iterations)..=iterations {
            for y in (-iterations)..=iterations {
                for z in (-iterations)..=iterations {
                    if x == 0 && y == 0 && z == 0 {
                        continue;
                    }
                    let (x, y, z) = (
                        (x as f64) / (iterations as f64),
                        (y as f64) / (iterations as f64),
                        (z as f64) / (iterations as f64),
                    );
                    let ray = Ray::new(start, DVec3::new(x, y, z));
                    let x = to_canonical_form(&ray);
                    let final_dir = DVec3::new(x, 0.0, ray.dir.z);
                    let original_dir = from_canonical_form(&final_dir, &ray);
                    assert_eq!((original_dir - ray.dir).length() < epsilon, true);
                }
            }
        }
    }

    #[test]
    fn binary_search_all() {
        let cache = [
            RayCachedAnswer {
                x: -10.0,
                final_dir: DVec3::ZERO,
            },
            RayCachedAnswer {
                x: -2.0,
                final_dir: DVec3::ZERO,
            },
            RayCachedAnswer {
                x: 0.0,
                final_dir: DVec3::ZERO,
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
        let ray_cache = RayCache::compute_new(1000, &field, &start, std::f64::consts::FRAC_2_PI);

        let mut false_positive = Vec::new();
        let mut false_negative = Vec::new();

        let iterations = 100000;
        for x in (-iterations)..=iterations {
            let x = (x as f64) / (iterations as f64);
            let ray = Ray::new(start, DVec3::new(x, 0.0, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 100.0);
            let approx_dir = ray_cache.final_dir(&ray, &field);
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
        let ray_cache = RayCache::compute_new(1000, &field, &start, std::f64::consts::FRAC_2_PI);

        let mut false_positive = Vec::new();
        let mut false_negative = Vec::new();
        let iterations = 100000;
        for y in 0..=(iterations + 1) {
            let y = (y as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(start, DVec3::new(0.0, y, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 100.0);
            let approx_dir = ray_cache.final_dir(&ray, &field);
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
        let cache_size = 100000;
        let ray_cache =
            RayCache::compute_new(cache_size, &field, &start, std::f64::consts::FRAC_2_PI);

        let mut max_error = 0.0;
        let iterations = 100000;
        let mut worst_case = DVec3::new(0.0, 0.0, -10.0);
        let mut approx = DVec3::new(0.0, 0.0, -10.0);
        let mut actual = DVec3::new(0.0, 0.0, -10.0);
        for x in 0..=(iterations + 1) {
            let x = (x as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(start, DVec3::new(x, 0.0, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 100.0);
            if actual_dir.is_some() {
                let approximate_dir = ray_cache.final_dir(&ray, &field).unwrap();
                let actual_dir = cast_ray_steps(&ray, &field, 100.0).unwrap();
                let error = (approximate_dir - actual_dir).length();
                if error > max_error {
                    max_error = error;
                    worst_case = ray.dir;
                    approx = approximate_dir;
                    actual = actual_dir;
                }
            }
        }

        println!(
            "Cache size: {}\nIteration size: {}\nMax Error: {}\nInitial Dir: {:?}\nApprox: {:?}Actual: {:?}\n",
            cache_size, iterations, max_error, worst_case, approx, actual
        );
    }

    #[test]
    fn final_dir_in_y_plane() {
        let start = -5.0 * DVec3::Z;
        let r = 1.0;
        let field = Field::new(r, &start);
        let cache_size = 100000;
        let ray_cache =
            RayCache::compute_new(cache_size, &field, &start, std::f64::consts::FRAC_2_PI);

        let iterations = 100000;
        let mut max_error = 0.0;
        let mut worst_case = DVec3::new(0.0, 0.0, -10.0);
        let mut approx = DVec3::new(0.0, 0.0, -10.0);
        let mut actual = DVec3::new(0.0, 0.0, -10.0);
        for y in 0..=(iterations + 1) {
            let y = (y as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(start, DVec3::new(0.0, y, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 100.0);
            if actual_dir.is_some() {
                let approximate_dir = ray_cache.final_dir(&ray, &field).unwrap();
                let actual_dir = cast_ray_steps(&ray, &field, 100.0).unwrap();
                let error = (approximate_dir - actual_dir).length();
                if error > max_error {
                    max_error = error;
                    worst_case = ray.dir;
                    approx = approximate_dir;
                    actual = actual_dir;
                }
            }
        }

        println!(
            "Cache size: {}\nIteration size: {}\nMax Error: {}\nInitial Dir: {:?}\nApprox: {:?}Actual: {:?}\n",
            cache_size, iterations, max_error, worst_case, approx, actual
        );
    }
}
