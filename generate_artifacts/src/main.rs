use std::f64::consts::TAU;

use std::fs::{self};

use artifact_utils::get_or_generate_file;
use combined_approximation_utils::plot_combined_approximate_ray_analysis;

use serde::{Deserialize, Serialize};
use wire_structs::sampler::dimension_params::DimensionParams;

use wire_structs::sampler::render_params::RenderParams;
use wire_structs::sampler::simple_path_generator;
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

const GENERATED_COMBINED_PATH: &str = "generate_artifacts/output/artifact/combined.txt";
mod artifact_utils;
mod combined_approximation_utils;
fn main() {
    let dist = DimensionParams {
        size: 32,
        bounds: [5., 30.],
    };
    let view = DimensionParams {
        size: 256,
        bounds: [0., 0.5_f32.sqrt()],
    };
    let angle = DimensionParams {
        size: 512,
        bounds: [0., TAU as f32],
    };

    let render_params = RenderParams {
        black_hole_radius: 1.5,
        fov_degrees: 60.0,
    };

    let test_angle = DimensionParams {
        size: 512,
        bounds: [0., TAU as f32],
    };

    let all_paths_sample;
    {
        all_paths_sample = get_or_generate_file(ALL_VIEW_SAMPLE_PATH, &move || {
            simple_path_generator::generate_paths(&dist, &view, &angle, &render_params)
        });
    }

    plot_combined_approximate_ray_analysis(&all_paths_sample, &dist, &view, &angle);
    let x = get_or_generate_file(GENERATED_COMBINED_PATH, &move || {
        plot_combined_approximate_ray_analysis(&all_paths_sample, &dist, &view, &angle)
    });

    // plot_view_sampler_analysis(&view_sampler, &dist, &view);
    // plot_path_sampler_analysis(&path_sampler, &dist, &view, &angle);

    // plot_approximate_far_path_analysis(
    //     &approx_sampler,
    //     &path_sampler_test,
    //     &test_angle,
    //     &view_sampler,
    // );
    // plot_close_approximate_ray_analysis(
    //     &path_sampler_test,
    //     &dist,
    //     &view,
    //     &test_angle,
    //     &view_sampler,
    // );
}
