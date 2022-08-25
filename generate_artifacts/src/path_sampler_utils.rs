use std::f64::consts::TAU;

use test_utils::plot_with_title;
use wire_structs::sampler::{
    dimension_params::DimensionParams,
    path_sampler::PathSampler,
    ray_approximation::{measure_error, RayApproximation},
};

pub fn plot_sampled_paths(sampler: &PathSampler, dist: &DimensionParams, angle: &DimensionParams) {
    let angles = angle.generate_list();
    let dists = dist.generate_list();
    for (i, dist_rays) in sampler.far.iter().enumerate() {
        let mut paths = Vec::new();
        for ray in dist_rays {
            let mut path = Vec::new();
            for (i, dist) in ray.ray.angle_dist.iter().enumerate() {
                let dist = *dist;
                if dist == 0. || dist > 30. {
                    break;
                }
                let angle = angles[i];
                path.push((angle.sin() * dist, -angle.cos() * dist));
            }
            paths.push(path);
        }

        let dist = dists[i];
        plot_with_title(
            &format!("Sampler Paths at dist = {}", dist),
            &format!(
                "generate_artifacts/output/path_sample/path_sample_plot_{}.png",
                dist
            ),
            &paths,
            ((-30., 30.), (-30., 30.)),
        )
        .unwrap();
    }
}

pub fn plot_approx_paths(sampler: &PathSampler, dist: &DimensionParams, angle: &DimensionParams) {
    let angles = angle.generate_list();
    let dists = dist.generate_list();
    for (i, dist_rays) in sampler.far.iter().enumerate() {
        let dist = dists[i];
        let mut paths = Vec::new();
        for (ray_index, ray) in dist_rays.iter().enumerate() {
            let approx = RayApproximation::generate_optimal(&ray.ray, dist, angle);
            let mut path = Vec::new();
            for angle in &angles {
                let angle = *angle;
                if angle >= approx.final_angle {
                    break;
                }
                let dist = approx.get_dist(angle);
                if dist > 31. {
                    break;
                }
                path.push((angle.sin() * dist, -angle.cos() * dist));
            }
            paths.push(path);
        }

        let dist = dists[i];
        plot_with_title(
            &format!("Approximate Ray Paths at dist = {}", dist),
            &format!(
                "generate_artifacts/output/approx_sample/path/approx_path_error_{}.png",
                dist
            ),
            &paths,
            ((-30., 30.), (-30., 30.)),
        )
        .unwrap();
    }
}

pub fn plot_approx_errors_by_angle(
    sampler: &PathSampler,
    dist: &DimensionParams,
    view: &DimensionParams,
    angle: &DimensionParams,
) {
    let angles = angle.generate_list();
    let dists = dist.generate_list();
    let views = view.generate_list();
    for (i, dist_rays) in sampler.far.iter().enumerate() {
        let dist = dists[i];
        let mut paths = Vec::new();
        for (ray_index, ray) in dist_rays.iter().enumerate() {
            let mut path = Vec::new();
            let approx = RayApproximation::generate_optimal(&ray.ray, dist, angle);
            for (angle_index, angle) in angles.iter().enumerate() {
                if ray.ray.angle_dist[angle_index] == 0. {
                    break;
                }
                let error = (approx.get_dist(*angle) - ray.ray.angle_dist[angle_index]);
                path.push((*angle, error));
            }
            paths.push(path);
        }

        let dist = dists[i];
        plot_with_title(
            &format!("Approximate Ray Error by angle at dist = {}", dist),
            &format!(
                "generate_artifacts/output/approx_sample/error_by_angle/approx_path_error_{}.png",
                dist
            ),
            &paths,
            ((0., 1.), (-1., 1.)),
        )
        .unwrap();
    }
}

pub fn plot_approx_errors(
    sampler: &PathSampler,
    dist: &DimensionParams,
    view: &DimensionParams,
    angle: &DimensionParams,
) {
    let angles = angle.generate_list();
    let dists = dist.generate_list();
    let views = view.generate_list();
    for (i, dist_rays) in sampler.far.iter().enumerate() {
        let dist = dists[i];
        let mut paths = Vec::new();
        let mut path = Vec::new();
        for (ray_index, ray) in dist_rays.iter().enumerate() {
            let approx = RayApproximation::generate_optimal(&ray.ray, dist, angle);
            path.push((views[ray_index], measure_error(&approx, &ray.ray, &angle)));
        }
        paths.push(path);

        let dist = dists[i];
        plot_with_title(
            &format!("Approximate Ray Error at dist = {}", dist),
            &format!(
                "generate_artifacts/output/approx_sample/total_error/approx_path_plot_{}.png",
                dist
            ),
            &paths,
            ((0., 1.), (0., 1.)),
        )
        .unwrap();
    }
}

pub fn plot_error_by_interpolation(
    sampler: &PathSampler,
    dist: &DimensionParams,
    view: &DimensionParams,
    angle: &DimensionParams,
) {
    let angles = angle.generate_list();
    let dists = dist.generate_list();
    let views = view.generate_list();
    for (i, dist_rays) in sampler.far.iter().enumerate() {
        let dist = dists[i];
        let mut paths = Vec::new();
        let mut path = Vec::new();
        for (ray_index, ray) in dist_rays.iter().enumerate() {
            if ray_index == 0 || ray_index == dist_rays.len() - 1 {
                continue;
            }

            let prior_approx =
                RayApproximation::generate_optimal(&dist_rays[ray_index - 1].ray, dist, angle);
            let next_approx =
                RayApproximation::generate_optimal(&dist_rays[ray_index + 1].ray, dist, angle);
            let average = RayApproximation::generate_average(&[prior_approx, next_approx]);
            path.push((views[ray_index], measure_error(&average, &ray.ray, &angle)));
        }
        paths.push(path);

        let dist = dists[i];
        plot_with_title(
            &format!("Interpolated Ray Error at dist = {}", dist),
            &format!(
                "generate_artifacts/output/approx_sample/interpolation_error/error_at_dist_{}.png",
                dist
            ),
            &paths,
            ((0., 1.), (0., 1.)),
        )
        .unwrap();
    }
}

pub fn plot_error_by_interpolation_by_angle(
    sampler: &PathSampler,
    dist: &DimensionParams,
    view: &DimensionParams,
    angle: &DimensionParams,
) {
    let angles = angle.generate_list();
    let dists = dist.generate_list();
    let views = view.generate_list();

    for d_index in 1..(dists.len() - 1) {
        let mut paths = Vec::new();
        let prev_d_index = d_index - 1;
        let next_d_index = d_index + 1;
        for r_index in 1..(views.len() - 1) {
            let mut path = Vec::new();
            let prev_r_index = r_index - 1;
            let next_r_index = r_index + 1;
            let true_ray = &sampler.far[d_index][r_index];

            let mut approximations = Vec::new();
            for r in [prev_r_index, next_r_index] {
                for d in [prev_d_index, next_d_index] {
                    approximations.push(RayApproximation::generate_optimal(
                        &sampler.far[d][r].ray,
                        dists[d],
                        angle,
                    ))
                }
            }
            let average = RayApproximation::generate_average(&approximations);
            for (angle_index, angle) in angles.iter().enumerate() {
                if true_ray.ray.angle_dist[angle_index] == 0. {
                    break;
                }
                let error = (average.get_dist(*angle) - true_ray.ray.angle_dist[angle_index]);
                path.push((*angle, error));
            }
            paths.push(path);
        }

        let dist = dists[d_index];
        plot_with_title(
            &format!("Interpolated Ray Error by angle dist = {}", dist),
            &format!(
                "generate_artifacts/output/approx_sample/interpolation_error_by_angle/error_at_dist_{}.png",
                dist
            ),
            &paths,
            ((0., TAU), (-1., 1.)),
        )
        .unwrap();
    }
}
