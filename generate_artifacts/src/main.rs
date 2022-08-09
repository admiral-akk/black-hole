use std::fs::{self};

use generate_artifacts::final_direction_cache::direction_cache::DirectionCache;
use path_distance_cache::distance_cache::DistanceCache;
use path_distance_cache::fixed_distance_distance_cache::FixedDistanceDistanceCache;
use path_integration::cache::{angle_cache::AngleCache, ray_cache::RayCache};
use serde::Serialize;

mod final_direction_cache;
mod path_distance_cache;
mod path_integration2;
const FIXED_DISTANCE_DISTANCE_CACHE: &str =
    "generate_artifacts/output/fixed_distance_distance_cache.txt";
const DISTANCE_DISTANCE_CACHE: &str = "generate_artifacts/output/distance_cache.txt";
const FIXED_DISTANCE_DISTANCE_CACHE_FLEX_BUFFER: &str =
    "generate_artifacts/output/fixed_distance_distance_cache.flex";
const RAY_CACHE_PATH: &str = "generate_artifacts/output/ray_cache.txt";
const ANGLE_CACHE_PATH: &str = "generate_artifacts/output/angle_cache.txt";
const DIRECTION_CACHE_PATH: &str = "generate_artifacts/output/direction_cache.txt";

fn generate_ray_cache() {
    let cache_dimensions = (128, 512);
    let black_hole_radius = 1.5;
    let distance_bounds = (7.0, 20.0);
    let ray_cache = RayCache::compute_new(cache_dimensions, black_hole_radius, distance_bounds);
    let data = serde_json::to_string(&ray_cache).unwrap();
    fs::write(RAY_CACHE_PATH, data).expect("Unable to write file");
}

fn generate_distance_cache() {
    let cache_size = (16, 256, 64);
    let black_hole_radius = 1.5;
    let distance = (5., 20.0);
    let max_disc_radius = (1.5, 12.0);
    let angle_cache =
        DistanceCache::compute_new(cache_size, distance, black_hole_radius, max_disc_radius);
    let data = serde_json::to_string(&angle_cache).unwrap();
    fs::write(
        DISTANCE_DISTANCE_CACHE.replace(
            ".txt",
            format!("{}_{}_{}.txt", cache_size.0, cache_size.1, cache_size.2).as_str(),
        ),
        data,
    )
    .expect("Unable to write file");
}
fn generate_direction_cache() {
    let cache_dimensions = (1 << 6, 1 << 10);
    let black_hole_radius = 1.5;
    let distance_bounds = (5.0, 20.0);
    let angle_cache =
        DirectionCache::compute_new(cache_dimensions, distance_bounds, black_hole_radius);
    let data = serde_json::to_string(&angle_cache).unwrap();
    fs::write(DIRECTION_CACHE_PATH, data).expect("Unable to write file");
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
use std::fs::File;
use std::io::Read;

fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}
fn generate_fixed_distance_distance_cache() {
    let cache_dimensions = (64, 64);
    let camera_distance = 17.0;
    let black_hole_radius = 1.5;
    let disc_radius = (1.5, 12.0);
    let angle_cache = FixedDistanceDistanceCache::compute_new(
        cache_dimensions,
        camera_distance,
        black_hole_radius,
        disc_radius,
    );
    let data = serde_json::to_string(&angle_cache).unwrap();
    fs::write(
        FIXED_DISTANCE_DISTANCE_CACHE.replace(
            ".txt",
            format!("{}_{}.txt", cache_dimensions.0, cache_dimensions.1).as_str(),
        ),
        data,
    )
    .expect("Unable to write file");
}

fn write_file_as_byte_vec(filename: &String, bytes: Vec<u8>) {
    fs::write(filename, &bytes).unwrap();
}

fn reserialize_fixed_distance_distance_cache() {
    let angle_cache_u8 = get_file_as_byte_vec(&FIXED_DISTANCE_DISTANCE_CACHE.to_string());
    let fixed_distance_distance_cache =
        serde_json::from_slice::<FixedDistanceDistanceCache>(&angle_cache_u8).unwrap();

    let mut s = flexbuffers::FlexbufferSerializer::new();
    fixed_distance_distance_cache.serialize(&mut s).unwrap();
    write_file_as_byte_vec(
        &FIXED_DISTANCE_DISTANCE_CACHE_FLEX_BUFFER.to_string(),
        s.take_buffer(),
    );
}

fn main() {
    // generate_ray_cache();
    // generate_angle_cache();
    generate_direction_cache();
    // reserialize_fixed_distance_distance_cache();
}
