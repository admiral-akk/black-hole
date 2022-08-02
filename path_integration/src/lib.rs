use glam::{DVec3, Vec3};
use structs::particle::Particle;

pub mod cache;
mod structs;
pub use structs::ray::Ray;

pub use structs::field::Field;
use structs::step::{hit, step_particle};

// Takes in a ray and a parameterization of the black hole; returns the path taken.
// Also returns the final direction if it doesn't hit the black hole.
pub fn cast_ray_steps(
    ray: &Ray,
    field: &Field,
    escape_radius: f64,
    max_distance: f64,
) -> (Vec<DVec3>, Option<DVec3>) {
    let mut particle = Particle::new(ray, field);
    let mut distance = 0.0;
    let mut steps = Vec::new();
    while particle.p.length() < escape_radius && distance < max_distance {
        steps.push(particle.p);
        if hit(&particle, field) {
            return (steps, None);
        }
        let prev = particle.p;
        step_particle(&mut particle, field);
        distance += (particle.p - prev).length();
    }
    if distance >= max_distance {
        return (steps, None);
    }
    steps.push(particle.p);
    (steps, Some(particle.v))
}

pub fn find_bound(camera_pos: &Vec3, field: &Field, epsilon: f64, max_distance: f64) -> f64 {
    let (mut miss_z, mut hit_z) = (-1.0, 1.0);
    while hit_z - miss_z > epsilon {
        let z = 0.5 * (hit_z + miss_z);
        let test = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
        let ray = Ray::new(camera_pos.as_dvec3(), test);
        let final_dir = cast_ray_steps(&ray, field, max_distance, 10.0 * max_distance).1;
        if final_dir.is_none() {
            // hit the black hole
            hit_z = test.z;
        } else {
            miss_z = test.z;
        }
    }
    miss_z
}
