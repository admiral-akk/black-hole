use std::f64::consts::TAU;

use std::fs::{self};

use generate_artifacts::analysis::angle_test_point::AngleTestPoint;
use generate_artifacts::analysis::cache_stats::plot_cache_statistics;
use generate_artifacts::analysis::generate_test_set::regenerate_angle_distance_test_points;
use generate_artifacts::analysis::z_error::plot_angle_error_by_z;
use generate_artifacts::black_hole_cache::BlackHoleCache;
use generate_artifacts::final_direction_cache::direction_cache::DirectionCache;
use generate_artifacts::path_distance_cache::distance_cache::DistanceCache;

use generate_artifacts::texture::texture_2d::{
    generate_final_angle_texture, sample_final_angle_texture, IndexMapping, Texture2D,
};

use serde::{Deserialize, Serialize};
use test_utils::plot_trajectories;
use wire_structs::angle_distance_cache::{AngleDistanceCache, AngleDistanceCacheParams};
use wire_structs::dimension_params::DimensionParams;

mod factory;
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

#[derive(Serialize, Deserialize, Debug)]
pub struct DirectionTestPoint {
    pub z: f64,
    pub dist: f64,
    pub final_angle: Option<f64>,
}

const DIRECTION_CACHE_SIZE: (usize, usize) = (1 << 5, 1 << 8);
const DISTANCE_CACHE_SIZE: (usize, usize, usize) = (1 << 4, 1 << 6, 1 << 4);
const DISTANCE_BOUNDS: (f64, f64) = (3.0, 30.0);
const DISTANCE_BOUNDS_F32: (f32, f32) = (DISTANCE_BOUNDS.0 as f32, DISTANCE_BOUNDS.1 as f32);
const BLACK_HOLE_RADIUS: f64 = 1.5;
const DISC_BOUNDS: (f64, f64) = (1.5, 12.0);
const DIST_TEST_POINTS: usize = 50;
const ANGLE_TEST_POINTS: usize = 45;
const Z_TEST_POINTS: usize = 2000;
use crate::path_integration2::path::cast_ray_steps_response;

const ANGLE_CACHE_PATH: &str = "generate_artifacts/output/angle_cache.txt";
const ANGLE_PLOT_Z_PATH: &str = "generate_artifacts/output/angle_cache_z_bound.png";
const ANGLE_ERROR_PLOT_PATH: &str = "generate_artifacts/output/angle_error.png";
const Z_POW: f32 = 32.;
const Z_LINEAR: f32 = 10.;
const DIST_POW: f32 = 0.5;

fn plot_z_bounds(tex: &Texture2D) {
    let distance_mapping = IndexMapping {
        i_01_to_dist_01: |i_01| i_01,
        dist_01_to_i_01: |dist_01| dist_01,
    };
    let mut low = Vec::new();
    let mut high = Vec::new();
    for d_index in 0..100 {
        let d_01 = distance_mapping.val_to_i_01(d_index as f32, (0., 100.));
        let z_bounds = tex.get_z_bounds(d_01);
        low.push((d_01, z_bounds.0));
        high.push((d_01, z_bounds.1));
    }

    plot_trajectories(
        ANGLE_PLOT_Z_PATH,
        &[low, high].to_vec(),
        ((0., 1.), (-1., 1.)),
    )
    .unwrap();
}

fn plot_error_by_z(tex: &Texture2D, distance_mapping: &IndexMapping, z_mapping: &IndexMapping) {
    let test_points = get_file_as_byte_vec(DIRECTION_TEST_PATH);
    let mut test_points: Vec<DirectionTestPoint> =
        serde_json::from_slice::<Vec<DirectionTestPoint>>(&test_points.unwrap()).unwrap();

    test_points.sort_by(|p_1, p_2| p_1.dist.partial_cmp(&p_2.dist).unwrap());
    let mut lines = Vec::new();
    let mut curr_dist = test_points[0].dist as f32;
    let mut line = Vec::new();
    for point in test_points {
        let dist = point.dist as f32;
        if dist != curr_dist {
            curr_dist = dist;
            lines.push(line);
            line = Vec::new();
        }
        let d_01 = distance_mapping.val_to_i_01(dist, DISTANCE_BOUNDS_F32);
        let z_bounds = tex.get_z_bounds(d_01);
        let z = point.z as f32;
        if z > z_bounds.1 || z < z_bounds.0 {
            continue;
        }
        if point.final_angle.is_none() {
            println!(
                "In bounds, but no final angle.\nPoint: {:?}\napprox z_bounds: {:?}",
                point,
                tex.get_z_bounds(d_01),
            );
            continue;
        }
        let v = sample_final_angle_texture(
            &tex,
            &distance_mapping,
            &z_mapping,
            dist,
            z,
            DISTANCE_BOUNDS_F32,
        );
        let diff = (v - point.final_angle.unwrap() as f32).abs();
        line.push((z.log2(), diff.log2()));
    }
    plot_trajectories(ANGLE_ERROR_PLOT_PATH, &lines, ((-10., 0.), (-10., 0.))).unwrap();
}

fn plot_angle_texture_stats(distance_mapping: &IndexMapping, z_mapping: &IndexMapping) {
    let tex_u8 = get_file_as_byte_vec(ANGLE_CACHE_PATH);
    let tex = serde_json::from_slice::<Texture2D>(&tex_u8.unwrap()).unwrap();
    plot_z_bounds(&tex);
    plot_error_by_z(&tex, distance_mapping, z_mapping);
}

fn generate_angle_texture(distance_mapping: &IndexMapping, z_mapping: &IndexMapping) {
    let tex_u8 = get_file_as_byte_vec(ANGLE_CACHE_PATH);
    let tex: Texture2D;
    if tex_u8.is_ok() {
        tex = serde_json::from_slice::<Texture2D>(&tex_u8.unwrap()).unwrap();
    } else {
        tex = generate_final_angle_texture(
            (32, 64),
            DISTANCE_BOUNDS_F32,
            BLACK_HOLE_RADIUS as f32,
            &distance_mapping,
            &z_mapping,
        );
        fs::write(ANGLE_CACHE_PATH, serde_json::to_string(&tex).unwrap())
            .expect("Unable to write file");
    }
}

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
            let final_angle = res.get_final_angle();
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
                let dist_at_angle = angle_dist.get_dist(target_angle as f64);

                let test_point = AngleTestPoint {
                    view_port_coord: z,
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
} // lib.rs

const ANGLE_DISTANCE_CACHE_PATH: &str = "generate_artifacts/output/angle_distance_cache.txt";

fn regenerate_angle_distance_cache(dimensions: [usize; 3]) -> AngleDistanceCache {
    let dist_bounds = [5.0, 30.0];
    let view_bounds = [0., 0.5_f32.sqrt()];
    let angle_bounds = [0.01 * TAU as f32 / 360.0, TAU as f32];
    let black_hole_radius = 1.5;
    let fov_degrees = 60.;

    let params = AngleDistanceCacheParams {
        dist: DimensionParams {
            size: dimensions[0],
            bounds: dist_bounds,
        },
        view_dist: DimensionParams {
            size: dimensions[1],
            bounds: view_bounds,
        },
        angle: DimensionParams {
            size: dimensions[2],
            bounds: angle_bounds,
        },
        black_hole_radius,
        fov_degrees,
    };
    let path = format!(
        "generate_artifacts/output/angle_distance_cache_{}.txt",
        params.cache_name()
    );
    let cache = fs::read(&path);
    if cache.is_ok() {
        println!("Found existing cache!");
        let cache = cache.unwrap();
        //let de: AngleDistanceCache = ciborium::de::from_reader(&*cache).unwrap();
        let de: AngleDistanceCache = serde_json::from_slice(&cache).unwrap();
        return de;
    }

    let cache = AngleDistanceCache::generate_angle_distance_cache_gpu(&params);
    //let cache = AngleDistanceCache::generate_angle_distance_cache(params);

    //ciborium::ser::into_writer(&cache, &mut buffer).unwrap();
    let buffer = serde_json::to_string(&cache).unwrap();
    //fs::write(&path, buffer).expect("Unable to write file");
    return cache;
}

fn test_angle_distance_cache(
    cache: &AngleDistanceCache,
    test_points: &Vec<AngleTestPoint>,
) -> Vec<(AngleTestPoint, Option<f32>)> {
    println!("Generating approximations for test points.");
    test_points
        .iter()
        .map(|test_point| {
            (
                *test_point,
                cache.get_dist(
                    test_point.dist,
                    test_point.view_port_coord,
                    test_point.target_angle,
                ),
            )
        })
        .collect()
}
fn main() {
    let dimensions = [32, 1024, 128];
    let cache = regenerate_angle_distance_cache(dimensions);
    plot_cache_statistics(&cache);
    let data = regenerate_angle_distance_test_points(&cache.params);
    let results = test_angle_distance_cache(&cache, &data);
    plot_angle_error_by_z(&cache, &results);
}
