use geometry::{Ray, Vec3};
use particle::Particle;

mod particle;

fn field(pos: &Vec3, force_scale: f32, field_center: &Vec3) -> Vec3 {
    let diff = *pos - *field_center;

    force_scale * diff.normalize() / diff.length().powi(5)
}

fn step_particle(particle: &mut Particle, force_scale: f32, field_center: &Vec3, h: f32) {
    particle.v += h * field(&particle.p, force_scale, field_center);
    particle.p += h * particle.v;
}
fn runge_kutta(h: f32, t_n: f32, y_n: Vec3, k1: Vec3, k2: Vec3, k3: Vec3, k4: Vec3) -> (f32, Vec3) {
    (
        t_n + h,
        y_n + (1.0 / 6.0) * h * (k1 + 2.0 * k2 + 2.0 * k3 + k4),
    )
}

// Takes in a ray and a parameterization of the black hole; returns the final direction.
pub fn cast_ray_steps(ray: &Ray, force_scale: f32, field_center: &Vec3) -> Vec3 {
    let mut particle = Particle::new(ray.pos, ray.dir);
    for _ in 0..10 {
        step_particle(&mut particle, force_scale, field_center, 0.001);
    }

    particle.v
}

// Takes in a ray and a parameterization of the black hole; returns the final direction.
pub fn cast_ray_steps_debug(ray: &Ray, force_scale: f32, field_center: &Vec3) -> Vec<Vec3> {
    let mut particle = Particle::new(ray.pos, ray.dir);
    let mut steps = Vec::new();
    for _ in 0..10000 {
        steps.push(particle.p);
        step_particle(&mut particle, force_scale, field_center, 0.01);
    }
    steps.push(particle.p);

    steps
}
