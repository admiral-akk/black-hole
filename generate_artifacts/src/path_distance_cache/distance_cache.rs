use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::path_distance_cache::fixed_distance_fixed_angle_distance_cache::MIN_ANGLE;

use super::{
    fixed_distance_distance_cache::{FixedDistanceDistanceCache, DISTANCE_CACHE_SIZE},
    fixed_distance_fixed_angle_distance_cache::{
        FixedDistanceFixedAngleDistanceCache, ANGLE_DISTANCE_CACHE_SIZE,
    },
};

pub const ALL_DISTANCE_CACHE_SIZE: usize = 1 << 5;
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DistanceCache {
    pub cache_size: (usize, usize, usize),
    pub distance_bounds: (f64, f64),
    pub black_hole_radius: f64,
    pub disc_bounds: (f64, f64),
    pub distance_angle_to_z_to_distance: Vec<FixedDistanceDistanceCache>,
}

fn float_01_to_left_index(float_01: f64, vec_len: usize) -> (usize, f64) {
    let float_index = (vec_len - 1) as f64 * float_01;
    let index = (float_index as usize).clamp(0, vec_len - 2);
    let t = float_index - index as f64;
    (index, t)
}
fn index_to_float_01(index: usize, vec_len: usize) -> f64 {
    let float_01 = (index as f64) / (vec_len - 1) as f64;
    return float_01.clamp(0., 1.);
}
impl DistanceCache {
    pub fn compute_new(
        cache_size: (usize, usize, usize),
        distance_bounds: (f64, f64),
        black_hole_radius: f64,
        disc_bounds: (f64, f64),
    ) -> Self {
        let cache_size = (
            ALL_DISTANCE_CACHE_SIZE,
            DISTANCE_CACHE_SIZE,
            ANGLE_DISTANCE_CACHE_SIZE,
        );
        let mut distance_angle_to_z_to_distance = Vec::new();

        for i in 0..cache_size.0 {
            let float_01 = index_to_float_01(i, cache_size.0);
            let distance = (distance_bounds.1 - distance_bounds.0) * float_01 + distance_bounds.0;
            println!("Generating: {:?}", (distance));
            let angle_to_z_to_distance = FixedDistanceDistanceCache::compute_new(
                (cache_size.1, cache_size.2),
                distance,
                black_hole_radius,
                disc_bounds,
            );
            distance_angle_to_z_to_distance.push(angle_to_z_to_distance);
        }
        DistanceCache {
            cache_size,
            distance_bounds,
            black_hole_radius,
            disc_bounds,
            distance_angle_to_z_to_distance,
        }
    }

    pub fn get_z_bounds(&self, distance_01: f64, angle: f64) -> (f64, f64) {
        let (index, t) =
            float_01_to_left_index(distance_01, self.distance_angle_to_z_to_distance.len());
        let angle_01 = angle / TAU;
        let left = &self.distance_angle_to_z_to_distance[index].get_z_bounds(angle_01);
        let right = &self.distance_angle_to_z_to_distance[index + 1].get_z_bounds(angle_01);
        let lower_z_bound = right.0 * t + (1. - t) * left.0;
        let upper_z_bound = right.1 * t + (1. - t) * left.1;
        (lower_z_bound, upper_z_bound)
    }

    pub fn get_dist(&self, distance_01: f64, angle: f64, z: f64) -> Option<f64> {
        let angle_01 = angle / TAU;
        let (index, t) =
            float_01_to_left_index(distance_01, self.distance_angle_to_z_to_distance.len());
        let left = &self.distance_angle_to_z_to_distance[index];
        let right = &self.distance_angle_to_z_to_distance[index + 1];
        Some(
            t * right.get_dist(angle_01, z).unwrap()
                + (1. - t) * left.get_dist(angle_01, z).unwrap(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{f64::consts::TAU, fs};

    use serde::{Deserialize, Serialize};
    use test_utils::plot_trajectories;

    use crate::{
        path_distance_cache::{
            fixed_distance_distance_cache::DISTANCE_CACHE_SIZE,
            fixed_distance_fixed_angle_distance_cache::{ANGLE_DISTANCE_CACHE_SIZE, MIN_ANGLE},
        },
        path_integration2::path::cast_ray_steps_response,
    };

    use super::{DistanceCache, FixedDistanceDistanceCache, ALL_DISTANCE_CACHE_SIZE};
    #[test]
    fn full_test_error() {
        let cache_size = (
            ALL_DISTANCE_CACHE_SIZE,
            DISTANCE_CACHE_SIZE,
            ANGLE_DISTANCE_CACHE_SIZE,
        );
        let black_hole_radius = 1.5;
        let distance = (5., 20.0);
        let max_disc_radius = (1.5, 12.0);
        let mut lines = Vec::new();
        let cache =
            DistanceCache::compute_new(cache_size, distance, black_hole_radius, max_disc_radius);
        let distance_iterations = 2 * cache_size.0;
        let angle_iterations = 2 * cache_size.1;
        let z_iterations = 2 * cache_size.2;
        for j in 0..=distance_iterations {
            let mut line = Vec::new();
            let dist_01 = (j as f64) / (distance_iterations as f64);
            let dist = (distance.1 - distance.0) * dist_01 + distance.0;
            for k in 0..=angle_iterations {
                let angle_01 = (k as f64) / (angle_iterations as f64);
                let angle = (TAU - MIN_ANGLE) * angle_01 + MIN_ANGLE;
                let z_bounds = cache.get_z_bounds(dist_01, angle);
                let mut sq_err = 0.0;
                let mut total_samples = 0;
                for l in 0..z_iterations {
                    let z_01 = (l as f64) / (z_iterations as f64);
                    let z = (z_bounds.1 - z_bounds.0) * z_01 + z_bounds.0;
                    let approx_dist = cache.get_dist(dist_01, angle, z);
                    assert!(
                        approx_dist.is_some(),
                        "\nApprox dist missing value!\nAngle: {}\nz: {}",
                        angle,
                        z
                    );
                    let approx_dist = approx_dist.unwrap();
                    let true_path =
                        cast_ray_steps_response(z, dist, cache.black_hole_radius).get_angle_dist();
                    let true_dist = true_path.get_dist(angle);
                    if true_dist.is_none() {
                        continue;
                    }
                    let true_dist = true_dist.unwrap();
                    if (true_dist - approx_dist).abs() > 0.1 {
                        println!(
                            "Dist: {}, Angle: {}, z_01: {}, data: {:?}",
                            dist,
                            angle,
                            z_01,
                            (z_01 as f32, (true_dist - approx_dist).abs() as f32,)
                        );
                    }
                    total_samples += 1;
                    sq_err += (true_dist - approx_dist).powi(2);
                }
                assert!(
                    total_samples >= 0,
                    "\nNo valid samples found!: Distance: {}, angle: {}",
                    dist,
                    angle,
                );
                sq_err = sq_err.sqrt() / total_samples as f64;
                line.push((angle as f32, sq_err as f32));
            }
            lines.push(line);
        }
        plot_trajectories(
            "output/angle_cache/error_rates.png",
            &lines,
            ((0., 1.), (0., 1.0)),
        )
        .unwrap();
    }

    #[test]
    fn serialization() {
        let cache_size = (16, 16);
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
