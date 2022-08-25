use test_utils::plot_with_title;
use wire_structs::sampler::{
    dimension_params::DimensionParams,
    ray_approximation_sampler::RayApproximationSampler,
    view_bound_sampler::{ViewBoundSampler, ViewType},
};

pub fn plot_approx_paths_sampled(
    approx_sampler: &RayApproximationSampler,
    dist: &DimensionParams,
    view: &DimensionParams,
    angle: &DimensionParams,
    view_bound_sampler: &ViewBoundSampler,
) {
    let dists = dist.generate_list();
    let views = view.generate_list();
    let angles = angle.generate_list();

    for (d, dist) in dists.iter().enumerate() {
        let mut paths = Vec::new();
        for (v, view) in views.iter().enumerate() {
            let mut path = Vec::new();
            match view_bound_sampler.get_view_01(*dist, *view).0 {
                ViewType::Far => {}
                _ => {
                    continue;
                }
            }
            let approx = approx_sampler.sample(*dist, *view);
            for (a, angle) in angles.iter().enumerate() {
                let angle = *angle;
                if angle >= approx.final_angle {
                    break;
                }
                let dist = approx.get_dist(angle);
                if dist > 41. {
                    break;
                }
                path.push((angle.sin() * dist, -angle.cos() * dist));
            }
            paths.push(path);
        }
        plot_with_title(
            &format!("Approximate Ray Paths at dist = {}", dist),
            &format!(
                "generate_artifacts/output/ray_approx_sampler/path/approx_path_error_{}.png",
                dist
            ),
            &paths,
            ((-30., 30.), (-30., 30.)),
        )
        .unwrap();
    }
}
