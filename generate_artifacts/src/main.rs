use std::fs;

use path_integration::cache::ray_cache::RayCache;

const RAY_CACHE_PATH: &str = "generate_artifacts/output/ray_cache.txt";
const ANGLE_CACHE_PATH: &str = "generate_artifacts/output/angle_cache.txt";

fn main() {
    let cache_dimensions = (128, 512);
    let black_hole_radius = 1.5;
    let distance_bounds = (5.0, 20.0);
    let ray_cache = RayCache::compute_new(cache_dimensions, black_hole_radius, distance_bounds);
    let data = serde_json::to_string(&ray_cache).unwrap();
    fs::write(RAY_CACHE_PATH, data).expect("Unable to write file");
}
