use glam::{Vec3, Vec3A};

use super::polar_coordinates::PolarCoordinates;

#[derive(Clone)]
pub enum Data {
    None,
    Sample(usize, f32, f32),
    ObserverDir(usize, Vec3A),
    FinalDir(usize, PolarCoordinates),
    Polar(usize, PolarCoordinates),
    RGBA(usize, [u8; 4]),
}
