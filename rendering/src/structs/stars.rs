use std::f64::consts::{FRAC_PI_2, TAU};

use glam::DVec3;
use image::{DynamicImage, GenericImageView};

pub struct Stars {
    background: DynamicImage,
}

struct PolarCoordinates {
    pub phi: f64,
    pub theta: f64,
}

impl PolarCoordinates {
    // Note that y is up, not z.
    pub fn new(vec: &DVec3) -> PolarCoordinates {
        let horizontal_len = (vec.x * vec.x + vec.z * vec.z).sqrt();
        let mut phi = f64::atan2(vec.z, vec.x);
        if phi < 0.0 {
            phi += TAU;
        }
        PolarCoordinates {
            phi,
            theta: f64::atan2(vec.y, horizontal_len) + FRAC_PI_2,
        }
    }
}
impl Stars {
    pub fn new(background: DynamicImage) -> Self {
        Self { background }
    }

    pub fn get_rgba(&self, dir: &DVec3) -> [u8; 4] {
        let polar = PolarCoordinates::new(&dir);
        let x = (self.background.width() as f64) * polar.phi / std::f64::consts::TAU;
        let y = (self.background.height() as f64) * polar.theta / std::f64::consts::PI;
        let rgba = self.background.get_pixel(
            (x as u32) % self.background.width(),
            (y as u32) % self.background.height(),
        );

        rgba.0
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use glam::DVec3;

    use super::PolarCoordinates;

    #[test]
    fn polar_coordinates_phi() {
        let test_cases = [
            (DVec3::X, 0.0),
            (DVec3::X + DVec3::Y, 0.0),
            (DVec3::X - DVec3::Y, 0.0),
            (DVec3::Z, 0.5 * PI),
            (DVec3::Z + DVec3::Y, 0.5 * PI),
            (DVec3::Z - DVec3::Y, 0.5 * PI),
            (-DVec3::X, 1.0 * PI),
            (-DVec3::X + DVec3::Y, 1.0 * PI),
            (-DVec3::X - DVec3::Y, 1.0 * PI),
            (-DVec3::Z, 1.5 * PI),
            (-DVec3::Z + DVec3::Y, 1.5 * PI),
            (-DVec3::Z - DVec3::Y, 1.5 * PI),
        ];
        for (vector, phi) in test_cases {
            let polar = PolarCoordinates::new(&vector);
            assert_eq!(polar.phi, phi, "Vector: {:?}", vector);
        }
    }

    #[test]
    fn polar_coordinates_theta() {
        let test_cases = [
            (DVec3::Y, PI),
            (DVec3::Y + DVec3::X, 0.75 * PI),
            (DVec3::Y - DVec3::X, 0.75 * PI),
            (DVec3::Y + DVec3::Z, 0.75 * PI),
            (DVec3::Y - DVec3::Z, 0.75 * PI),
            (DVec3::X, 0.5 * PI),
            (DVec3::Z, 0.5 * PI),
            (-DVec3::X, 0.5 * PI),
            (-DVec3::Z, 0.5 * PI),
            (-DVec3::Y + DVec3::X, 0.25 * PI),
            (-DVec3::Y - DVec3::X, 0.25 * PI),
            (-DVec3::Y + DVec3::Z, 0.25 * PI),
            (-DVec3::Y - DVec3::Z, 0.25 * PI),
            (-DVec3::Y, 0.0),
        ];
        for (vector, theta) in test_cases {
            let polar = PolarCoordinates::new(&vector);
            assert_eq!(polar.theta, theta, "Vector: {:?}", vector);
        }
    }
}
