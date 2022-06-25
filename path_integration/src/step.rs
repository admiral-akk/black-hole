use crate::{particle::Particle, Field};

pub fn step_particle(particle: &mut Particle, field: &Field) {
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

// 0.1 is fast enough to compute an image
fn step_size(particle: &mut Particle, field: &Field) -> f64 {
    let diff = field.center - particle.p;
    let v = particle.v.length();
    let r = diff.length();
    let m_4 = 4.0 * field.m;
    if r > m_4 {
        if diff.dot(particle.v) > 0.0 {
            return 0.1 * (r - m_4) + 0.001 / v;
        } else {
            return 0.1 * r * r / v;
        }
    } else {
        return 0.001 / v;
    }
}

pub fn hit(particle: &Particle, field: &Field) -> bool {
    (particle.p - field.center).length() < field.schwarzchild_radius()
}
