use glam::Vec3;

use super::polar_coordinates::PolarCoordinates;

#[derive(Clone)]
pub enum Data {
    None,
    Sample(usize, usize, f32, f32),
    CanonDir(usize, usize, Vec3),
    FinalDir(usize, usize, Vec3),
    Polar(usize, usize, PolarCoordinates),
    RGBA(usize, usize, [u8; 4]),
}
