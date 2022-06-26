use glam::DVec3;
use particle::Particle;

mod ray;
mod structs;
pub use ray::Ray;
pub use structs::ray_cache::RayCache;

mod field;
mod particle;
mod step;
pub use field::Field;
use step::{hit, step_particle};

// Takes in a ray and a parameterization of the black hole; returns the final direction.
pub fn cast_ray_steps(ray: &Ray, field: &Field, max_distance: f64) -> Option<DVec3> {
    let mut particle = Particle::new(ray, field);
    let mut distance = 0.0;
    while (particle.p - field.center).length() < 20.0 && distance < max_distance {
        if hit(&particle, field) {
            return None;
        }
        let prev = particle.p;
        step_particle(&mut particle, field);
        distance += (particle.p - prev).length();
    }
    if distance >= max_distance {
        return None;
    }

    Some(particle.v)
}

// Takes in a ray and a parameterization of the black hole; returns the final direction.
pub fn cast_ray_steps_debug(ray: &Ray, field: &Field, max_distance: f64) -> Vec<DVec3> {
    let mut particle = Particle::new(ray, field);
    let mut steps = Vec::new();
    let mut distance = 0.0;
    while (particle.p - field.center).length() < 10.0 && distance < max_distance {
        steps.push(particle.p);
        if hit(&particle, field) {
            return steps;
        }
        step_particle(&mut particle, field);
        distance += (particle.p - steps[steps.len() - 1]).length();
    }
    steps.push(particle.p);

    steps
}
