use wgpu::{util::DeviceExt, Buffer, Device};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlackHole {
    pub disc_bounds: [f32; 2],
    pub distance_bounds: [f32; 2],
    pub radius: [f32; 1],
}

impl BlackHole {
    pub fn to_buffer(&self, device: &Device) -> (Buffer, usize) {
        let mut padded_slice: Vec<u8> = bytemuck::cast_slice(&[*self]).to_vec();
        while padded_slice.len() % 16 != 0 {
            padded_slice.push(0);
        }
        (
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Simulation Parameter Buffer"),
                contents: &padded_slice,
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            }),
            padded_slice.len(),
        )
    }
}
