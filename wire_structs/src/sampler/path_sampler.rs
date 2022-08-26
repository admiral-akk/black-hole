use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::{
    dimension_params::DimensionParams,
    gpu::{
        field::{Field, Particle},
        gpu_state::simulate_particles_groups,
        simulated_ray::SimulatedRay,
    },
    render_params::RenderParams,
    view_bound_sampler::{ViewBoundSampler, ViewType},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct SimulatedPath {
    pub ray: SimulatedRay,
    pub dist: f32,
    pub view: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PathSampler {
    pub close: Vec<Vec<SimulatedPath>>,
    pub far: Vec<Vec<SimulatedPath>>,
    distance: DimensionParams,
    angle: DimensionParams,
    view: DimensionParams,
}

fn generate_particles(dist: f32, views: &Vec<f32>, render_params: &RenderParams) -> Vec<Particle> {
    let mut particles = Vec::new();
    let field = Field::new(1.5, dist as f64);
    for v in 0..views.len() {
        let view_coord = views[v];
        particles.push(field.spawn_particle(
            dist * Vec2::NEG_Y,
            render_params.view_coord_to_vec(view_coord),
        ));
    }
    particles
}

impl PathSampler {
    pub fn generate(
        distance: DimensionParams,
        angle: DimensionParams,
        view: DimensionParams,
        view_sampler: &ViewBoundSampler,
        render_params: &RenderParams,
    ) -> Self {
        let dists = distance.generate_list();
        let mut particle_groups = Vec::new();

        for (_, dist) in dists.iter().enumerate() {
            let far_views = view_sampler.generate_list(ViewType::Far, *dist, view.size);
            let far_particles = generate_particles(*dist, &far_views, &render_params);
            particle_groups.push(far_particles);
        }
        for (_, dist) in dists.iter().enumerate() {
            let close_views = view_sampler.generate_list(ViewType::Close, *dist, view.size);
            let close_particles = generate_particles(*dist, &close_views, &render_params);
            particle_groups.push(close_particles);
        }
        let ray_groups = simulate_particles_groups(particle_groups, &angle, 40.);
        let mut far = Vec::new();
        for (d_index, dist) in dists.iter().enumerate() {
            let far_views = view_sampler.generate_list(ViewType::Far, *dist, view.size);

            let far_rays = ray_groups[d_index].to_vec();
            far.push(
                far_rays
                    .iter()
                    .enumerate()
                    .map(|(i, ray)| {
                        let mut ray = ray.clone();
                        for i in 0..ray.angle_dist.len() {
                            if f32::is_nan(ray.angle_dist[i]) {
                                ray.angle_dist[i] = 0.;
                            }
                        }
                        SimulatedPath {
                            ray,
                            dist: *dist,
                            view: far_views[i],
                        }
                    })
                    .collect(),
            );
        }
        let mut close = Vec::new();
        for (d_index, dist) in dists.iter().enumerate() {
            let close_views = view_sampler.generate_list(ViewType::Close, *dist, view.size);

            let close_rays = ray_groups[dists.len() + d_index].to_vec();
            close.push(
                close_rays
                    .iter()
                    .enumerate()
                    .map(|(i, ray)| {
                        let mut ray = ray.clone();
                        for i in 0..ray.angle_dist.len() {
                            if f32::is_nan(ray.angle_dist[i]) {
                                ray.angle_dist[i] = 0.;
                            }
                        }
                        SimulatedPath {
                            ray,
                            dist: *dist,
                            view: close_views[i],
                        }
                    })
                    .collect(),
            );
        }
        Self {
            far,
            close,
            distance,
            angle,
            view,
        }
    }
}
