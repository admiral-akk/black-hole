use std::f32::consts::PI;

use glam::{DVec3, Vec3, Vec3A};

use crate::structs::polar_coordinates::PolarCoordinates;

pub trait ToPolar {
    fn to_polar(&self) -> PolarCoordinates;
}

impl ToPolar for Vec3 {
    fn to_polar(&self) -> PolarCoordinates {
        PolarCoordinates::new(self)
    }
}

impl ToPolar for Vec3A {
    fn to_polar(&self) -> PolarCoordinates {
        PolarCoordinates::newA(self)
    }
}

pub trait ToVec3 {
    fn to_vec3(&self) -> Vec3;
}

pub trait ToDVec3 {
    fn to_dvec3(&self) -> DVec3;
}

impl ToVec3 for DVec3 {
    fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
}

impl ToVec3 for PolarCoordinates {
    fn to_vec3(&self) -> Vec3 {
        let cos_theta = self.theta.cos();
        Vec3::new(
            self.phi.cos() * cos_theta,
            self.theta.sin(),
            self.phi.sin() * cos_theta,
        )
    }
}

impl ToDVec3 for Vec3 {
    fn to_dvec3(&self) -> DVec3 {
        DVec3::new(self.x as f64, self.y as f64, self.z as f64)
    }
}

pub trait ToDegrees {
    fn to_degrees(&self) -> Self;
}

const DEGREES_TO_RADIANS: f32 = PI / 180.0;
const RADIANS_TO_DEGREES: f32 = 180.0 / PI;
impl ToDegrees for f32 {
    fn to_degrees(&self) -> Self {
        DEGREES_TO_RADIANS * self
    }
}

pub trait ToRadians {
    fn to_radians(&self) -> Self;
}

impl ToRadians for f32 {
    fn to_radians(&self) -> Self {
        RADIANS_TO_DEGREES * self
    }
}
