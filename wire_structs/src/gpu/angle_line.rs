#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AngleLine {
    pub dir: [f32; 2],
    pub temp: [f32; 2],
}

impl AngleLine {
    pub fn new(angle: f32) -> Self {
        Self {
            dir: [angle.sin(), -angle.cos()],
            temp: [0.; 2],
        }
    }
}
