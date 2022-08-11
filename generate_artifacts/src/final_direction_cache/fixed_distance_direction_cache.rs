use std::f64::consts::TAU;

use crate::path_integration2::{
    path::cast_ray_steps_response,
    path::find_optimal_z,
    response::{Response, ToAngle},
};
use glam::DVec3;
use serde::{Deserialize, Serialize};

const ANGLE_EPSILON: f64 = 0.001;
const LINEAR_SCALE: f64 = 20.;
const MAX_ANGLE: f64 = TAU;
const POW_F: f64 = 16.0;

fn index_to_z_01(i: usize, cache_size: usize) -> f64 {
    let z_01 = (i as f64) / (cache_size - 1) as f64;
    let mut z_01_high = z_01.powf(1. / POW_F).clamp(0., 1.);
    let mut z_01_low = (LINEAR_SCALE * z_01).clamp(0., 1.);
    f64::min(z_01_low, z_01_high)
}

fn z_01_to_left_index(z_01: f64, cache_size: usize) -> (usize, f64) {
    let mut z_01_high = z_01.powf(POW_F).clamp(0., 1.);
    let mut z_01_low = (z_01 / LINEAR_SCALE).clamp(0., 1.);

    let z_01 = f64::max(z_01_low, z_01_high) * (cache_size - 1) as f64;

    let index = (z_01 as usize).clamp(0, cache_size - 2);
    let t = z_01 - index as f64;
    (index, t)
}

fn index_to_z(max_z: f64, i: usize, cache_size: usize) -> f64 {
    let float_01 = index_to_z_01(i, cache_size);
    (max_z + 1.0) * float_01 - 1.0
}
fn z_to_left_index(max_z: f64, z: f64, cache_size: usize) -> (usize, f64) {
    let z_01 = ((z + 1.0) / (max_z + 1.0));
    z_01_to_left_index(z_01, cache_size)
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FixedDistanceDirectionCache {
    pub max_z: f64,
    pub min_z: f64,
    pub camera_distance: f64,
    pub black_hole_radius: f64,
    pub z_to_final_dir: Vec<(f64, (f64, f64))>,
}

fn find_closest_z(camera_distance: f64, black_hole_radius: f64) -> f64 {
    let too_close =
        |r: Response| r.hits_black_hole() || r.get_angle_dist().get_max_angle() > MAX_ANGLE;
    find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        (-1., 1.),
        &too_close,
    )
    .0
}

// use this find z values where we don't have to apply anti-aliasing
fn find_minimum_pertubation_z(camera_distance: f64, black_hole_radius: f64, max_z: f64) -> f64 {
    let too_close = |r: Response| {
        let initial_dir = (r.path[1] - r.path[0]).get_angle();
        r.hits_black_hole() || r.get_angle_dist().get_max_angle() - initial_dir > ANGLE_EPSILON
    };
    find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        (-1., max_z),
        &too_close,
    )
    .0
}

impl FixedDistanceDirectionCache {
    pub fn compute_new(cache_size: usize, camera_distance: f64, black_hole_radius: f64) -> Self {
        let max_z = find_closest_z(camera_distance, black_hole_radius);
        let min_z = find_minimum_pertubation_z(camera_distance, black_hole_radius, max_z);
        let mut z_to_final_dir = Vec::new();
        for i in 0..cache_size {
            let z = index_to_z(max_z, i, cache_size);
            let final_dir =
                cast_ray_steps_response(z, camera_distance, black_hole_radius).final_dir;
            if final_dir.is_none() {
                panic!("Should always miss black hole!\nmax_z: {}\nz: {}", max_z, z);
            }
            let final_dir = final_dir.unwrap().normalize();
            z_to_final_dir.push((z, (final_dir.x, final_dir.z)));
        }
        println!("dist:{}\nMin_z: {}", camera_distance, min_z);
        FixedDistanceDirectionCache {
            max_z,
            min_z,
            camera_distance,
            black_hole_radius,
            z_to_final_dir,
        }
    }

    pub fn get_final_dir(&self, z_01: f64) -> DVec3 {
        let (index, t) = z_01_to_left_index(z_01, self.z_to_final_dir.len());
        let left = self.z_to_final_dir[index].1;
        let right = self.z_to_final_dir[index + 1].1;
        DVec3::new(
            t * right.0 + (1. - t) * left.0,
            0.0,
            t * right.1 + (1. - t) * left.1,
        )
    }
}

#[cfg(test)]
mod tests {

    use test_utils::plot_trajectories;

    use crate::{
        final_direction_cache::fixed_distance_direction_cache::{index_to_z, index_to_z_01},
        path_integration2::path::cast_ray_steps_response,
    };

    use super::FixedDistanceDirectionCache;

    #[test]
    fn fixed_distance_show_index_distribution() {
        let cache_size = 1 << 10;
        let camera_distance = 5.0;
        let black_hole_radius = 1.5;

        let mut errors = Vec::new();
        for pow in [1., 2., 4., 8., 16., 32.] {
            let cache = FixedDistanceDirectionCache::compute_new(
                cache_size,
                camera_distance,
                black_hole_radius,
            );
            let mut line = Vec::new();

            for i in 0..cache.z_to_final_dir.len() {
                let z_01 = index_to_z_01(i, cache_size);
                line.push(((i as f32) / (cache_size - 1) as f32, z_01 as f32));
            }
            errors.push(line);
        }
        plot_trajectories(
            "output/final_direction_cache/index_distribution.png",
            &errors,
            ((0., 1.), (0., 1.)),
        )
        .unwrap();
    }

    #[test]
    fn fixed_distance_show_errors_test() {
        let cache_size = 1 << 10;
        let camera_distance = 20.0;
        let black_hole_radius = 1.5;

        let mut errors = Vec::new();
        for camera_distance in [2., 5., 10., 15., 20.] {
            let cache = FixedDistanceDirectionCache::compute_new(
                cache_size,
                camera_distance,
                black_hole_radius,
            );

            let mut line = Vec::new();

            for i in 0..cache.z_to_final_dir.len() {
                let z = index_to_z(cache.max_z, i, cache_size);
                let curr_angle =
                    cast_ray_steps_response(z, cache.camera_distance, cache.black_hole_radius)
                        .get_angle_dist()
                        .get_max_angle();
                println!("z: {}, Final dir: {:?}", z, curr_angle);
                line.push(((i as f32) / (cache_size - 1) as f32, curr_angle as f32));
            }
            errors.push(line);
        }
        plot_trajectories(
            "output/final_direction_cache/error_function.png",
            &errors,
            ((0., 1.), (0., 10.)),
        )
        .unwrap();
    }

    #[test]
    fn fixed_distance_show_paths_test() {
        let cache_size = 1 << 10;
        let camera_distance = 2.0;
        let black_hole_radius = 1.5;

        let cache = FixedDistanceDirectionCache::compute_new(
            cache_size,
            camera_distance,
            black_hole_radius,
        );

        let mut paths = Vec::new();

        for z in &cache.z_to_final_dir {
            let z = z.0;
            let response =
                cast_ray_steps_response(z, cache.camera_distance, cache.black_hole_radius);
            paths.push(
                response
                    .path
                    .iter()
                    .map(|v| (v.x as f32, v.z as f32))
                    .collect(),
            );
        }
        plot_trajectories(
            "output/final_direction_cache/fixed_distance_paths.png",
            &paths,
            ((-10., 10.), (-10., 10.)),
        )
        .unwrap();
    }

    #[test]
    fn fixed_distance_direction_test() {
        let cache_size = 1 << 11;
        let black_hole_radius = 1.5;
        let mut lines = Vec::new();
        let mut samples = Vec::new();
        for camera_distance in [5., 10., 15., 20.] {
            let cache = FixedDistanceDirectionCache::compute_new(
                cache_size,
                camera_distance,
                black_hole_radius,
            );

            for i in 0..(2 * cache_size) {
                let z_01 = i as f64 / (2 * cache_size - 1) as f64;
                samples.push(z_01);
                if z_01 > 0. && z_01 < 1. {
                    samples.push(z_01 * z_01);
                    samples.push(z_01.sqrt());
                }
            }
            samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let mut line = Vec::new();
            for z_01 in &samples {
                let approx_final_dir = cache.get_final_dir(*z_01);
                let z = (cache.max_z + 1.) * z_01 - 1.;
                let response =
                    cast_ray_steps_response(z, cache.camera_distance, cache.black_hole_radius);

                let true_final_dir = response.final_dir;
                assert!(
                    true_final_dir.is_some(),
                    "\nTrue path is too shallow!\nMaxz: {}\nCache final dir: {:?}\nz: {}",
                    cache.max_z,
                    approx_final_dir,
                    z
                );
                let true_final_dir = true_final_dir.unwrap();
                let error = (true_final_dir - approx_final_dir).length();
                if error > 0.1 {
                    println!("z: {}\nerror: {}", z, error);
                }
                line.push((*z_01 as f32, error as f32));
            }
            lines.push(line);
        }
        plot_trajectories(
            "output/final_direction_cache/fixed_distance_error_rates.png",
            &lines,
            ((0., 1.), (0., 1.0)),
        )
        .unwrap();
    }

    #[test]
    fn serialization() {
        let cache_size = 16;
        let distance = 10.0;
        let black_hole_radius = 1.5;
        let cache =
            FixedDistanceDirectionCache::compute_new(cache_size, distance, black_hole_radius);

        let serialized = serde_json::to_string(&cache);

        assert!(serialized.is_ok());

        let deserialized: Result<FixedDistanceDirectionCache, serde_json::Error> =
            serde_json::from_str(serialized.unwrap().as_str());

        assert!(deserialized.is_ok());

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized, cache);
    }
}
