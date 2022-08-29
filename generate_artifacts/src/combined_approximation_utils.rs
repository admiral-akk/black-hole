use std::f32::consts::TAU;

use test_utils::plot_with_title;
use wire_structs::sampler::{
    combined_ray_approximation::{measure_error, CombinedRayApproximation},
    dimension_params::DimensionParams,
    path_sampler::{PathSampler, SimulatedPath},
    render_params::RenderParams,
    view_bound_sampler::ViewBoundSampler,
};

use crate::path_distance_cache::distance_cache;

pub fn plot_combined_approximate_ray_analysis(
    paths: &Vec<SimulatedPath>,
    dist: &DimensionParams,
    view: &DimensionParams,
    angle: &DimensionParams,
    view_sampler: &RenderParams,
) -> Vec<(f32, Vec<(f32, CombinedRayApproximation)>)> {
    let close_rays = generate_combined_rays(paths, angle);

    plot_actual_paths(&paths, dist, angle, view_sampler);
    plot_property_by_dist(
        &close_rays,
        angle,
        &|ray, dist, view, a| {
            let d = ray.get_dist(a);
            (a.sin() * d, -a.cos() * d)
        },
        "path",
        "combined_approx_paths",
        "Combined Approximate Paths",
        ((-30., 30.), (-30., 30.)),
    );
    plot_ray_property_by_dist(
        &close_rays,
        angle,
        &|ray, dist, view| (view, ray.final_angle),
        "final_angle",
        "final_angle",
        "Final Angles",
        ((0., 1.), (0., TAU as f64)),
    );
    plot_ray_property_by_dist(
        &close_rays,
        angle,
        &|ray, dist, view| (view, ray.curve_start_angle),
        "start_dist",
        "start_dist",
        "Start of the Spiral Dist",
        ((0., 1.), (0., 10.)),
    );
    plot_ray_property_by_dist(
        &close_rays,
        angle,
        &|ray, dist, view| (view, ray.curve_start_angle),
        "start_angle",
        "start_angle",
        "Start of the Spiral Angle",
        ((0., 1.), (0., TAU as f64)),
    );
    plot_ray_property_by_dist(
        &close_rays,
        angle,
        &|ray, dist, view| (view, ray.close_weight),
        "weight",
        "weight",
        "Close weight",
        ((0., 1.), (0., 1. as f64)),
    );
    close_rays
}
fn plot_actual_paths(
    paths: &Vec<SimulatedPath>,
    dist: &DimensionParams,
    angle: &DimensionParams,
    view_sampler: &RenderParams,
) {
    let d = dist.generate_list();
    let angles = angle.generate_list();
    let view_len = paths.len() / d.len();
    for (i, dist) in d.iter().enumerate() {
        let mut plot_paths = Vec::new();
        for ray in &paths[i * view_len..(i + 1) * view_len] {
            let mut path = Vec::new();
            for (i, a) in angles.iter().enumerate() {
                let d = ray.ray.angle_dist[i];
                if d == 0. {
                    break;
                }
                path.push((a.sin() * d, -a.cos() * d))
            }
            plot_paths.push(path);
        }
        plot_with_title(
            &format!("{} at dist = {}", "true path", *dist),
            &format!(
                "generate_artifacts/output/combined_approx/{}/{}_{}.png",
                "true_path", "true_path", *dist
            ),
            &plot_paths,
            ((-30., 30.), (-30., 30.)),
        )
        .unwrap();
    }
}

fn generate_combined_rays(
    paths: &Vec<SimulatedPath>,
    angle: &DimensionParams,
) -> Vec<(f32, Vec<(f32, CombinedRayApproximation)>)> {
    let angles = angle.generate_list();
    println!("Generating combined rays");
    let mut dist_to_rays = Vec::new();
    let mut rays = Vec::new();

    let mut count = 0;

    for path in paths {
        if path.dist != 5. {
            continue;
        }
        let ray = CombinedRayApproximation::generate_optimal(&path.ray, path.dist, angle);
        if count % 100 == 0 {
            println!("Generating combined {}/{}", count + 1, paths.len());
            println!("Generated ray: {:?}", ray);
        }
        rays.push(((path.dist, path.view), ray));
        count += 1;
    }
    rays.sort_by(|a, b| {
        let cmp = a.0 .0.partial_cmp(&b.0 .0);
        if cmp.is_some() {
            return cmp.unwrap();
        } else {
            return a.0 .1.partial_cmp(&b.0 .1).unwrap();
        }
    });
    let mut r = (rays[0].0 .0, Vec::new());
    for ((dist, view), ray) in rays {
        if dist != r.0 {
            dist_to_rays.push(r);
            r = (dist, Vec::new());
        }
        r.1.push((view, ray));
    }
    dist_to_rays.push(r);
    dist_to_rays
}

fn plot_ray_property_by_dist(
    rays: &Vec<(f32, Vec<(f32, CombinedRayApproximation)>)>,
    angle: &DimensionParams,
    property: &dyn Fn(&CombinedRayApproximation, f32, f32) -> (f32, f32),
    file_name: &str,
    folder: &str,
    plot_title: &str,
    bounds: ((f64, f64), (f64, f64)),
) {
    println!("Generating {}", plot_title);
    let angles = angle.generate_list();
    for (dist, rays) in rays {
        let mut d_paths = Vec::new();
        let mut path = Vec::new();
        for (view, ray) in rays {
            path.push(property(ray, *dist, *view));
        }
        d_paths.push(path);
        println!("Generating {}, dist: {}", plot_title, dist);
        plot_with_title(
            &format!("{} at dist = {}", plot_title, dist),
            &format!(
                "generate_artifacts/output/combined_approx/{}/{}_{}.png",
                folder, file_name, dist
            ),
            &d_paths,
            bounds,
        )
        .unwrap();
    }
}

fn plot_property_by_dist(
    rays: &Vec<(f32, Vec<(f32, CombinedRayApproximation)>)>,
    angle: &DimensionParams,
    property: &dyn Fn(&CombinedRayApproximation, f32, f32, f32) -> (f32, f32),
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
                "generate_artifacts/output/combined_approx/{}/{}_{}.png",
                folder, file_name, dist
            ),
            &d_paths,
            bounds,
        )
        .unwrap();
    }
}
