use glam::{DVec3, Vec3Swizzles};
use quaternion::Quaternion;
use std::ops::{Add, Mul};

use crate::{cast_ray_steps, Field, Ray};

pub struct RayCache {
    cache: Vec<RayCachedAnswer>,
}

#[derive(Debug)]
struct RayCachedAnswer {
    pub x: f64,
    pub final_dir: DVec3,
}

fn rotate_to_z(ray: &Ray) -> Ray {
    let mut up = ray.pos.cross(ray.dir);
    if up.length() == 0.0 {
        up = DVec3::new(ray.pos.z, -ray.pos.x, -ray.pos.y).normalize();
        up = up - up.dot(ray.pos.normalize()) * up;
    }
    up = up.normalize();
    let q1: Quaternion<f64> =
        quaternion::rotation_from_to(ray.pos.to_array(), (-DVec3::Z).to_array());
    let p = quaternion::rotate_vector(q1, ray.pos.to_array());
    let d = quaternion::rotate_vector(q1, ray.dir.to_array());
    return Ray::new(DVec3::from_array(p), DVec3::from_array(d));
}

fn rotate_around_z(ray: &Ray, original_ray: &Ray) -> Ray {
    let mut up = original_ray.pos.cross(original_ray.dir);
    if up.length() == 0.0 {
        up = DVec3::new(original_ray.pos.z, -original_ray.pos.x, -original_ray.pos.y).normalize();
        up = up - up.dot(original_ray.pos.normalize()) * original_ray.pos.normalize();
    }
    up = up.normalize();
    println!("up: {:?}", up);
    println!("up * pos: {:?}", up.dot(original_ray.pos));
    let q1: Quaternion<f64> =
        quaternion::rotation_from_to(original_ray.pos.to_array(), (-DVec3::Z).to_array());
    let rotated_up = quaternion::rotate_vector(q1, up.to_array());
    println!("Rotated up: {:?}", rotated_up);
    let q2 = quaternion::rotation_from_to(rotated_up, DVec3::Y.to_array());
    let p = quaternion::rotate_vector(q2, ray.pos.to_array());
    let d = quaternion::rotate_vector(q2, ray.dir.to_array());
    return Ray::new(DVec3::from_array(p), DVec3::from_array(d));
}

// Need to:
// 1. Rotate the position to (0.0,0.0,-Z)
// 2. Pivot about (0.0,0.0,-Z) such that dir lies on the (-X,0.0,-Z) plane
// 3. Return x
fn canonical_rotation(ray: &Ray) -> Quaternion<f64> {
    let mut up = ray.pos.cross(ray.dir);
    if up.length() == 0.0 {
        up = DVec3::new(ray.pos.z, -ray.pos.x, -ray.pos.y).normalize();
        up = up - up.dot(ray.pos.normalize()) * ray.pos.normalize();
    }
    up = up.normalize();
    let q1: Quaternion<f64> =
        quaternion::rotation_from_to(ray.pos.to_array(), (-DVec3::Z).to_array());
    let rotated_up = quaternion::rotate_vector(q1, up.to_array());

    let q2 = quaternion::rotation_from_to(rotated_up, DVec3::Y.to_array());
    quaternion::mul(q2, q1)
}

fn to_canonical_form(ray: &Ray) -> f64 {
    let q = canonical_rotation(ray);
    quaternion::rotate_vector(q, ray.dir.to_array())[0]

    // find "up"

    // -(ray.dir.x.powi(2) + ray.dir.y.powi(2)).sqrt()
}

// let's rotate the ray start to (0.0,0.0,-Z), then calculate
fn from_canonical_form(dir: &DVec3, original_ray: &Ray) -> DVec3 {
    let q = canonical_rotation(original_ray);
    let q_len = quaternion::square_len(q);
    let q_inv = quaternion::scale(quaternion::conj(q), 1.0 / q_len);
    let rotated_dir = quaternion::rotate_vector(q_inv, dir.to_array());
    return DVec3::new(rotated_dir[0], rotated_dir[1], rotated_dir[2]);

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

// want to skew sample ([0,1]) to 1 values, as they're closer to the boundary.
fn rescale(r: f64) -> f64 {
    r.powf(1.0 / 3.0)
}

impl RayCache {
    pub fn compute_new(size: usize, field: &Field, camera_pos: &DVec3, fov_radians: f64) -> Self {
        let mut cache = Vec::new();

        // We're always projecting from (0.0, 0.0, -Z)
        let cache_pos = -camera_pos.length() * DVec3::Z;

        // TODO: need to figure out right constant to multiply by.
        // Too large ruins the sample effeciency.
        // Too small results in missing some outer rays.
        let left_dir = -4.0 * f64::tan(fov_radians / 2.0) * DVec3::X + DVec3::Z;

        // right_x gets close but misses.
        let right_x = find_bound(&cache_pos, field, 0.0000001, &left_dir);
        let right_dir = right_x * DVec3::X + DVec3::Z;

        for i in 0..size {
            let r = (i as f64) / ((size - 1) as f64);
            let r = rescale(r);
            let dir = (1.0 - r) * left_dir + r * right_dir;
            let ray = Ray::new(cache_pos, dir);
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

    pub fn final_dir(&self, ray: &Ray) -> Option<DVec3> {
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

    use crate::{
        cast_ray_steps,
        structs::ray_cache::{rotate_around_z, rotate_to_z, RayCache},
        Field, Ray,
    };

    use super::{binary_search, from_canonical_form, to_canonical_form, RayCachedAnswer};

    #[test]
    fn canonical_form_idempotent() {
        let iterations = 10;
        let epsilon = 0.0001;

        let starts = [
            -DVec3::Z,
            DVec3::Z,
            -DVec3::X,
            DVec3::X,
            -DVec3::Y,
            DVec3::Y,
        ];
        for start in starts {
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

                        if false && (original_dir - ray.dir).length() >= epsilon {
                            println!(
                                "\ninitial: {:?}
                                \nidempotent: {:?}
                                \npos:{:?}",
                                ray.dir, original_dir, ray.pos
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn canonical_form_center() {
        let epsilon = 0.0000001;
        let starts = [
            (-DVec3::Z, -DVec3::Z),
            (DVec3::Z, DVec3::Z),
            (-DVec3::X, -DVec3::X),
            (DVec3::X, DVec3::X),
            (-DVec3::Y, -DVec3::Y),
            (DVec3::Y, DVec3::Y),
        ];
        for (start, dir) in starts {
            let ray = Ray::new(start, dir);
            assert_eq!(
                (to_canonical_form(&ray) - 0.0).abs() < epsilon,
                true,
                "Start: {:?}, x: {}",
                start,
                to_canonical_form(&ray)
            );
        }
    }

    #[test]
    fn canonical_form_left() {
        let epsilon = 0.0000001;
        let starts = [
            (-DVec3::Z, -DVec3::X),
            (DVec3::Z, DVec3::Y),
            (-DVec3::X, -DVec3::Y),
            (DVec3::X, DVec3::Z),
            (-DVec3::Y, -DVec3::Z),
            (DVec3::Y, DVec3::X),
        ];
        for (start, dir) in starts {
            let ray = Ray::new(start, dir);
            assert_eq!(
                (to_canonical_form(&ray).abs() - 1.0).abs() < epsilon,
                true,
                "Start: {:?}",
                start
            );
        }
    }

    #[test]
    fn canonical_form_45_degrees() {
        let epsilon = 0.00001;
        let starts = [
            (-DVec3::Z, -DVec3::Z - DVec3::Y),
            (-DVec3::Z, -DVec3::Z + DVec3::Y),
            (-DVec3::Z, -DVec3::Z - DVec3::X),
            (-DVec3::Z, -DVec3::Z + DVec3::X),
            (-DVec3::Z, DVec3::Z - DVec3::Y),
            (-DVec3::Z, DVec3::Z + DVec3::Y),
            (-DVec3::Z, DVec3::Z - DVec3::X),
            (-DVec3::Z, DVec3::Z + DVec3::X),
            (-DVec3::X, DVec3::X + DVec3::Z),
            (-DVec3::X, DVec3::X - DVec3::Z),
            (-DVec3::X, DVec3::X + DVec3::Y),
            (-DVec3::X, DVec3::X - DVec3::Y),
            (-DVec3::X, -DVec3::X + DVec3::Z),
            (-DVec3::X, -DVec3::X - DVec3::Z),
            (-DVec3::X, -DVec3::X + DVec3::Y),
            (-DVec3::X, -DVec3::X - DVec3::Y),
        ];
        for (start, dir) in starts {
            let ray = Ray::new(start, dir);
            assert_eq!(
                (to_canonical_form(&ray).abs() - 1.0 / 2.0_f64.sqrt()).abs() < epsilon,
                true,
                "Start: {:?}\n
                Dir: {:?}\n
                x: {}\n",
                start,
                dir,
                to_canonical_form(&ray)
            );
        }
    }

    #[test]
    fn canonical_form_135_degrees() {
        let epsilon = 0.00001;
        let starts = [
            (-DVec3::X + DVec3::Z, -DVec3::X + DVec3::Z),
            (-DVec3::X + DVec3::Z, -DVec3::X - DVec3::Z),
            (-DVec3::X + DVec3::Z, DVec3::X - DVec3::Z),
            (-DVec3::X + DVec3::Z, DVec3::X + DVec3::Z),
            (-DVec3::X + DVec3::Z, DVec3::X - DVec3::Y),
            (-DVec3::X + DVec3::Z, DVec3::X + DVec3::Y),
            (-DVec3::X + DVec3::Z, -DVec3::X - DVec3::Y),
            (-DVec3::X + DVec3::Z, -DVec3::X + DVec3::Y),
        ];
        for (start, dir) in starts {
            let ray = Ray::new(start, dir);
            let new_ray = rotate_to_z(&ray);
            let f = rotate_around_z(&new_ray, &ray);
            println!("Old: {:?}\nNew: {:?}\nFinal: {:?}", ray, new_ray, f);
            let d = dir.normalize();
            let x = (d - d.dot(ray.pos.normalize()) * ray.pos.normalize()).length();
            assert_eq!(
                (to_canonical_form(&ray).abs() - x).abs() < epsilon,
                true,
                "Start: {:?}\n
                Dir: {:?}\n
                x: {}\n
                proper x: {}\n",
                start,
                dir,
                to_canonical_form(&ray),
                x
            );
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
        let ray_cache = RayCache::compute_new(10000, &field, &start, std::f64::consts::FRAC_2_PI);

        let mut false_positive = Vec::new();
        let mut false_negative = Vec::new();

        let iterations = 100000;
        for x in (-iterations)..=iterations {
            let x = (x as f64) / (iterations as f64);
            let ray = Ray::new(start, DVec3::new(x, 0.0, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 100.0);
            let approx_dir = ray_cache.final_dir(&ray);
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
        let ray_cache = RayCache::compute_new(10000, &field, &start, std::f64::consts::FRAC_2_PI);

        let mut false_positive = Vec::new();
        let mut false_negative = Vec::new();
        let iterations = 100000;
        for y in 0..=(iterations + 1) {
            let y = (y as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(start, DVec3::new(0.0, y, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 100.0);
            let approx_dir = ray_cache.final_dir(&ray);
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
                let approximate_dir = ray_cache.final_dir(&ray).unwrap();
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
        let cache_size = 10000;
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
                let approximate_dir = ray_cache.final_dir(&ray).unwrap();
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
