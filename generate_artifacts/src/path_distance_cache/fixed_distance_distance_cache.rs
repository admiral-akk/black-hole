use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::{
    path_distance_cache::fixed_distance_fixed_angle_distance_cache::{
        ANGLE_DISTANCE_CACHE_SIZE, MIN_ANGLE,
    },
    path_integration2::response::Response,
};

use super::fixed_distance_fixed_angle_distance_cache::FixedDistanceFixedAngleDistanceCache;

use crate::path_integration2::path::find_optimal_z;
pub const DISTANCE_CACHE_SIZE: usize = 1 << 5;
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FixedDistanceDistanceCache {
    pub min_angle: f64,
    pub min_z: f64,
    pub camera_distance: f64,
    pub black_hole_radius: f64,
    pub disc_bounds: (f64, f64),
    pub angle_to_z_to_distance: Vec<FixedDistanceFixedAngleDistanceCache>,
}
// use this find z values where we don't have to apply anti-aliasing
fn find_grazing_z(camera_distance: f64, black_hole_radius: f64, target_dist: f64) -> f64 {
    // if we're too close, any direction could hit the disc.
    if camera_distance < target_dist {
        return -1.;
    }
    let too_close = move |r: Response| {
        r.hits_black_hole()
            || r.path
                .iter()
                .map(|p| p.length())
                .fold(f64::INFINITY, f64::min)
                < target_dist
    };
    find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        (-1., 1.),
        &too_close,
    )
    .0
}

fn float_01_to_left_index(float_01: f64, vec_len: usize) -> (usize, f64) {
    let float_index = (vec_len - 1) as f64 * float_01;
    let index = (float_index as usize).clamp(0, vec_len - 2);
    let t = float_index - index as f64;
    (index, t)
}
fn index_to_float_01(index: usize, vec_len: usize) -> f64 {
    let float_01 = (index as f64) / (vec_len - 1) as f64;
    float_01.clamp(0., 1.)
}
impl FixedDistanceDistanceCache {
    pub fn compute_new(
        _cache_size: (usize, usize),
        camera_distance: f64,
        black_hole_radius: f64,
        disc_bounds: (f64, f64),
    ) -> Self {
        let mut angle_to_z_to_distance = Vec::new();

        for i in 0..DISTANCE_CACHE_SIZE {
            let float_01 = index_to_float_01(i, DISTANCE_CACHE_SIZE);
            let angle = (TAU - MIN_ANGLE) * float_01 + MIN_ANGLE;
            println!("Generating: {:?}", (angle));
            let z_to_distance_cache = FixedDistanceFixedAngleDistanceCache::compute_new(
                ANGLE_DISTANCE_CACHE_SIZE,
                camera_distance,
                black_hole_radius,
                disc_bounds,
                angle,
            );
            angle_to_z_to_distance.push(z_to_distance_cache);
        }
        let _min_z = find_grazing_z(camera_distance, black_hole_radius, disc_bounds.1);
        FixedDistanceDistanceCache {
            min_angle: MIN_ANGLE,
            min_z: 0.0,
            camera_distance,
            black_hole_radius,
            disc_bounds,
            angle_to_z_to_distance,
        }
    }

    pub fn get_z_bounds(&self, angle_01: f64) -> (f64, f64) {
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
        let z_bound = self.get_z_bounds(angle_01);
        let _diff = z_bound.1 - z_bound.0;
        let z_01 = (z - z_bound.0) / (z_bound.1 - z_bound.0);
        Some(t * right.get_dist(z_01) + (1. - t) * left.get_dist(z_01))
    }
}

#[cfg(test)]
mod tests {
    use std::{f64::consts::TAU};

    use serde::{Deserialize, Serialize};
    use test_utils::plot_trajectories;

    use crate::{
        path_distance_cache::fixed_distance_fixed_angle_distance_cache::{
            ANGLE_DISTANCE_CACHE_SIZE, MIN_ANGLE,
        },
        path_integration2::path::cast_ray_steps_response,
    };

    use super::{FixedDistanceDistanceCache, DISTANCE_CACHE_SIZE};
    #[test]
    fn fixed_distance_test_error() {
        let cache_size = (DISTANCE_CACHE_SIZE, ANGLE_DISTANCE_CACHE_SIZE);
        let distance = 5.0;
        let black_hole_radius = 1.5;
        let max_disc_radius = (1.5, 12.0);
        let mut lines = Vec::new();

        let cache = FixedDistanceDistanceCache::compute_new(
            cache_size,
            distance,
            black_hole_radius,
            max_disc_radius,
        );
        let angle_iterations = 2 * cache_size.0;
        let distance_iterations = 2 * cache_size.1;
        for j in 0..=angle_iterations {
            let mut line = Vec::new();
            let angle_01 = (j as f64) / (angle_iterations as f64);
            let angle = (TAU - MIN_ANGLE) * angle_01 + MIN_ANGLE;
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
                if true_path.get_max_angle() < angle {
                    continue;
                }
                assert!(
                    true_path.get_max_angle() >= angle,
                    "\nTrue path is too shallow!\nMax angle: {}\nCache angle: {}\nz: {}\napprox_dist: {}",
                    true_path.get_max_angle(),
                    angle,
                    z,
                    approx_dist
                );
                let true_dist = true_path.get_dist(angle).unwrap();
                if (true_dist - approx_dist).abs() > 0.1 {
                    println!(
                        "Angle: {}, z_01: {}, data: {:?}",
                        angle,
                        z_01,
                        (z_01 as f32, (true_dist - approx_dist).abs() as f32,)
                    );
                }
                line.push((z_01 as f32, (true_dist - approx_dist).abs() as f32));
            }
            lines.push(line);
        }
        plot_trajectories(
            "output/angle_cache/fixed_distance_error_rates.png",
            &lines,
            ((0., 1.), (0., 4.0)),
        )
        .unwrap();
    }

    #[test]
    fn serialization() {
        let cache_size = (DISTANCE_CACHE_SIZE, ANGLE_DISTANCE_CACHE_SIZE);
        let distance = 5.0;
        let black_hole_radius = 1.5;
        let max_disc_radius = (1.5, 12.0);
        let cache = FixedDistanceDistanceCache::compute_new(
            cache_size,
            distance,
            black_hole_radius,
            max_disc_radius,
        );

        // Get a serialized version of the input data as a `Bson`.
        let mut s = flexbuffers::FlexbufferSerializer::new();
        let serialized = cache.serialize(&mut s);
        assert!(serialized.is_ok());

        let r = flexbuffers::Reader::get_root(s.view()).unwrap();

        // Serialization is similar to JSON. Field names are stored in the buffer but are reused
        // between all maps and structs.

        let deserialized = FixedDistanceDistanceCache::deserialize(r);
        assert!(deserialized.is_ok());
        assert_eq!(cache, deserialized.unwrap());
    }
}
