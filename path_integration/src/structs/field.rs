use glam::DVec3;

pub struct Field {
    pub magnitude: f64,
    pub m: f64,
}

impl Field {
    pub fn new(radius: f64, camera_pos: &DVec3) -> Self {
        let r_0 = camera_pos.length();
        let magnitude = 2.0 / ((2.0 / radius.powi(4)) - (1.0 / r_0.powi(4)));
        Self {
            magnitude,
            m: 0.5 * radius,
        }
    }

    pub fn force(&self, pos: &DVec3) -> DVec3 {
        let diff = -1.0 * *pos;

        self.magnitude * diff.normalize() / diff.length().powi(5)
    }

    pub fn potential(&self, particle_pos: &DVec3) -> f64 {
        1.0 / (4.0 * particle_pos.length().powi(4))
    }

    pub fn schwarzchild_radius(&self) -> f64 {
        2.0 * self.m
    }

    pub fn initial_speed(&self, particle_start: &DVec3) -> f64 {
        let diff = particle_start.length();

        (0.5 * self.magnitude * (2.0 / self.schwarzchild_radius().powi(4) - 1.0 / diff.powi(4)))
            .sqrt()
    }
}
