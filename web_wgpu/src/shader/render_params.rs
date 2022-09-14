use glam::{Mat3, Mat4, Quat, Vec3};
use wgpu::{util::DeviceExt, Buffer, Device};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderParams {
    pub observer_matrix: [f32; 16],
    pub cursor_pos: [f32; 2],
    pub coords_scale: [f32; 2],
    pub coords_offset: [f32; 2],
    pub resolution: [f32; 2],
    pub distance: [f32; 1],
    pub time_s: [f32; 1],
    pub view_width: [f32; 1],
}

impl RenderParams {
    pub fn update_cursor(&mut self, cursor_pos: [f32; 2]) {
        self.cursor_pos = cursor_pos;
        self.update_observer_matrix();
    }

    pub fn update_resolution(&mut self, resolution: [f32; 2]) {
        self.resolution = resolution;

        self.update_coords();
        self.update_observer_matrix();
    }

    fn update_coords(&mut self) {
        self.coords_scale = [1., 1.];
        self.coords_offset = [0., 0.];
        let (min, max) = (
            self.resolution[0].min(self.resolution[1]),
            self.resolution[0].max(self.resolution[1]),
        );
        let index = match self.resolution[0] > self.resolution[1] {
            true => 0,
            false => 1,
        };
        self.coords_scale[index] = max / min;
        self.coords_offset[index] = -(max - min) / (2. * max);
    }
    pub fn update_distance(&mut self, delta: f32, bounds: [f32; 2]) {
        self.distance[0] = (self.distance[0] + delta).clamp(bounds[0], bounds[1]);
    }

    pub fn update_time(&mut self, time: f32) {
        self.time_s = [time];
    }

    fn update_observer_matrix(&mut self) {
        let theta = self.cursor_pos[0] / self.resolution[0] * std::f32::consts::TAU;
        let phi = (self.cursor_pos[1] / self.resolution[1]) * std::f32::consts::PI;

        let start = Vec3::NEG_Z;
        let intermediate = Vec3::new(f32::cos(theta), 0., f32::sin(theta)).normalize();
        let final_pos = Vec3::new(
            f32::cos(theta) * f32::cos(phi),
            f32::sin(phi),
            f32::sin(theta) * f32::cos(phi),
        )
        .normalize();

        let observer_quat = Quat::from_rotation_arc(start, intermediate);
        let euler = Quat::to_euler(observer_quat, glam::EulerRot::XYZ);
        let observer_mat = Mat3::from_euler(glam::EulerRot::XYZ, euler.0, euler.1, euler.2);
        let observer_quat = Quat::from_rotation_arc(intermediate, final_pos);
        let euler = Quat::to_euler(observer_quat, glam::EulerRot::XYZ);
        let observer_mat =
            Mat3::from_euler(glam::EulerRot::XYZ, euler.0, euler.1, euler.2) * observer_mat;

        self.observer_matrix = Mat4::from_mat3(observer_mat).to_cols_array();
    }
}

impl RenderParams {
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
