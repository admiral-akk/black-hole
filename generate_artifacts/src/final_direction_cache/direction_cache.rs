use std::f64::consts::TAU;

use glam::DVec3;
use serde::{Deserialize, Serialize};

pub const DIRECTION_CACHE_SIZE: usize = 1 << 5;
use super::fixed_distance_direction_cache::{FixedDistanceDirectionCache, DISTANCE_CACHE_SIZE};
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DirectionCache {
    pub cache_size: (usize, usize),
    pub distance_bounds: (f64, f64),
    pub black_hole_radius: f64,
    pub distance_angle_to_z_to_distance: Vec<FixedDistanceDirectionCache>,
}

fn d_01_to_left_index(d_01: f64, vec_len: usize) -> (usize, f64) {
    let float_index = (vec_len - 1) as f64 * d_01;
    let index = (float_index as usize).clamp(0, vec_len - 2);
    let t = float_index - index as f64;
    (index, t)
}
fn index_to_float_01(index: usize, vec_len: usize) -> f64 {
    let float_01 = (index as f64) / (vec_len - 1) as f64;
    return float_01.clamp(0., 1.);
}

impl DirectionCache {
    pub fn compute_new(
        cache_size: (usize, usize),
        distance_bounds: (f64, f64),
        black_hole_radius: f64,
    ) -> Self {
        let mut distance_angle_to_z_to_distance = Vec::new();
        for i in 0..DIRECTION_CACHE_SIZE {
            let dist = (distance_bounds.1 - distance_bounds.0) as f64 * i as f64
                / (DIRECTION_CACHE_SIZE - 1) as f64
                + distance_bounds.0;
            println!("Generating dist: {}", dist);
            let fixed_distance_cache =
                FixedDistanceDirectionCache::compute_new(dist, black_hole_radius);
            distance_angle_to_z_to_distance.push(fixed_distance_cache);
        }
        DirectionCache {
            cache_size: (DIRECTION_CACHE_SIZE, DISTANCE_CACHE_SIZE),
            distance_bounds,
            black_hole_radius,
            distance_angle_to_z_to_distance,
        }
    }
    pub fn get_z_bounds(&self, d_01: f64) -> (f64, f64) {
        let (index, t) = d_01_to_left_index(d_01, self.distance_angle_to_z_to_distance.len());
        let left = (
            self.distance_angle_to_z_to_distance[index].min_z,
            self.distance_angle_to_z_to_distance[index].max_z,
        );
        let right = (
            self.distance_angle_to_z_to_distance[index + 1].min_z,
            self.distance_angle_to_z_to_distance[index + 1].max_z,
        );
        (
            right.0 * t + (1. - t) * left.0,
            right.1 * t + (1. - t) * left.1,
        )
    }

    pub fn get_final_dir(&self, d_01: f64, z: f64) -> DVec3 {
        let (index, t) = d_01_to_left_index(d_01, self.distance_angle_to_z_to_distance.len());
        let z_bounds = self.get_z_bounds(d_01);
        let z_01 = ((z - z_bounds.0) / (z_bounds.1 - z_bounds.0)).clamp(0., 1.);
        let left = self.distance_angle_to_z_to_distance[index].get_final_dir(z_01);
        let right = self.distance_angle_to_z_to_distance[index + 1].get_final_dir(z_01);
        t * right + (1. - t) * left
    }
}

#[cfg(test)]
mod tests {

    use test_utils::plot_trajectories;

    use crate::{
        final_direction_cache::{
            direction_cache::DirectionCache, fixed_distance_direction_cache::DISTANCE_CACHE_SIZE,
        },
        path_integration2::path::cast_ray_steps_response,
    };

    use super::DIRECTION_CACHE_SIZE;

    #[test]
    fn all_distance_direction_test() {
        let cache_size = (DIRECTION_CACHE_SIZE, DISTANCE_CACHE_SIZE);
        let distance = (5.0, 30.);
        let black_hole_radius = 1.5;
        let cache = DirectionCache::compute_new(cache_size, distance, black_hole_radius);

        let mut lines = Vec::new();
        let mut samples = Vec::new();
        for i in 0..(4 * cache_size.1) {
            let z_01 = i as f64 / (4 * cache_size.1 - 1) as f64;
            samples.push(z_01);
            if z_01 > 0. && z_01 < 1. {
                samples.push(z_01 * z_01);
                samples.push(z_01.sqrt());
            }
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut dist_samples = Vec::new();
        for i in 0..10 {
            let d_01 = i as f64 / (2 * cache_size.0 - 1) as f64;
            dist_samples.push(d_01);
            if d_01 > 0. && d_01 < 1. {
                dist_samples.push(d_01 * d_01);
                dist_samples.push(d_01.sqrt());
            }
        }
        dist_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for d_01 in &dist_samples {
            let mut line = Vec::new();
            for z_01 in &samples {
                let z = 2. * z_01 - 1.;
                let z_bounds = cache.get_z_bounds(*d_01);
                if z < z_bounds.0 || z > z_bounds.1 {
                    println!("Out of bounds, z: {}", z);
                    continue;
                }

                let dist = (distance.1 - distance.0) * d_01 + distance.0;
                let approx_final_dir = cache.get_final_dir(*d_01, z);
                let response = cast_ray_steps_response(z, dist, cache.black_hole_radius);

                let true_final_dir = response.final_dir;
                if true_final_dir.is_none() {
                    println!("No val at z:{}, dist:{}", z, dist);
                    continue;
                }
                let true_final_dir = true_final_dir.unwrap();
                let error = (true_final_dir - approx_final_dir).length();
                if error > 0.1 {
                    println!("z: {}\ndist: {}\nerror: {}", z, dist, error);
                }
                line.push((*z_01 as f32, error as f32));
            }
            lines.push(line);
        }

        plot_trajectories(
            "output/final_direction_cache/all_distance_error_rates.png",
            &lines,
            ((0., 1.), (0., 4.0)),
        )
        .unwrap();
    }

    #[test]
    fn serialization() {
        let cache_size = (DIRECTION_CACHE_SIZE, DISTANCE_CACHE_SIZE);
        let distance = (5.0, 20.);
        let black_hole_radius = 1.5;
        let cache = DirectionCache::compute_new(cache_size, distance, black_hole_radius);

        let serialized = serde_json::to_string(&cache);

        assert!(serialized.is_ok());

        let deserialized: Result<DirectionCache, serde_json::Error> =
            serde_json::from_str(serialized.unwrap().as_str());

        assert!(deserialized.is_ok());

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized, cache);
    }
}
