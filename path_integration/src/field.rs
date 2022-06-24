use geometry::Vec3;

pub struct Field {
    pub center: Vec3,
    pub magnitude: f32,
}

impl Field {
    pub fn new(center: Vec3, magnitude: f32) -> Self {
        Self { center, magnitude }
    }

    pub fn force(&self, pos: &Vec3) -> Vec3 {
        let diff = self.center - *pos;

        self.magnitude * diff.normalize() / diff.length().powi(5)
    }
}
