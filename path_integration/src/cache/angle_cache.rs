use glam::{DVec3, Vec3};

use crate::{cast_ray_steps, Field, Ray};

use super::ray_cache::RAY_START_DIR;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AngleCache {
    pub cache: Vec<AngleCacheAnswer>,
    pub min_z: f32,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AngleCacheAnswer {
    pub z: f32,
    pub angle: f32,
    // this keeps track of where the ray might hit the accretion disc
    pub angle_dist: Vec<(f32, f32)>,
}

fn find_angle(
    cache_pos: &Vec3,
    field: &Field,
    epsilon: f64,
    max_distance: f64,
    ring_radius: f64,
    target_angle: f64,
) -> f64 {
    let (mut miss_z, mut hit_z) = (-1.0, 1.0);
    while hit_z - miss_z > epsilon {
        let z = (miss_z + hit_z) / 2.;
        // println!("test: {}", z);
        let test = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
        let ray = Ray::new(cache_pos.as_dvec3(), test);
        let path = cast_ray_steps(&ray, field, max_distance, 10.0 * max_distance).0;
        let angles = to_angles(&path);
        let example = angles.iter().find(|x| x.0 >= target_angle as f32);

        if example.is_some() {
            let dist = example.unwrap().1 as f64;
            // could hit the disc, check the angle of the hit.
            if dist < ring_radius {
                hit_z = test.z;
            } else {
                miss_z = test.z;
            }
        } else {
            let final_delta = angles[angles.len() - 1].1 - angles[angles.len() - 2].1;
            if final_delta > 0.0 {
                miss_z = test.z;
            } else {
                hit_z = test.z;
            }
        }
    }
    hit_z
}

fn to_angle_dist(pos: &DVec3) -> (f32, f32) {
    let mut angle = f64::atan2(pos.x, -pos.z) as f32;
    if angle < 0.0 {
        angle += std::f32::consts::TAU;
    }
    return (angle, pos.length() as f32);
}

fn to_angles(path: &Vec<DVec3>) -> Vec<(f32, f32)> {
    let mut output: Vec<(f32, f32)> = Vec::new();
    let mut offset = 0.0;
    for i in 0..path.len() {
        let angle_dist = to_angle_dist(&path[i]);
        if output.len() > 0 && output[output.len() - 1].0 > angle_dist.0 + offset {
            offset += std::f32::consts::TAU;
        }
        output.push((angle_dist.0 + offset, angle_dist.1));
    }
    output
}

impl AngleCache {
    pub fn compute_new(
        size: usize,
        black_hole_radius: f32,
        camera_distance: f32,
        max_disc_radius: f32,
    ) -> Self {
        let mut cache = Vec::new();
        let field = Field::new(black_hole_radius as f64, camera_distance as f64);

        let cache_pos = camera_distance * RAY_START_DIR;

        let max_distance = 10.0 * camera_distance as f64;
        let epsilon = 0.00000001;
        let ring_radius = max_disc_radius as f64;

        for i in 0..(size + 1) {
            let target_angle = std::f64::consts::TAU * i as f64 / size as f64;
            let z = find_angle(
                &cache_pos,
                &field,
                epsilon,
                max_distance,
                ring_radius,
                target_angle,
            );
            let dir = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
            let ray = Ray::new(cache_pos.as_dvec3(), dir);
            let path = cast_ray_steps(&ray, &field, max_distance, 10.0 * max_distance).0;

            let angles = to_angles(&path);

            cache.push(AngleCacheAnswer {
                z: ray.dir.z as f32,
                angle: target_angle as f32,
                angle_dist: angles,
            });
        }
        Self {
            cache,
            min_z: 1.0 as f32,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::cache::angle_cache::AngleCache;

    #[test]
    fn serialization() {
        let distance = 10.0;
        let max_disc_radius = 6.0;
        let r = 1.0;
        let cache_size = 5;
        let angle_cache = AngleCache::compute_new(cache_size, r, distance, max_disc_radius);

        let serialized = serde_json::to_string(&angle_cache);

        assert!(serialized.is_ok());

        let deserialized: Result<AngleCache, serde_json::Error> =
            serde_json::from_str(serialized.unwrap().as_str());

        assert!(deserialized.is_ok());

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized, angle_cache);
    }
}
