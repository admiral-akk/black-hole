use glam::{DVec3, Vec3};

pub trait PolarAngle<T> {
    fn get_angle(&self) -> T;
}

impl PolarAngle<f32> for Vec3 {
    fn get_angle(&self) -> f32 {
        let mut angle = f32::atan2(self.x, -self.z);
        if angle < 0.0 {
            angle += std::f32::consts::TAU;
        }
        angle
    }
}

impl PolarAngle<f64> for DVec3 {
    fn get_angle(&self) -> f64 {
        let mut angle = f64::atan2(self.x, -self.z);
        if angle < 0.0 {
            angle += std::f64::consts::TAU;
        }
        angle
    }
}
