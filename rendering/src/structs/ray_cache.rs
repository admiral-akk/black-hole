use glam::{DVec3, Vec3};
use path_integration::{cast_ray_steps, Field, Ray};

use crate::utils::extensions::ToPolar;

use super::data::Data;

pub struct RayCache {
    pub cache: Vec<RayCachedAnswer>,
    pub max_z: f32,
    pub z_to_index_multiple: f32,
}

// We're always projecting from (0.0, 0.0, -Z)
pub const RAY_START_DIR: Vec3 = Vec3::new(0.0, 0.0, -1.0);

#[derive(Debug)]
pub struct RayCachedAnswer {
    pub z: f32,
    pub final_dir: Vec3,
}

fn find_bound(camera_pos: &Vec3, field: &Field, epsilon: f64, max_distance: f64) -> f64 {
    let (mut miss_z, mut hit_z) = (-1.0, 1.0);
    while hit_z - miss_z > epsilon {
        let z = 0.5 * (hit_z + miss_z);
        let test = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
        let ray = Ray::new(camera_pos.as_dvec3(), test);
        let final_dir = cast_ray_steps(&ray, field, max_distance, 10.0 * max_distance);
        if final_dir.is_none() {
            // hit the black hole
            hit_z = test.z;
        } else {
            miss_z = test.z;
        }
    }
    miss_z
}

const MIN_Z: f32 = -1.0;
const MIN_Z_F64: f64 = MIN_Z as f64;

fn index_to_z(index: usize, size: usize, max_z: f64) -> f64 {
    let r = (index as f64) / ((size - 1) as f64);
    let r = r.sqrt();

    let z = (MIN_Z_F64 + (max_z - MIN_Z_F64) * r);
    z
}

fn rotate_about_z(angle: f32, vec: &mut Vec3) {
    let (sin, cos) = angle.sin_cos();
    let (x, y) = (vec.x, vec.y);
    vec.x = x * cos - y * sin;
    vec.y = x * sin + y * cos;
}

impl RayCache {
    fn z_to_index(&self, z: f32) -> f32 {
        (self.z_to_index_multiple * (z - MIN_Z) * (z - MIN_Z))
    }

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

        Self {
            cache,
            max_z: max_z as f32,
            z_to_index_multiple: ((size - 1) as f64 / (max_z as f64 - MIN_Z_F64).powi(2)) as f32,
        }
    }

    pub fn calculate_final_dir(&self, data: &mut Vec<Data>) {
        let mut empty_index = 0_usize;

        for i in 0..data.len() {
            match data[i] {
                Data::ObserverDir(index, start_dir) => {
                    let z = start_dir.z / start_dir.length();
                    let fetch = self.fetch_final_dir(z);
                    if fetch.is_some() {
                        let mut fetch = fetch.unwrap();
                        let angle = fast_math::atan2(start_dir.y, start_dir.x);
                        let index1 = self.z_to_index(z);
                        let index2 = index1 as usize;

                        let left = self.cache[index2].final_dir;
                        let right = self.cache[index2 + 1].final_dir;
                        let r_w = index1 - (index2 as f32);
                        let l_w = 1.0 - r_w;

                        let mut v = r_w * right + l_w * left;
                        rotate_about_z(fast_math::atan2(start_dir.y, start_dir.x), &mut v);

                        // rotate final_dir to match start_dir
                        data[empty_index] = Data::Polar(index, v.to_polar());
                        empty_index += 1;
                    }
                }
                _ => {
                    panic!("Should be canon dir format here!")
                }
            }
        }

        data.drain(empty_index..data.len());
    }

    // r = (i / (size-1)).sqrt()
    // z = -1 + (max_z +1)* (i / (size-1)).sqrt()
    // if z', then

    // (size-1)*(z + 1) ^ 2 / (max_z + 1) = i
    // z[i] = (max_z + 1)
    // i -> (max_z + 1)
    pub fn fetch_final_dir(&self, z: f32) -> Option<Vec3> {
        if z > self.max_z {
            return None;
        }
        let closest_index = self.z_to_index(z) as usize;
        let left = &self.cache[closest_index];
        if closest_index == self.cache.len() - 1 {
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
    use path_integration::{cast_ray_steps, Field, Ray};

    use crate::structs::ray_cache::RayCache;

    #[test]
    fn ray_cache_absorbed_in_x_plane() {
        let pos = -5.0 * DVec3::Z;
        let r = 1.0;
        let field = Field::new(r, pos.length());
        let cache_size = 10000;
        let ray_cache = RayCache::compute_new(cache_size, r as f32, pos.length() as f32);

        let mut false_positive = Vec::new();
        let mut false_negative = Vec::new();

        let iterations = 100000;
        for x in (-iterations)..=iterations {
            let x = (x as f64) / (iterations as f64);
            let ray = Ray::new(pos, DVec3::new(x, 0.0, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0);
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
        let cache_size = 10000;
        let ray_cache = RayCache::compute_new(cache_size, r as f32, pos.length() as f32);

        let mut false_positive = Vec::new();
        let mut false_negative = Vec::new();
        let iterations = 100000;
        for y in 0..=(iterations + 1) {
            let y = (y as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(pos, DVec3::new(0.0, y, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0);
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
        let cache_size = 10000;
        let ray_cache = RayCache::compute_new(cache_size, r as f32, pos.length() as f32);

        let mut max_error = 0.0;
        let iterations = 100000;
        let mut worst_case = DVec3::new(0.0, 0.0, -10.0);
        let mut approx = Vec3::new(0.0, 0.0, -10.0);
        let mut actual = Vec3::new(0.0, 0.0, -10.0);
        let mut index = 0;
        for x in 0..=(iterations + 1) {
            let t = x;
            let x = (x as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(pos, DVec3::new(x, 0.0, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0);
            if actual_dir.is_some() {
                let approximate_dir = ray_cache.fetch_final_dir(ray.dir.z as f32).unwrap();
                let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0).unwrap();
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
        let cache_size = 10000;
        let ray_cache = RayCache::compute_new(cache_size, r as f32, pos.length() as f32);

        let iterations = 100000;
        let mut max_error = 0.0;
        let mut worst_case = DVec3::new(0.0, 0.0, -10.0);
        let mut approx = Vec3::new(0.0, 0.0, -10.0);
        let mut actual = Vec3::new(0.0, 0.0, -10.0);
        let mut index = 0;
        for y in 0..=(iterations + 1) {
            let t = y;
            let y = (y as f64 - (iterations as f64 / 2.0)) / (iterations as f64 / 2.0);
            let ray = Ray::new(pos, DVec3::new(0.0, y, 1.0));
            let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0);
            if actual_dir.is_some() {
                let approximate_dir = ray_cache.fetch_final_dir(ray.dir.z as f32).unwrap();
                let actual_dir = cast_ray_steps(&ray, &field, 20.0, 100.0).unwrap();
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
