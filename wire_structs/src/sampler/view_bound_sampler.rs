use std::f32::consts::TAU;

use glam::Vec2;

use super::{
    dimension_params::DimensionParams,
    gpu::{
        field::{Field, Particle},
        gpu_state::{simulate_particles, simulate_particles_groups, SimulatedRay},
    },
    render_params::RenderParams,
    texture::texture_1d::Texture1D,
};

pub struct ViewBoundSampler {
    texture: Texture1D,
    dist: DimensionParams,
    view_bounds: [f32; 2],
}

pub enum ViewType {
    Close,
    Far,
}

fn remap_view_01(view_01: f32) -> f32 {
    view_01 * view_01
}

fn invert_view_01(view_01: f32) -> f32 {
    view_01.sqrt()
}

impl ViewBoundSampler {
    pub fn get_view_01(&self, dist: f32, view: f32) -> (ViewType, f32) {
        let dist_01 = self.dist.val_to_01(dist);
        let bound = self.texture.get([dist_01]);
        let view_type = match view > bound {
            true => ViewType::Far,
            false => ViewType::Close,
        };

        let (low, delta) = match view_type {
            ViewType::Close => (bound, self.view_bounds[0] - bound),
            ViewType::Far => (bound, self.view_bounds[1] - bound),
        };

        let view_01 = (view - low) / delta;
        let view_01 = remap_view_01(view_01);
        (view_type, view_01)
    }
}

const ITERATIONS: usize = 2;
fn generate_particle_with_fixed_dist(
    dist: f32,
    view: &DimensionParams,
    render_params: &RenderParams,
) -> Vec<Particle> {
    let views = view.generate_list();
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

fn get_max_angle_index(ray: &SimulatedRay) -> usize {
    ray.angle_dist
        .iter()
        .enumerate()
        .find(|(_, v)| **v == 0.)
        .unwrap()
        .0
}

fn get_closest_ray_index(rays: &Vec<SimulatedRay>) -> usize {
    rays.iter()
        .enumerate()
        .rev()
        .max_by(|r1, r2| get_max_angle_index(&r1.1).cmp(&get_max_angle_index(&r2.1)))
        .unwrap()
        .0
}

fn find_near_miss(
    dist: f32,
    view: &DimensionParams,
    min_angle: f32,
    max_dist: f32,
    render_params: &RenderParams,
) -> f32 {
    let mut view = view.clone();
    let angles = DimensionParams {
        size: 128,
        bounds: [min_angle, 2. * TAU],
    };

    for _ in 0..ITERATIONS {
        let particles = generate_particle_with_fixed_dist(dist, &view, render_params);
        let rays = simulate_particles(particles, &angles, max_dist);
        let closest_index = get_closest_ray_index(&rays);
        view.bounds = [
            view.index_to_val(closest_index - 1),
            view.index_to_val(closest_index),
        ]
    }
    return view.bounds[1];
}

impl ViewBoundSampler {
    pub fn generate(
        dist: DimensionParams,
        view: DimensionParams,
        angle: DimensionParams,
        render_params: &RenderParams,
    ) -> Self {
        let mut texture = Texture1D::new([dist.size]);

        let mut view_groups = Vec::new();
        let angles = DimensionParams {
            size: 128,
            bounds: [angle.bounds[0], 2. * TAU],
        };

        for (_, _) in dist.generate_list().iter().enumerate() {
            view_groups.push(view.clone());
        }

        for _ in 0..ITERATIONS {
            let mut particle_groups = Vec::new();
            for (i, dist) in dist.generate_list().iter().enumerate() {
                let particles =
                    generate_particle_with_fixed_dist(*dist, &view_groups[i], render_params);
                particle_groups.push(particles);
            }

            let ray_groups = simulate_particles_groups(particle_groups, &angles, 40.);
            for (i, rays) in ray_groups.iter().enumerate() {
                let closest_index = get_closest_ray_index(&rays);
                view_groups[i].bounds = [
                    view.index_to_val(closest_index - 1),
                    view.index_to_val(closest_index),
                ]
            }
        }
        for (i, _) in dist.generate_list().iter().enumerate() {
            texture.insert([i], view_groups[i].bounds[1])
        }
        let view_bounds = view.bounds;
        Self {
            texture,
            dist,
            view_bounds,
        }
    }

    pub fn generate_list(&self, view_type: ViewType, distance: f32, count: usize) -> Vec<f32> {
        let dist_01 = self.dist.val_to_01(distance);
        let bound = self.texture.get([dist_01]);
        let (low, delta) = match view_type {
            ViewType::Close => (bound, self.view_bounds[0] - bound),
            ViewType::Far => (bound, self.view_bounds[1] - bound),
        };
        (0..count)
            .map(|i| i as f32 / (count - 1) as f32)
            .map(|v_01| invert_view_01(v_01))
            .map(|v_01| v_01 * delta + low)
            .collect()
    }
}
