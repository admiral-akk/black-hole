use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use super::fixed_distance_fixed_angle_distance_cache::FixedDistanceFixedAngleDistanceCache;

const MIN_ANGLE: f64 = TAU * (0.1 / 360.);
const Z_EPSILON: f64 = 0.000000001;
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FixedDistanceDistanceCache {
    pub min_angle: f64,
    pub camera_distance: f64,
    pub black_hole_radius: f64,
    pub disc_bounds: (f64, f64),
    pub angle_to_z_to_distance: Vec<FixedDistanceFixedAngleDistanceCache>,
}

fn float_01_to_left_index(mut float_01: f64, vec_len: usize) -> (usize, f64) {
    let float_index = (vec_len - 1) as f64 * float_01;
    let mut index = float_index as usize;
    if index == vec_len - 1 {
        index -= 1;
    }
    let t = float_index - index as f64;
    (index, t)
}
fn index_to_float_01(index: usize, vec_len: usize) -> f64 {
    let mut float_01 = (index as f64) / (vec_len - 1) as f64;
    float_01
}
impl FixedDistanceDistanceCache {
    pub fn compute_new(
        cache_size: (usize, usize),
        camera_distance: f64,
        black_hole_radius: f64,
        disc_bounds: (f64, f64),
    ) -> Self {
        let mut angle_to_z_to_distance = Vec::new();

        for i in 0..cache_size.0 {
            let float_01 = index_to_float_01(i, cache_size.0);
            let angle = (TAU - MIN_ANGLE) * float_01 + MIN_ANGLE;
            println!("Generating: {:?}", (angle));
            let z_to_distance_cache = FixedDistanceFixedAngleDistanceCache::compute_new(
                cache_size.1,
                camera_distance,
                black_hole_radius,
                disc_bounds,
                angle,
            );
            angle_to_z_to_distance.push(z_to_distance_cache);
        }
        FixedDistanceDistanceCache {
            min_angle: MIN_ANGLE,
            camera_distance,
            black_hole_radius,
            disc_bounds,
            angle_to_z_to_distance,
        }
    }

    fn get_z_bounds(&self, angle_01: f64) -> (f64, f64) {
        let (index, t) = float_01_to_left_index(angle_01, self.angle_to_z_to_distance.len());
        let left = &self.angle_to_z_to_distance[index];
        let right = &self.angle_to_z_to_distance[index + 1];
        let lower_z_bound = right.z_bounds.0 * t + (1. - t) * left.z_bounds.0;
        let upper_z_bound = right.z_bounds.1 * t + (1. - t) * left.z_bounds.1;
        (lower_z_bound, upper_z_bound)
    }

    pub fn get_dist(&self, angle_01: f64, z: f64) -> Option<f64> {
        let (index, t) = float_01_to_left_index(angle_01, self.angle_to_z_to_distance.len());
        let left = &self.angle_to_z_to_distance[index];
        let right = &self.angle_to_z_to_distance[index + 1];
        let lower_z_bound = right.z_bounds.0 * t + (1. - t) * left.z_bounds.0;
        let upper_z_bound = right.z_bounds.1 * t + (1. - t) * left.z_bounds.1;
        if z > upper_z_bound || z < lower_z_bound {
            return None;
        }
        let z_01 = (z - lower_z_bound) / (upper_z_bound - lower_z_bound);
        Some(t * right.get_dist(z_01) + (1. - t) * left.get_dist(z_01))
    }
}

#[cfg(test)]
mod tests {
    use std::{f64::consts::TAU, fs};

    use test_utils::plot_trajectories;

    use crate::cast_ray_steps_response;

    use super::FixedDistanceDistanceCache;
    #[test]
    fn fixed_distance_test_error() {
        let cache_size = (512, 64);
        let distance = 17.0;
        let black_hole_radius = 1.5;
        let max_disc_radius = (3.0, 6.0);
        let mut lines = Vec::new();

        let cache = serde_json::from_str::<FixedDistanceDistanceCache>(
            &fs::read_to_string("fixed_distance_distance_cache2.txt").unwrap(),
        )
        .unwrap();
        let angle_iterations = 256;
        let distance_iterations = 1024;
        for j in 0..=angle_iterations {
            let mut line = Vec::new();
            let angle_01 = (j as f64) / (angle_iterations as f64);
            let angle = TAU * angle_01;
            let z_bounds = cache.get_z_bounds(angle_01);
            for i in 0..=distance_iterations {
                let z_01 = (i as f64) / (distance_iterations as f64);
                let z = (z_bounds.1 - z_bounds.0) * z_01 + z_bounds.0;
                let approx_dist = cache.get_dist(angle_01, z);
                assert!(
                    approx_dist.is_some(),
                    "\nApprox dist missing value!\nAngle: {}\nz: {}",
                    angle,
                    z
                );
                let approx_dist = approx_dist.unwrap();
                let true_path =
                    cast_ray_steps_response(z, cache.camera_distance, cache.black_hole_radius)
                        .get_angle_dist();
                assert!(
                    true_path.get_max_angle() >= angle,
                    "\nTrue path is too shallow!\nMax angle: {}\nCache angle: {}\nz: {}\napprox_dist: {}",
                    true_path.get_max_angle(),
                    angle,
                    z,
                    approx_dist
                );
                let true_dist = true_path.get_dist(angle).unwrap();
                println!(
                    "Angle: {}, data: {:?}",
                    angle,
                    (z_01 as f32, (true_dist - approx_dist).abs() as f32,)
                );
                line.push((z_01 as f32, (true_dist - approx_dist).abs() as f32));
            }
            lines.push(line);
        }
        plot_trajectories(
            "output/angle_cache/fixed_distance_error_rates.png",
            &lines,
            ((0., 1.), (0., 1.0)),
        )
        .unwrap();
    }

    #[test]
    fn serialization() {
        let cache_size = (16, 16);
        let distance = 10.0;
        let black_hole_radius = 1.5;
        let max_disc_radius = (3.0, 6.0);
        let cache = FixedDistanceDistanceCache::compute_new(
            cache_size,
            distance,
            black_hole_radius,
            max_disc_radius,
        );

        let serialized = serde_json::to_string(&cache);

        assert!(serialized.is_ok());

        let deserialized: Result<FixedDistanceDistanceCache, serde_json::Error> =
            serde_json::from_str(serialized.unwrap().as_str());

        assert!(deserialized.is_ok());

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized, cache);
    }
}
