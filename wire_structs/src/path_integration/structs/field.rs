use glam::DVec3;

pub struct Field {
    pub magnitude: f64,
    pub m: f64,
}

impl Field {
    pub fn new(radius: f64, camera_distance: f64) -> Self {
        let magnitude = 2.0 / ((2.0 / radius.powi(4)) - (1.0 / camera_distance.powi(4)));
        Self {
            magnitude,
            m: 0.5 * radius,
        }
    }

    pub fn force(&self, pos: &DVec3) -> DVec3 {
        let diff: DVec3 = -1.0 * *pos;

        self.magnitude * diff.normalize() / diff.length().powi(5)
    }

    pub fn schwarzchild_radius(&self) -> f64 {
        2.0 * self.m
    }

    pub fn initial_speed(&self, particle_start: &DVec3) -> f64 {
        let diff = particle_start.length();

        (0.5 * self.magnitude * (2.0 / self.schwarzchild_radius().powi(4) - 1.0 / diff.powi(4)))
            .sqrt()
    }

    pub fn spawn_particle(&self, p: DVec3, velocity_direction: DVec3) -> Particle {
        Particle {
            p,
            v: velocity_direction.normalize() * self.initial_speed(&p),
        }
    }
}

pub struct Particle {
    pub p: DVec3,
    pub v: DVec3,
}
