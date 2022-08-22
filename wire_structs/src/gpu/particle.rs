#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    pub pv: [f32; 4],
    pub index: u32,
    pub black_hole_magnitude: f32,
    pub black_hole_radius: f32,
    pub filler: u32,
}
