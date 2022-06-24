use geometry::{Ray, Vec3};
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

fn step_size(particle: &mut Particle, field: &Field) -> f32 {
    return 0.001;
    let diff = particle.p - field.center;
    let r = diff.length();
    let is_facing = particle.v.normalize().dot(diff) >= 0.0;

    let hit_dist = 0.15;

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
