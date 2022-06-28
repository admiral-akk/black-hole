use glam::DVec3;

use super::polar_coordinates::PolarCoordinates;

#[derive(Clone)]
pub enum Data {
    None,
    Sample(usize, usize, f32, f32),
    CanonDir(usize, usize, DVec3),
    FinalDir(usize, usize, DVec3),
    Polar(usize, usize, PolarCoordinates),
    RGBA(usize, usize, [u8; 4]),
}
