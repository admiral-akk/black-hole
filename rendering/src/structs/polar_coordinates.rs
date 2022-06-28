use std::f32::consts::{FRAC_PI_2, TAU};

use glam::Vec3;

use super::data::Data;

#[derive(Clone)]
pub struct PolarCoordinates {
    pub phi: f32,
    pub theta: f32,
}

impl PolarCoordinates {
    // Note that y is up, not z.
    pub fn new(vec: &Vec3) -> PolarCoordinates {
        let horizontal_len = (vec.x * vec.x + vec.z * vec.z).sqrt();
        let mut phi = fast_math::atan2(vec.z, vec.x);
        if phi < 0.0 {
            phi += TAU;
        }
        PolarCoordinates {
            phi,
            theta: fast_math::atan2(vec.y, horizontal_len) + FRAC_PI_2,
        }
    }

    pub fn to_polar(data: &mut Vec<Data>) {
        for sample in data.iter_mut() {
            match sample {
                Data::FinalDir(index, final_dir) => {
                    *sample = Data::Polar(*index, PolarCoordinates::new(final_dir));
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use glam::Vec3;

    use super::PolarCoordinates;

    #[test]
    fn polar_coordinates_phi() {
        let test_cases = [
            (Vec3::X, 0.0),
            (Vec3::X + Vec3::Y, 0.0),
            (Vec3::X - Vec3::Y, 0.0),
            (Vec3::Z, 0.5 * PI),
            (Vec3::Z + Vec3::Y, 0.5 * PI),
            (Vec3::Z - Vec3::Y, 0.5 * PI),
            (-Vec3::X, 1.0 * PI),
            (-Vec3::X + Vec3::Y, 1.0 * PI),
            (-Vec3::X - Vec3::Y, 1.0 * PI),
            (-Vec3::Z, 1.5 * PI),
            (-Vec3::Z + Vec3::Y, 1.5 * PI),
            (-Vec3::Z - Vec3::Y, 1.5 * PI),
        ];
        for (vector, phi) in test_cases {
            let polar = PolarCoordinates::new(&vector);
            assert_eq!(
                (polar.phi - phi).abs() < 0.0001,
                true,
                "Vector: {:?}",
                vector
            );
        }
    }

    #[test]
    fn polar_coordinates_theta() {
        let test_cases = [
            (Vec3::Y, PI),
            (Vec3::Y + Vec3::X, 0.75 * PI),
            (Vec3::Y - Vec3::X, 0.75 * PI),
            (Vec3::Y + Vec3::Z, 0.75 * PI),
            (Vec3::Y - Vec3::Z, 0.75 * PI),
            (Vec3::X, 0.5 * PI),
            (Vec3::Z, 0.5 * PI),
            (-Vec3::X, 0.5 * PI),
            (-Vec3::Z, 0.5 * PI),
            (-Vec3::Y + Vec3::X, 0.25 * PI),
            (-Vec3::Y - Vec3::X, 0.25 * PI),
            (-Vec3::Y + Vec3::Z, 0.25 * PI),
            (-Vec3::Y - Vec3::Z, 0.25 * PI),
            (-Vec3::Y, 0.0),
        ];
        for (vector, theta) in test_cases {
            let polar = PolarCoordinates::new(&vector);
            assert_eq!(
                (polar.theta - theta).abs() < 0.0001,
                true,
                "Vector: {:?}",
                vector
            );
        }
    }
}
