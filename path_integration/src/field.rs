use geometry::DVec3;

pub struct Field {
    pub center: DVec3,
    pub magnitude: f64,
}

impl Field {
    pub fn new(center: DVec3, magnitude: f64) -> Self {
        Self { center, magnitude }
    }

    pub fn force(&self, pos: &DVec3) -> DVec3 {
        let diff = self.center - *pos;

        self.magnitude * diff.normalize() / diff.length().powi(5)
    }

    pub fn calculate_radius(&self) -> f64 {
        0.0
    }
}
