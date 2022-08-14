use std::f64::consts::TAU;
use std::fs::{self};

use generate_artifacts::black_hole_cache::BlackHoleCache;
use generate_artifacts::final_direction_cache::direction_cache::DirectionCache;
use generate_artifacts::path_distance_cache::distance_cache::DistanceCache;
use serde::{Deserialize, Serialize};

mod final_direction_cache;
mod path_distance_cache;
mod path_integration2;
const BLACK_HOLE_CACHE_PATH: &str = "generate_artifacts/output/black_hole_cache.txt";
const DISTANCE_TEST_PATH: &str = "generate_artifacts/output/distance_test_points.txt";
const DIRECTION_TEST_PATH: &str = "generate_artifacts/output/direction_test_points.txt";
use std::fs::File;
use std::io::Read;

fn get_file_as_byte_vec(filename: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut f = File::open(&filename)?;
    let metadata = fs::metadata(&filename)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer)?;

    Ok(buffer)
}

#[derive(Serialize, Deserialize)]
pub struct DirectionTestPoint {
    pub z: f64,
    pub dist: f64,
    pub final_angle: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct AngleTestPoint {
    pub z: f64,
    pub target_angle: f64,
    pub dist: f64,
    pub dist_at_angle: Option<f64>,
}

const DIRECTION_CACHE_SIZE: (usize, usize) = (1 << 5, 1 << 8);
const DISTANCE_CACHE_SIZE: (usize, usize, usize) = (1 << 4, 1 << 6, 1 << 4);
const DISTANCE_BOUNDS: (f64, f64) = (3.0, 30.0);
const BLACK_HOLE_RADIUS: f64 = 1.5;
const DISC_BOUNDS: (f64, f64) = (1.5, 12.0);
const DIST_TEST_POINTS: usize = 50;
const ANGLE_TEST_POINTS: usize = 45;
const Z_TEST_POINTS: usize = 2000;
use crate::path_integration2::path::cast_ray_steps_response;
use generate_artifacts::path_integration2::response::ToAngle;
fn generate_test_points() {
    let mut dist_test_points = Vec::new();
    let mut angle_test_points = Vec::new();
    for d_index in 0..DIST_TEST_POINTS {
        for z_index in 0..Z_TEST_POINTS {
            println!("Generating dist, d: {}, z: {}", d_index, z_index);
            let dist = (DISTANCE_BOUNDS.1 - DISTANCE_BOUNDS.0)
                * (d_index as f64 / (DIST_TEST_POINTS - 1) as f64)
                + DISTANCE_BOUNDS.0;
            let z = z_index as f64 / (Z_TEST_POINTS - 1) as f64;
            let res = cast_ray_steps_response(z, dist, BLACK_HOLE_RADIUS);
            let final_dir = res.final_dir;
            let mut final_angle = None;
            if final_dir.is_some() {
                final_angle = Some(final_dir.unwrap().get_angle())
            }
            let test_point = DirectionTestPoint {
                z,
                dist,
                final_angle,
            };
            dist_test_points.push(test_point);
            let angle_dist = res.get_angle_dist();
            for a_index in 0..ANGLE_TEST_POINTS {
                println!(
                    "Generating angle, d: {}, a: {}, z: {}",
                    d_index, a_index, z_index
                );
                let target_angle = TAU * a_index as f64 / (ANGLE_TEST_POINTS - 1) as f64;
                let dist_at_angle = angle_dist.get_dist(target_angle);

                let test_point = AngleTestPoint {
                    z,
                    target_angle,
                    dist,
                    dist_at_angle,
                };
                angle_test_points.push(test_point);
            }
        }
    }
    let data = serde_json::to_string(&dist_test_points).unwrap();
    println!("Writing distance test points out.");
    fs::write(DIRECTION_TEST_PATH, data).expect("Unable to write file");
    let data = serde_json::to_string(&angle_test_points).unwrap();
    println!("Writing angle test points out.");
    fs::write(DISTANCE_TEST_PATH, data).expect("Unable to write file");
}

fn regenerate_black_hole_cache() {
    println!("Attempting to load existing black hole cache.");
    let curr_cache_vec = get_file_as_byte_vec(BLACK_HOLE_CACHE_PATH);
    let mut curr_cache = None;
    if curr_cache_vec.is_ok() {
        curr_cache =
            Some(serde_json::from_slice::<BlackHoleCache>(&curr_cache_vec.unwrap()).unwrap());
    }

    let direction_cache_size = (1 << 5, 1 << 8);
    let distance_cache_size = (1 << 4, 1 << 6, 1 << 4);
    let distance_bounds = (3.0, 30.0);
    let black_hole_radius = 1.5;
    let disc_bounds = (1.5, 12.0);

    let direction_cache: DirectionCache;
    let distance_cache: DistanceCache;

    if curr_cache.is_none() {
        println!("Black hole cache not found.");
        println!("Generating direction cache.");
        direction_cache =
            DirectionCache::compute_new(direction_cache_size, distance_bounds, black_hole_radius);
        println!("Generating distance cache.");
        distance_cache = DistanceCache::compute_new(
            distance_cache_size,
            distance_bounds,
            black_hole_radius,
            disc_bounds,
        );
    } else {
        println!("Black hole cache found.");
        let curr_cache = curr_cache.unwrap();
        let curr_direction_cache = curr_cache.direction_cache;
        if curr_direction_cache.black_hole_radius != black_hole_radius
            || curr_direction_cache.cache_size != direction_cache_size
            || curr_direction_cache.distance_bounds != distance_bounds
        {
            println!("Generating direction cache.");
            direction_cache = DirectionCache::compute_new(
                direction_cache_size,
                distance_bounds,
                black_hole_radius,
            );
        } else {
            println!("Using existing direction cache.");
            direction_cache = curr_direction_cache;
        }

        let curr_distance_cache = curr_cache.distance_cache;
        if curr_distance_cache.black_hole_radius != black_hole_radius
            || curr_distance_cache.cache_size != distance_cache_size
            || curr_distance_cache.distance_bounds != distance_bounds
            || curr_distance_cache.disc_bounds != disc_bounds
            || true
        {
            println!("Generating distance cache.");
            distance_cache = DistanceCache::compute_new(
                distance_cache_size,
                distance_bounds,
                black_hole_radius,
                disc_bounds,
            );
        } else {
            println!("Using existing distance cache.");
            distance_cache = curr_distance_cache;
        }
    }

    let new_cache = BlackHoleCache::new(distance_cache, direction_cache);
    let data = serde_json::to_string(&new_cache).unwrap();
    println!("Writing black hole cache out.");
    fs::write(BLACK_HOLE_CACHE_PATH, data).expect("Unable to write file");
}

fn main() {
    generate_test_points();
    //regenerate_black_hole_cache();
}
