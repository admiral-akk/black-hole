use std::ops::{Div, Mul, Sub};

#[derive(Debug, PartialEq, Clone)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn xyz(&self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        let l_xyz = self.xyz();
        let r_xyz = other.xyz();

        l_xyz.0 * r_xyz.0 + l_xyz.1 * r_xyz.1 + l_xyz.2 * r_xyz.2
    }

    fn len(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(&self) -> Vec3 {
        self / self.len()
    }

    pub fn normalize(&mut self) {
        let len = self.len();
        self.x /= len;
        self.y /= len;
        self.z /= len;
    }
}

impl Mul<&Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, rhs: &Vec3) -> Vec3 {
        let (x, y, z) = rhs.xyz();
        Vec3 {
            x: self * x,
            y: self * y,
            z: self * z,
        }
    }
}

impl Mul<f32> for &Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Vec3 {
        rhs * self
    }
}

impl Sub for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: &Vec3) -> Vec3 {
        let l_xyz = self.xyz();
        let r_xyz = rhs.xyz();

        Vec3::new(l_xyz.0 - r_xyz.0, l_xyz.1 - r_xyz.1, l_xyz.2 - r_xyz.2)
    }
}

impl Div<f32> for &Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Vec3 {
        let xyz = self.xyz();

        Vec3::new(xyz.0 / rhs, xyz.1 / rhs, xyz.2 / rhs)
    }
}
