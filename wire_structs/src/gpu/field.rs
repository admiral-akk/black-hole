use glam::{DVec3, Vec2};
use wgpu::{util::DeviceExt, Buffer, Device};

use super::particle::Particle;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Field {
    pub magnitude: f32,
    pub radius: f32,
}

impl Field {
    pub fn to_buffer(&self, device: &Device) -> (Buffer, usize) {
        let mut padded_slice: Vec<u8> = bytemuck::cast_slice(&[*self]).to_vec();
        while padded_slice.len() % 8 != 0 {
            padded_slice.push(0);
        }
        (
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Field Buffer"),
                contents: &padded_slice,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
            padded_slice.len(),
        )
    }
}

impl Field {
    pub fn new(radius: f64, camera_distance: f64) -> Self {
        let magnitude = 2.0 / (2.0 / radius.powi(4) - (1.0 / camera_distance.powi(4)));
        Self {
            magnitude: magnitude as f32,
            radius: radius as f32,
        }
    }
    pub fn initial_speed(&self, particle_start: &Vec2) -> f32 {
        let diff = particle_start.length();

        (0.5 * self.magnitude * (2.0 / self.radius.powi(4) - 1.0 / diff.powi(4))).sqrt()
    }
    pub fn spawn_particle(&self, p: Vec2, velocity_direction: Vec2) -> Particle {
        let v = (velocity_direction.normalize() * self.initial_speed(&p)).to_array();
        let p = p.to_array();
        Particle {
            pv: [p[0], p[1], v[0], v[1]],
        }
    }
}
