use glam::Vec2;

use super::{
    dimension_params::DimensionParams,
    gpu::{
        field::{Field, Particle},
        gpu_state::{simulate_particles, SimulatedRay},
    },
    render_params::RenderParams,
    texture::texture_3d::Texture3D,
    view_bound_sampler::{ViewBoundSampler, ViewType},
};

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

fn fill_texture(tex: &mut Texture3D, d_index: usize, rays: Vec<SimulatedRay>) {
    for (view_index, ray) in rays.iter().enumerate() {
        for (angle_index, dist_at_angle) in ray.angle_dist.iter().enumerate() {
            tex.insert([d_index, view_index, angle_index], *dist_at_angle);
        }
    }
}

impl AngleDistanceSampler {
    pub fn generate(
        distance: DimensionParams,
        angle: DimensionParams,
        view: DimensionParams,
        render_params: RenderParams,
    ) -> Self {
        let view_sampler = ViewBoundSampler::generate(distance, view, angle, &render_params);
        let mut close = Texture3D::new([distance.size, view.size, angle.size]);
        let mut far = Texture3D::new([distance.size, view.size, angle.size]);
        let dists = distance.generate_list();
        for (d_index, dist) in dists.iter().enumerate() {
            let far_views = view_sampler.generate_list(ViewType::Far, *dist, view.size);
            let far_particles = generate_particles(*dist, &far_views, &render_params);
            let far_rays = simulate_particles(far_particles, &angle, 40.);
            fill_texture(&mut far, d_index, far_rays);

            let close_views = view_sampler.generate_list(ViewType::Close, *dist, view.size);
            let close_particles = generate_particles(*dist, &close_views, &render_params);
            let close_rays = simulate_particles(close_particles, &angle, 40.);
            fill_texture(&mut close, d_index, close_rays);
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
