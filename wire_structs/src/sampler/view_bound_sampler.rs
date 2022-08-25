use std::f32::consts::TAU;

use glam::Vec2;

use super::{
    dimension_params::DimensionParams,
    gpu::{
        field::{Field, Particle},
        gpu_state::simulate_particles_groups,
        simulated_ray::SimulatedRay,
    },
    render_params::RenderParams,
    texture::texture_1d::Texture1D,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewBoundSampler {
    texture: Texture1D,
    dist: DimensionParams,
    view_bounds: [f32; 2],
    power: f32,
}

pub enum ViewType {
    Close,
    Far,
}

impl ViewBoundSampler {
    fn remap_view_01(&self, view_01: f32) -> f32 {
        view_01.powf(self.power)
    }

    fn invert_view_01(&self, view_01: f32) -> f32 {
        view_01.powf(1. / self.power)
    }

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
        let view_01 = self.remap_view_01(view_01);
        (view_type, view_01)
    }
}

const ITERATIONS: usize = 3;
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

impl ViewBoundSampler {
    pub fn generate(
        dist: DimensionParams,
        view: DimensionParams,
        angle: DimensionParams,
        render_params: &RenderParams,
        power: f32,
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

        let view_bounds = view.bounds;
        let view = 0;

        for _ in 0..ITERATIONS {
            let mut particle_groups = Vec::new();
            for (i, dist) in dist.generate_list().iter().enumerate() {
                println!("Before: i: {},  closest: {:?}", i, view_groups[i].bounds);
                let particles =
                    generate_particle_with_fixed_dist(*dist, &view_groups[i], render_params);
                particle_groups.push(particles);
            }

            let ray_groups = simulate_particles_groups(particle_groups, &angles, 40.);
            for (i, rays) in ray_groups.iter().enumerate() {
                let mut closest_index = get_closest_ray_index(&rays);
                if closest_index == 0 {
                    closest_index = 1;
                }
                view_groups[i].bounds = [
                    view_groups[i].index_to_val(closest_index - 1),
                    view_groups[i].index_to_val(closest_index),
                ];
                println!(
                    "After: i: {}, index: {}, closest: {:?}",
                    i, closest_index, view_groups[i].bounds
                );
            }
        }
        for (i, _) in dist.generate_list().iter().enumerate() {
            texture.insert([i], view_groups[i].bounds[1])
        }
        Self {
            texture,
            dist,
            view_bounds,
            power,
        }
    }

    pub fn show_bound(&self) -> Vec<f32> {
        self.dist
            .generate_list()
            .iter()
            .map(|v| self.texture.get([self.dist.val_to_01(*v)]))
            .collect()
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
            .map(|v_01| self.invert_view_01(v_01))
            .map(|v_01| v_01 * delta + low)
            .collect()
    }
}
