use std::f32::consts::TAU;

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
    let close_rays = generate_close_rays(path_sampler, angle);
    plot_property_by_dist(
        &close_rays,
        angle,
        &|ray, dist, view, a| {
            let d = ray.get_dist(a);
            (a.sin() * d, -a.cos() * d)
        },
        "path",
        "close_approx_paths",
        "Close Approximate Paths",
        ((-30., 30.), (-30., 30.)),
    );
    plot_property_by_dist(
        &close_rays,
        angle,
        &|ray, dist, view, a| {
            let d = ray.get_dist(a);
            (view, ray.final_angle)
        },
        "final_angle",
        "final_angle",
        "Final Angles",
        ((0., 1.), (0., TAU as f64)),
    );
    plot_property_by_dist(
        &close_rays,
        angle,
        &|ray, dist, view, a| {
            let d = ray.get_dist(a);
            (view, ray.spiral_start_dist)
        },
        "start_dist",
        "start_dist",
        "Start of the Spiral Dist",
        ((0., 1.), (0., 10.)),
    );
    plot_property_by_dist(
        &close_rays,
        angle,
        &|ray, dist, view, a| {
            let d = ray.get_dist(a);
            (view, ray.spiral_start_angle)
        },
        "start_angle",
        "start_angle",
        "Start of the Spiral Angle",
        ((0., 1.), (0., TAU as f64)),
    );
}

fn generate_close_rays(
    path_sampler: &PathSampler,
    angle: &DimensionParams,
) -> Vec<(f32, Vec<(f32, CloseRayApproximation)>)> {
    let angles = angle.generate_list();
    println!("Generating close rays");
    let mut dist_to_rays = Vec::new();
    for paths in &path_sampler.close {
        let mut rays = Vec::new();
        for path in paths {
            if path.dist > 5. {
                break;
            }
            let close_ray = CloseRayApproximation::generate_optimal(&path.ray, path.dist, angle);
            rays.push((path.view, close_ray));
        }
        if rays.len() == 0 {
            continue;
        }

        let dist = paths[0].dist;
        dist_to_rays.push((dist, rays));
    }
    dist_to_rays
}

fn plot_property_by_dist(
    rays: &Vec<(f32, Vec<(f32, CloseRayApproximation)>)>,
    angle: &DimensionParams,
    property: &dyn Fn(&CloseRayApproximation, f32, f32, f32) -> (f32, f32),
    file_name: &str,
    folder: &str,
    plot_title: &str,
    bounds: ((f64, f64), (f64, f64)),
) {
    println!("Generating {}", plot_title);
    let angles = angle.generate_list();
    for (dist, rays) in rays {
        let mut d_paths = Vec::new();
        for (view, ray) in rays {
            let mut path = Vec::new();

            for a in &angles {
                if *a > ray.final_angle {
                    break;
                }
                path.push(property(ray, *dist, *view, *a));
            }
            d_paths.push(path);
        }
        println!("Generating {}, dist: {}", plot_title, dist);
        plot_with_title(
            &format!("{} at dist = {}", plot_title, dist),
            &format!(
                "generate_artifacts/output/close_approx/{}/{}_{}.png",
                folder, file_name, dist
            ),
            &d_paths,
            bounds,
        )
        .unwrap();
    }
}

fn plot_close_final_angle(
    rays: &Vec<(f32, Vec<(f32, CloseRayApproximation)>)>,
    angle: &DimensionParams,
) {
    let angles = angle.generate_list();
    for (dist, rays) in rays {
        let mut d_paths = Vec::new();
        for (view, ray) in rays {
            let mut path = Vec::new();

            for a in &angles {
                if *a > ray.final_angle {
                    break;
                }
                let d = ray.get_dist(*a);
                if d < 1.5 {
                    break;
                }
                path.push((a.sin() * d, -a.cos() * d));
            }
            d_paths.push(path);
        }
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

fn plot_close_approximate_ray_paths(
    rays: &Vec<(f32, Vec<(f32, CloseRayApproximation)>)>,
    angle: &DimensionParams,
) {
    let angles = angle.generate_list();
    for (dist, rays) in rays {
        let mut d_paths = Vec::new();
        for (view, ray) in rays {
            let mut path = Vec::new();

            for a in &angles {
                if *a > ray.final_angle {
                    break;
                }
                let d = ray.get_dist(*a);
                if d < 1.5 {
                    break;
                }
                path.push((a.sin() * d, -a.cos() * d));
            }
            d_paths.push(path);
        }
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
