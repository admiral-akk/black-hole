use wgpu::{util::DeviceExt, Buffer, Device};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderParams {
    pub cursor_pos: [f32; 2],
    pub resolution: [f32; 2],
    // disc_bounds: [f32; 2],
    // fov_scale: [f32; 1],
    // normalized_dir: [f32; 3],
    // normalized_up: [f32; 3],
    // normalized_pos: [f32; 3],
    // distance: [f32; 1],
    // distance_bounds: [f32; 2],
    // time_s: [f32; 1],
    // position: [f32; 3],
    // tex_coords: [f32; 2],
}

impl RenderParams {
    pub fn to_buffer(&self, device: &Device) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation Parameter Buffer"),
            contents: bytemuck::cast_slice(&[*self]),
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        })
    }
}
