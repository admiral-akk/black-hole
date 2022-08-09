use glam::DVec3;

use super::{field::Field, field::Particle};

const MIN_STEP: f64 = 0.0002;

pub fn step_particle(particle: &mut Particle, field: &Field) {
    let h = step_size(particle, field);

    let (delta_p, delta_v) = rk4(&particle, field, h);

    particle.p += delta_p;
    particle.v += delta_v;
}

fn step_size(particle: &Particle, field: &Field) -> f64 {
    let diff = -1.0 * particle.p;
    let v = particle.v.length();
    let r = diff.length();
    let m_4 = 8.0 * field.m;
    let h = match r > m_4 {
        true => match diff.dot(particle.v) > 0. {
            true => (0.1 * (r - m_4) + MIN_STEP),
            false => (0.1 * r + MIN_STEP),
        },
        false => MIN_STEP,
    } / (v * v);
    h
}

fn rk4(particle: &Particle, field: &Field, h: f64) -> (DVec3, DVec3) {
    let k_0 = h * particle.v;
    let l_0 = h * field.force(&particle.p);

    let k_1 = h * (particle.v + 0.5 * l_0);
    let l_1 = h * field.force(&(particle.p + 0.5 * k_0));

    let k_2 = h * (particle.v + 0.5 * l_1);
    let l_2 = h * field.force(&(particle.p + 0.5 * k_1));

    let k_3 = h * (particle.v + l_2);
    let l_3 = h * field.force(&(particle.p + k_2));

    (
        (1.0 / 6.0) * (k_0 + 2.0 * k_1 + 2.0 * k_2 + k_3),
        (1.0 / 6.0) * (l_0 + 2.0 * l_1 + 2.0 * l_2 + l_3),
    )
}

pub fn hit(particle: &Particle, field: &Field) -> bool {
    // We add some error so that the geodesics that are on the edge of the schwarzchild radius don't get pulled in accidentally.
    particle.p.length() < 0.85 * field.schwarzchild_radius()
}
