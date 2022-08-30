use std::f64::consts::TAU;

use std::fs::{self};

use approximation_utils::analyze_approximations;
use artifact_utils::get_or_generate_file;
use combined_approximation_utils::plot_combined_approximate_ray_analysis;

use path_utils::analyze_paths;
use serde::{Deserialize, Serialize};
use view_bounds_utils::analyze_view_bounds;
use wire_structs::sampler::approximation_function::ApproximationFunction;
use wire_structs::sampler::dimension_params::DimensionParams;

use wire_structs::sampler::render_params::RenderParams;
use wire_structs::sampler::simple_path_generator;
use wire_structs::sampler::view_angle_parameter_cache::ViewAngleParameterCache;
use wire_structs::sampler::view_bound::ViewBound;
mod factory;
mod final_direction_cache;
mod path_distance_cache;
mod path_integration2;

#[derive(Serialize, Deserialize, Debug)]
pub struct DirectionTestPoint {
    pub z: f64,
    pub dist: f64,
    pub final_angle: Option<f64>,
}

const ALL_VIEW_SAMPLE_PATH: &str = "generate_artifacts/output/artifact/all_view.txt";
const APPROX_FUNCTION_PATH: &str = "generate_artifacts/output/artifact/approx_function.txt";
const VIEW_BOUNDS_PATH: &str = "generate_artifacts/output/artifact/view_bounds.txt";

const ANGLE_CACHE_PATH: &str = "generate_artifacts/output/artifact/angle_cache.txt";

mod approximation_utils;
mod artifact_utils;
mod combined_approximation_utils;
mod path_utils;
mod view_bounds_utils;
fn main() {
    let dist = DimensionParams {
        size: 128,
        bounds: [5., 30.],
    };
    let view = DimensionParams {
        size: 2048,
        bounds: [0., 0.5_f32.sqrt()],
    };
    let angle = DimensionParams {
        size: 512,
        bounds: [0., TAU as f32],
    };
    let angles = angle.generate_list();

    let render_params = RenderParams {
        black_hole_radius: 1.5,
        fov_degrees: 60.0,
    };

    let all_paths_sample;
    {
        all_paths_sample = get_or_generate_file(ALL_VIEW_SAMPLE_PATH, &|| {
            simple_path_generator::generate_paths(&dist, &view, &angle, &render_params)
        });
    }

    let all_approx;
    {
        all_approx = get_or_generate_file(APPROX_FUNCTION_PATH, &|| {
            all_paths_sample
                .iter()
                .map(|p| ApproximationFunction::generate(p, &angles, p.view))
                .collect::<Vec<ApproximationFunction>>()
        });
    };

    let view_bounds;
    {
        view_bounds = get_or_generate_file(VIEW_BOUNDS_PATH, &|| ViewBound::generate(&all_approx));
    }
    let angle_cache;
    {
        angle_cache = get_or_generate_file(ANGLE_CACHE_PATH, &|| {
            ViewAngleParameterCache::new(dist.size as u32, &all_approx)
        });
    }

    // analyze_view_bounds(&view_bounds);
    analyze_paths(&all_paths_sample, &angle);
    //analyze_approximations(&all_paths_sample, &dist, &angle);
}
