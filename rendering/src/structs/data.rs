use glam::Vec3;

use super::polar_coordinates::PolarCoordinates;

#[derive(Clone)]
pub enum Data {
    None,
    Sample(usize, f32, f32),
    CanonDir(usize, Vec3),
    FinalDir(usize, PolarCoordinates),
    Polar(usize, PolarCoordinates),
    RGBA(usize, [u8; 4]),
}
