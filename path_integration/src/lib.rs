use glam::DVec3;
use structs::particle::Particle;

mod structs;
pub use structs::ray::Ray;

pub use structs::field::Field;
use structs::step::{hit, step_particle};

// Takes in a ray and a parameterization of the black hole; returns the final direction.
pub fn cast_ray_steps(
    ray: &Ray,
    field: &Field,
    escape_radius: f64,
    max_distance: f64,
) -> Option<(Vec<DVec3>, DVec3)> {
    let mut particle = Particle::new(ray, field);
    let mut distance = 0.0;
    let mut steps = Vec::new();
    while particle.p.length() < escape_radius && distance < max_distance {
        steps.push(particle.p);
        if hit(&particle, field) {
            return None;
        }
        let prev = particle.p;
        step_particle(&mut particle, field);
        distance += (particle.p - prev).length();
    }
    if (distance >= max_distance) {
        return None;
    }
    steps.push(particle.p);
    Some((steps, particle.v))
}

// Takes in a ray and a parameterization of the black hole; returns the final direction.
pub fn cast_ray_steps_debug(ray: &Ray, field: &Field, max_distance: f64) -> Vec<DVec3> {
    let mut particle = Particle::new(ray, field);
    let mut steps = Vec::new();
    let mut distance = 0.0;
    while particle.p.length() < 10.0 && distance < max_distance {
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
