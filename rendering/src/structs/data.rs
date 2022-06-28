use glam::DVec3;

use super::polar_coordinates::PolarCoordinates;

#[derive(Clone)]
pub enum Data {
    None,
    Sample(usize, usize, f64, f64),
    CanonDir(usize, usize, DVec3),
    NoFinalDir(usize, usize),
    FinalDir(usize, usize, DVec3),
    Polar(usize, usize, PolarCoordinates),
    RGBA(usize, usize, [u8; 4]),
}
