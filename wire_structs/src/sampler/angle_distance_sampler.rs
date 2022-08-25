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
    texture::texture_3d::Texture3D,
    view_bound_sampler::{ViewBoundSampler, ViewType},
};

#[derive(Serialize, Deserialize)]
pub struct AngleDistanceSampler {
    close: Texture3D,
    far: Texture3D,
    distance: DimensionParams,
    angle: DimensionParams,
    view: DimensionParams,
    view_sampler: ViewBoundSampler,
}



impl AngleDistanceSampler {
    pub fn get_dist(&self, dist: f32, view: f32, angle: f32) -> f32 {
        let (view_type, view_01) = self.view_sampler.get_view_01(dist, view);
        let dist_01 = self.distance.val_to_01(dist);
        let angle_01 = self.angle.val_to_01(angle);
        match view_type {
            ViewType::Close => &self.close,
            ViewType::Far => &self.far,
        }
        .get([dist_01, view_01, angle_01])
    }
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

fn fill_texture(tex: &mut Texture3D, d_index: usize, rays: &Vec<SimulatedRay>, delta_theta: f32) {
    let delta_theta_2 = 2. * delta_theta;
    for (view_index, ray) in rays.iter().enumerate() {
        let mut angle_dist: Vec<f32> = ray.angle_dist.iter().map(|v| *v).collect();
        for angle_index in 0..angle_dist.len() {
            let mut dist_at_angle = angle_dist[angle_index];
            if f32::is_nan(dist_at_angle) {
                dist_at_angle = 0.;
            }
            if angle_index > 1 && dist_at_angle == 0. {
                let d_1 = angle_dist[angle_index - 2];
                let d_2 = angle_dist[angle_index - 1];
                if d_2 == 0. {
                    dist_at_angle = 0.;
                } else if d_2 >= 40. {
                    dist_at_angle = 40.
                } else if d_2 <= d_1 {
                    dist_at_angle = 0.;
                } else {
                    dist_at_angle = 40.
                }
            }
            if false {
                let d_1 = angle_dist[angle_index - 2];
                let d_2 = angle_dist[angle_index - 1];
                let p_3 = [delta_theta_2.cos(), delta_theta_2.sin()];

                let d_3 = d_1 * d_2 * delta_theta.sin()
                    / (p_3[0] * (delta_theta.cos() * d_2 - d_1) - p_3[1] * delta_theta.sin() * d_2);
                let d_3 = d_3.abs();
                if d_3 > 0. {
                    dist_at_angle = d_3;
                } else {
                    dist_at_angle = d_2;
                }
            }
            tex.insert([d_index, view_index, angle_index], dist_at_angle);
            angle_dist[angle_index] = dist_at_angle;
        }
    }
}

impl AngleDistanceSampler {
    pub fn generate(
        distance: DimensionParams,
        angle: DimensionParams,
        view: DimensionParams,
        view_sampler: ViewBoundSampler,
        render_params: &RenderParams,
    ) -> Self {
        let angles = angle.generate_list();
        let delta_theta = angles[1] - angles[0];
        let mut close = Texture3D::new([distance.size, view.size, angle.size]);
        let mut far = Texture3D::new([distance.size, view.size, angle.size]);
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

        for (d_index, dist) in dists.iter().enumerate() {
            let far_rays = &ray_groups[d_index];
            fill_texture(&mut far, d_index, far_rays, delta_theta);
        }

        for (d_index, dist) in dists.iter().enumerate() {
            let close_rays = &ray_groups[d_index + dists.len()];
            fill_texture(&mut close, d_index, close_rays, delta_theta);
        }
        Self {
            close,
            far,
            distance,
            angle,
            view,
            view_sampler,
        }
    }
}
