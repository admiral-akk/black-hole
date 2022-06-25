use geometry::{DVec3, Ray};
use particle::Particle;

mod field;
mod particle;
pub use field::Field;

fn step_particle(particle: &mut Particle, field: &Field) {
    let h = step_size(particle, field);

    // dx/dt
    // dv/dt

    let k_0 = h * particle.v;
    let l_0 = h * field.force(&particle.p);

    let k_1 = h * (particle.v + 0.5 * l_0);
    let l_1 = h * field.force(&(particle.p + 0.5 * k_0));

    let k_2 = h * (particle.v + 0.5 * l_1);
    let l_2 = h * field.force(&(particle.p + 0.5 * k_1));

    let k_3 = h * (particle.v + l_2);
    let l_3 = h * field.force(&(particle.p + k_2));

    particle.p += (1.0 / 6.0) * (k_0 + 2.0 * k_1 + 2.0 * k_2 + k_3);
    particle.v += (1.0 / 6.0) * (l_0 + 2.0 * l_1 + 2.0 * l_2 + l_3);
}

fn step_size(_particle: &mut Particle, _field: &Field) -> f64 {
    0.0001
}

pub fn hit(particle: &Particle, field: &Field) -> bool {
    (particle.p - field.center).length() < 0.1
}

// Takes in a ray and a parameterization of the black hole; returns the final direction.
pub fn cast_ray_steps(ray: &Ray, field: &Field, max_distance: f64) -> Option<DVec3> {
    let mut particle = Particle::new(ray.pos, ray.dir);
    let mut distance = 0.0;
    while (particle.p - field.center).length() < 10.0 && distance < max_distance {
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
    let mut particle = Particle::new(ray.pos, ray.dir);
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
