use std::{
    fs::{self, File},
    io::Read,
};

use test_utils::plot_with_title;
use wire_structs::sampler::{
    dimension_params::DimensionParams,
    render_params::RenderParams,
    view_bound_sampler::{ViewBoundSampler, ViewType},
};

use crate::artifact_utils::get_or_generate_file;

pub fn plot_view_sampler_analysis(
    sampler: &ViewBoundSampler,
    dist: &DimensionParams,
    view: &DimensionParams,
) {
    plot_view_sampler_bound(sampler, dist, view);
}

fn plot_view_sampler_bound(
    view_sampler: &ViewBoundSampler,
    dist_dim: &DimensionParams,
    view_dim: &DimensionParams,
) {
    let mut close_lines = Vec::new();
    let mut far_lines = Vec::new();
    for d in dist_dim.generate_list() {
        let mut close_line = Vec::new();
        let mut far_line = Vec::new();
        for v in view_dim.generate_list() {
            let (view_type, v_01) = view_sampler.get_view_01(d, v);
            match view_type {
                ViewType::Close => {
                    close_line.push((v, v_01));
                }
                ViewType::Far => {
                    far_line.push((v, v_01));
                }
            };
        }
        close_lines.push(close_line);
        far_lines.push(far_line);
    }

    plot_with_title(
        &format!("Close bounds remapping"),
        "generate_artifacts/output/view_sampler/close_bounds_sampler.png",
        &close_lines,
        ((0., 1.), (0., 1.)),
    )
    .unwrap();
    plot_with_title(
        &format!("Far bounds remapping"),
        "generate_artifacts/output/view_sampler/far_bounds_sampler.png",
        &far_lines,
        ((0., 1.), (0., 1.)),
    )
    .unwrap();

    let mut line = Vec::new();
    let bound = view_sampler.show_bound();
    bound.iter().enumerate().for_each(|(i, b)| {
        let v_01 = (i as f32) / (bound.len() - 1) as f32;
        line.push((v_01, *b));
        println!("{:?}", line.last().unwrap());
    });
    plot_with_title(
        &format!("Actual bounds"),
        "generate_artifacts/output/view_sampler/actual_bounds_sampler.png",
        &[line].to_vec(),
        ((0., 1.), (0., 1.)),
    )
    .unwrap();
}
