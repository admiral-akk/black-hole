use test_utils::plot_with_title;
use wire_structs::sampler::{
    close_ray_approximation::{measure_error, CloseRayApproximation},
    dimension_params::DimensionParams,
    path_sampler::PathSampler,
    view_bound_sampler::ViewBoundSampler,
};

pub fn plot_close_approximate_ray_analysis(
    path_sampler: &PathSampler,
    dist: &DimensionParams,
    view: &DimensionParams,
    angle: &DimensionParams,
    view_sampler: &ViewBoundSampler,
) {
    plot_close_approximate_ray_paths(path_sampler, angle);
}

fn plot_close_approximate_ray_paths(path_sampler: &PathSampler, angle: &DimensionParams) {
    let angles = angle.generate_list();
    for paths in &path_sampler.close {
        let dist = paths[0].dist;
        if dist != 30. {
            continue;
        }
        let mut d_paths = Vec::new();
        for path in paths {
            let close_ray = CloseRayApproximation::generate_optimal(&path.ray, path.dist, angle);
            println!("close approx: {:?}", close_ray);
            println!("error: {:?}", measure_error(&close_ray, &path.ray, angle));

            let mut path = Vec::new();

            for a in &angles {
                if *a > close_ray.final_angle {
                    break;
                }
                let d = close_ray.get_dist(*a);
                path.push((a.sin() * d, -a.cos() * d));
            }
            d_paths.push(path);
        }
        let dist = paths[0].dist;
        plot_with_title(
            &format!("Far sampler Paths at dist = {}", dist),
            &format!(
                "generate_artifacts/output/close_approx/path/path_sample_plot_{}.png",
                dist
            ),
            &d_paths,
            ((-30., 30.), (-30., 30.)),
        )
        .unwrap();
    }
}
