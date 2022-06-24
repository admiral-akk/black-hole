use geometry::{Ray, Vec3};
use particle::Particle;

mod field;
mod particle;
pub use field::Field;

fn step_particle(particle: &mut Particle, field: &Field) {
    let h = step_size(particle, field);
    particle.v += h * field.force(&particle.p);
    particle.p += h * particle.v;

    //
}

fn step_size(particle: &mut Particle, field: &Field) -> f32 {
    let diff = particle.p - field.center;
    let r = diff.length();
    let is_facing = particle.v.normalize().dot(diff) >= 0.0;

    let hit_dist = 0.15;
    return 0.0001;

    if r > hit_dist {
        if is_facing {
            return 0.1 * r * r;
        } else {
            return 0.1 * (r - hit_dist) + 0.0001;
        }
    } else {
        return 0.00001;
    }
}

fn runge_kutta(h: f32, t_n: f32, y_n: Vec3, k1: Vec3, k2: Vec3, k3: Vec3, k4: Vec3) -> (f32, Vec3) {
    (
        t_n + h,
        y_n + (1.0 / 6.0) * h * (k1 + 2.0 * k2 + 2.0 * k3 + k4),
    )
}

pub fn hit(particle: &Particle, field: &Field) -> bool {
    (particle.p - field.center).length() < 0.1
}

// Takes in a ray and a parameterization of the black hole; returns the final direction.
pub fn cast_ray_steps(ray: &Ray, field: &Field) -> Option<Vec3> {
    let mut particle = Particle::new(ray.pos, ray.dir);
    while (particle.p - field.center).length() < 10.0 {
        if hit(&particle, field) {
            return None;
        }
        step_particle(&mut particle, field);
    }
    Some(particle.v)
}

// Takes in a ray and a parameterization of the black hole; returns the final direction.
pub fn cast_ray_steps_debug(ray: &Ray, field: &Field) -> Vec<Vec3> {
    let mut particle = Particle::new(ray.pos, ray.dir);
    let mut steps = Vec::new();
    while (particle.p - field.center).length() < 10.0 {
        steps.push(particle.p);
        if hit(&particle, field) {
            return steps;
        }
        step_particle(&mut particle, field);
    }
    steps.push(particle.p);

    steps
}
