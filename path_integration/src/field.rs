use geometry::DVec3;

use crate::particle;

pub struct Field {
    pub center: DVec3,
    pub magnitude: f64,
    pub m: f64,
}

impl Field {
    pub fn new_rad(center: DVec3, radius: f64, camera_pos: &DVec3) -> Self {
        let r_0 = (center - *camera_pos).length();
        let magnitude = 2.0 / ((2.0 / radius.powi(4)) - (1.0 / r_0.powi(4)));
        Self {
            center,
            magnitude,
            m: 0.5 * radius,
        }
    }

    pub fn new(center: DVec3, magnitude: f64) -> Self {
        Self {
            center,
            magnitude,
            m: 1.0,
        }
    }

    pub fn force(&self, pos: &DVec3) -> DVec3 {
        let diff = self.center - *pos;

        self.magnitude * diff.normalize() / diff.length().powi(5)
    }

    pub fn potential(&self, particle_pos: &DVec3) -> f64 {
        1.0 / (4.0 * (*particle_pos - self.center).length().powi(4))
    }

    // Since we're relying on a mechanical (non-physical) interpretation of the pertubation of a black hole, we
    // have to numerically calculate the radius.
    pub fn calculate_radius(&self) -> f64 {
        0.0
    }
}
