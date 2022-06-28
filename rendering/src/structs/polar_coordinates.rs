use std::f64::consts::{FRAC_PI_2, TAU};

use glam::DVec3;

use super::data::Data;

#[derive(Clone)]
pub struct PolarCoordinates {
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

    pub fn to_polar(data: &mut Vec<Data>) {
        for sample in data.iter_mut() {
            match sample {
                Data::FinalDir(x, y, final_dir) => {
                    *sample = Data::Polar(*x, *y, PolarCoordinates::new(final_dir));
                }
                _ => {}
            }
        }
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
