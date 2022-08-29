use glam::Vec2;
use itertools::iproduct;

use super::{
    dimension_params::DimensionParams,
    gpu::{
        field::{Field, Particle},
        gpu_state::simulate_particles,
    },
    path_sampler::SimulatedPath,
    render_params::RenderParams,
};

fn generate_particle(dist: f32, view: f32, render_params: &RenderParams) -> Particle {
    let field = Field::new(1.5, dist as f64);
    field.spawn_particle(dist * Vec2::NEG_Y, render_params.view_coord_to_vec(view))
}

pub fn generate_paths(
    dist: &DimensionParams,
    view: &DimensionParams,
    angle: &DimensionParams,
    render_params: &RenderParams,
) -> Vec<SimulatedPath> {
    let dist = dist.generate_list();
    let view = view.generate_list();

    let particles: Vec<Particle> = iproduct!(dist.iter(), view.iter())
        .map(|(d, v)| generate_particle(*d, *v, render_params))
        .collect();

    let rays = simulate_particles(particles, &angle, 40.);

    rays.into_iter()
        .enumerate()
        .map(move |(i, ray)| {
            let mut ray = ray;
            ray.angle_dist = ray
                .angle_dist
                .into_iter()
                .map(|v| {
                    if f32::is_nan(v) {
                        return 0.;
                    }
                    v
                })
                .collect();
            SimulatedPath {
                ray: ray,
                dist: dist[i / view.len()],
                view: view[i % view.len()],
            }
        })
        .collect()
}
