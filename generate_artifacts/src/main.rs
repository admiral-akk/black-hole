use std::f64::consts::TAU;

use std::fs::{self};

use approximate_path_sampler_utils::plot_approximate_far_path_analysis;
use artifact_utils::get_or_generate_file;
use close_approximate_ray_utils::plot_close_approximate_ray_analysis;
use generate_artifacts::black_hole_cache::BlackHoleCache;
use generate_artifacts::final_direction_cache::direction_cache::DirectionCache;
use generate_artifacts::path_distance_cache::distance_cache::DistanceCache;

use path_sampler_utils::plot_path_sampler_analysis;
use serde::{Deserialize, Serialize};
use view_sampler_utils::plot_view_sampler_analysis;
use wire_structs::sampler::dimension_params::DimensionParams;
use wire_structs::sampler::path_sampler::PathSampler;
use wire_structs::sampler::ray_approximation_sampler::RayApproximationSampler;
use wire_structs::sampler::render_params::RenderParams;
use wire_structs::sampler::view_bound_sampler::ViewBoundSampler;

mod factory;
mod final_direction_cache;
mod path_distance_cache;
mod path_integration2;
mod view_sampler_utils;
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

const ANGLE_CACHE_PATH: &str = "generate_artifacts/output/angle_cache.txt";
const ANGLE_PLOT_Z_PATH: &str = "generate_artifacts/output/angle_cache_z_bound.png";
const ANGLE_ERROR_PLOT_PATH: &str = "generate_artifacts/output/angle_error.png";
const Z_POW: f32 = 32.;
const Z_LINEAR: f32 = 10.;
const DIST_POW: f32 = 0.5;

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

const VIEW_SAMPLER_PATH: &str = "generate_artifacts/output/artifact/view_sampler.txt";
const PATH_SAMPLER_PATH: &str = "generate_artifacts/output/artifact/path_sampler.txt";
const PATH_TEST_SAMPLER_PATH: &str = "generate_artifacts/output/artifact/path_test_sampler.txt";
const APPROX_SAMPLER_PATH: &str = "generate_artifacts/output/artifact/approx_sampler.txt";

mod angle_distance_sampler_utils;
mod approximate_path_sampler_utils;
mod artifact_utils;
mod close_approximate_ray_utils;
mod path_sampler_utils;
fn main() {
    let dist = DimensionParams {
        size: 16,
        bounds: [5., 30.],
    };
    let view = DimensionParams {
        size: 128,
        bounds: [0., 0.5_f32.sqrt()],
    };
    let angle = DimensionParams {
        size: 128,
        bounds: [0., TAU as f32],
    };
    let render_params = RenderParams {
        black_hole_radius: 1.5,
        fov_degrees: 60.,
    };

    let view_sampler = get_or_generate_file(VIEW_SAMPLER_PATH, &move || {
        ViewBoundSampler::generate(dist, view, angle, &render_params, 0.5)
    });
    let path_sampler;
    {
        let view_sampler = view_sampler.clone();
        path_sampler = get_or_generate_file(PATH_SAMPLER_PATH, &move || {
            PathSampler::generate(dist, angle, view, &view_sampler, &render_params)
        });
    }

    let test_angle = DimensionParams {
        size: 512,
        bounds: [0., TAU as f32],
    };
    let path_sampler_test;
    {
        let dist = DimensionParams {
            size: 4 * dist.size,
            bounds: dist.bounds,
        };
        let view = DimensionParams {
            size: 4 * view.size,
            bounds: view.bounds,
        };
        let view_sampler = view_sampler.clone();
        path_sampler_test = get_or_generate_file(PATH_TEST_SAMPLER_PATH, &move || {
            PathSampler::generate(dist, test_angle, view, &view_sampler, &render_params)
        });
    }
    let approx_sampler;
    {
        let path_sampler = path_sampler.clone();
        let view_sampler = view_sampler.clone();
        approx_sampler = get_or_generate_file(APPROX_SAMPLER_PATH, &move || {
            RayApproximationSampler::generate(&path_sampler, dist, angle, view, &view_sampler)
        });
    }
    // plot_view_sampler_analysis(&view_sampler, &dist, &view);
    // plot_path_sampler_analysis(&path_sampler, &dist, &view, &angle);

    // plot_approximate_far_path_analysis(
    //     &approx_sampler,
    //     &path_sampler_test,
    //     &test_angle,
    //     &view_sampler,
    // );
    plot_close_approximate_ray_analysis(&path_sampler, &dist, &view, &angle, &view_sampler);
}
