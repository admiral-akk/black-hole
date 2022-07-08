use glam::Vec3;

use super::polar_coordinates::PolarCoordinates;

#[derive(Clone, Debug)]
pub enum Data {
    None,
    Sample(usize, f32, f32),
    ObserverDir(usize, Vec3),
    FinalDir(usize, PolarCoordinates),
    Polar(usize, PolarCoordinates),
    RGBA(usize, [u8; 4]),
}
