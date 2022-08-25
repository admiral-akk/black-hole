use std::f64::consts::TAU;

use test_utils::plot_with_title;
use wire_structs::sampler::{
    dimension_params::DimensionParams,
    path_sampler::PathSampler,
    ray_approximation::measure_error,
    ray_approximation_sampler::RayApproximationSampler,
    view_bound_sampler::{ViewBoundSampler, ViewType},
};

pub fn plot_approx_paths_sampled(
    approx_sampler: &RayApproximationSampler,
    path_sampler: &PathSampler,
    angle: &DimensionParams,
    view_bound_sampler: &ViewBoundSampler,
) {
    let angles = angle.generate_list();

    for (d, dist) in path_sampler.far.iter().enumerate() {
        let mut paths = Vec::new();
        for (v, ray) in path_sampler.far[d].iter().enumerate() {
            let mut path = Vec::new();
            match view_bound_sampler.get_view_01(ray.dist, ray.view).0 {
                ViewType::Far => {}
                _ => {
                    continue;
                }
            }
            let approx = approx_sampler.sample(ray.dist, ray.view);
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
        let dist = dist[0].dist;
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

pub fn plot_approx_paths_total_error(
    approx_sampler: &RayApproximationSampler,
    path_sampler: &PathSampler,
    angle: &DimensionParams,
    view_bound_sampler: &ViewBoundSampler,
) {
    for (d, dist) in path_sampler.far.iter().enumerate() {
        let mut paths = Vec::new();
        let mut path = Vec::new();
        for (v, ray) in dist.iter().enumerate() {
            match view_bound_sampler.get_view_01(ray.dist, ray.view).0 {
                ViewType::Far => {}
                _ => {
                    continue;
                }
            }
            let approx = approx_sampler.sample(ray.dist, ray.view);
            path.push((ray.view, measure_error(&approx, &ray.ray, &angle)));
        }
        paths.push(path);
        let dist = dist[0].dist;
        plot_with_title(
            &format!("Approximate Ray Paths at dist = {}", dist),
            &format!(
                "generate_artifacts/output/ray_approx_sampler/total_error/approx_path_error_{}.png",
                dist
            ),
            &paths,
            ((0., 1.), (0., 10.)),
        )
        .unwrap();
    }
}

pub fn plot_approx_paths_error_by_angle(
    approx_sampler: &RayApproximationSampler,
    path_sampler: &PathSampler,
    angle: &DimensionParams,
    view_bound_sampler: &ViewBoundSampler,
) {
    let angles = angle.generate_list();
    for (d, dist) in path_sampler.far.iter().enumerate() {
        let mut paths = Vec::new();
        for (v, ray) in dist.iter().enumerate() {
            let mut path = Vec::new();
            match view_bound_sampler.get_view_01(ray.dist, ray.view).0 {
                ViewType::Far => {}
                _ => {
                    continue;
                }
            }
            let approx = approx_sampler.sample(ray.dist, ray.view);
            for (angle_index, angle) in angles.iter().enumerate() {
                if approx.final_angle <= *angle {
                    break;
                }
                if ray.ray.angle_dist[angle_index] == 0. {
                    break;
                }
                path.push((
                    *angle,
                    (approx.get_dist(*angle) - ray.ray.angle_dist[angle_index])
                        / ray.ray.angle_dist[angle_index],
                ));
            }
            paths.push(path);
        }
        let dist = dist[0].dist;
        plot_with_title(
            &format!("Approximate Ray Paths error, normalized by actual distance at dist = {}", dist),
            &format!(
                "generate_artifacts/output/ray_approx_sampler/error_by_angle/approx_path_error_{}.png",
                dist
            ),
            &paths,
            ((0., TAU), (-1., 1.)),
        )
        .unwrap();
    }
}
