use std::fs;

use path_integration::cache::{
    angle_cache::AngleCache, fixed_distance_distance_cache::FixedDistanceDistanceCache,
    ray_cache::RayCache,
};

const FIXED_DISTANCE_DISTANCE_CACHE: &str =
    "generate_artifacts/output/fixed_distance_distance_cache.txt";
const RAY_CACHE_PATH: &str = "generate_artifacts/output/ray_cache.txt";
const ANGLE_CACHE_PATH: &str = "generate_artifacts/output/angle_cache.txt";

fn generate_ray_cache() {
    let cache_dimensions = (128, 512);
    let black_hole_radius = 1.5;
    let distance_bounds = (7.0, 20.0);
    let ray_cache = RayCache::compute_new(cache_dimensions, black_hole_radius, distance_bounds);
    let data = serde_json::to_string(&ray_cache).unwrap();
    fs::write(RAY_CACHE_PATH, data).expect("Unable to write file");
}
fn generate_angle_cache() {
    let cache_dimensions = (1, 1024, 128);
    let black_hole_radius = 1.5;
    let distance_bounds = (16.0, 18.0);
    let disc_radius = (3.0, 6.0);
    let angle_cache = AngleCache::compute_new(
        cache_dimensions,
        black_hole_radius,
        distance_bounds,
        disc_radius,
    );
    let data = serde_json::to_string(&angle_cache).unwrap();
    fs::write(ANGLE_CACHE_PATH, data).expect("Unable to write file");
}
fn generate_fixed_distance_distance_cache() {
    let cache_dimensions = (512, 64);
    let camera_distance = 17.0;
    let black_hole_radius = 1.5;
    let disc_radius = (3.0, 6.0);
    let angle_cache = FixedDistanceDistanceCache::compute_new(
        cache_dimensions,
        camera_distance,
        black_hole_radius,
        disc_radius,
    );
    let data = serde_json::to_string(&angle_cache).unwrap();
    fs::write(FIXED_DISTANCE_DISTANCE_CACHE, data).expect("Unable to write file");
}

fn main() {
    // generate_ray_cache();
    // generate_angle_cache();
    generate_fixed_distance_distance_cache();
}
