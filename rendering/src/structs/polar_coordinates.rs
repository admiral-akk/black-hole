use std::f32::consts::TAU;

use glam::Vec3;

use super::data::Data;

#[derive(Clone, Debug)]
pub struct PolarCoordinates {
    pub phi: f32,
    pub theta: f32,
}

impl PolarCoordinates {
    // Note that y is up, not z.
    pub fn new(vec: &Vec3) -> PolarCoordinates {
        let horizontal_len = (vec.x * vec.x + vec.z * vec.z).sqrt();
        let mut phi = f32::atan2(vec.z, vec.x);
        if phi < 0.0 {
            phi += TAU;
        }
        PolarCoordinates {
            phi,
            theta: f32::atan2(vec.y, horizontal_len),
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

    pub fn to_vec(&self) -> Vec3 {
        let cos_theta = self.theta.cos();
        Vec3::new(
            self.phi.cos() * cos_theta,
            self.theta.sin(),
            self.phi.sin() * cos_theta,
        )
    }
}

pub trait Polar {
    fn to_polar(&self) -> PolarCoordinates;
}

impl Polar for Vec3 {
    fn to_polar(&self) -> PolarCoordinates {
        PolarCoordinates::new(self)
    }
}

#[cfg(test)]
mod tests {
    const EPSILON: f32 = 0.01;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    use glam::Vec3;

    use super::{Polar, PolarCoordinates};

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
                (polar.phi - phi).abs() < EPSILON,
                true,
                "Vector: {:?}",
                vector
            );
        }
    }

    #[test]
    fn polar_coordinates_theta() {
        let test_cases = [
            (Vec3::Y, FRAC_PI_2),
            (Vec3::Y + Vec3::X, FRAC_PI_4),
            (Vec3::Y - Vec3::X, FRAC_PI_4),
            (Vec3::Y + Vec3::Z, FRAC_PI_4),
            (Vec3::Y - Vec3::Z, FRAC_PI_4),
            (Vec3::X, 0.0),
            (Vec3::Z, 0.0),
            (-Vec3::X, 0.0),
            (-Vec3::Z, 0.0),
            (-Vec3::Y + Vec3::X, -FRAC_PI_4),
            (-Vec3::Y - Vec3::X, -FRAC_PI_4),
            (-Vec3::Y + Vec3::Z, -FRAC_PI_4),
            (-Vec3::Y - Vec3::Z, -FRAC_PI_4),
            (-Vec3::Y, -FRAC_PI_2),
        ];
        for (vector, theta) in test_cases {
            let polar = PolarCoordinates::new(&vector);
            assert_eq!(
                (polar.theta - theta).abs() < EPSILON,
                true,
                "Vector: {:?}",
                vector
            );
        }
    }

    #[test]
    fn to_polar_idempotent() {
        let epsilon = EPSILON;
        for x in [-Vec3::X, Vec3::ZERO, Vec3::X] {
            for y in [-Vec3::Y, Vec3::ZERO, Vec3::Y] {
                for z in [-Vec3::Z, Vec3::ZERO, Vec3::Z] {
                    let mut v = x + y + z;
                    if v == Vec3::ZERO {
                        continue;
                    }
                    v = v.normalize();
                    assert_eq!(
                        (v - v.to_polar().to_vec()).length() < epsilon,
                        true,
                        "\npolar: {:?}\nvec: {:?}\nnve: {:?}\ndist: {}",
                        v.to_polar(),
                        v,
                        v.to_polar().to_vec(),
                        (v - v.to_polar().to_vec()).length()
                    );
                }
            }
        }
    }
}
