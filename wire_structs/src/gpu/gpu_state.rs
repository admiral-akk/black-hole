use std::time::SystemTime;

use wgpu::{util::DeviceExt, BindGroupLayout, Buffer, ComputePipeline, Device, Queue};

use bytemuck::{self, Pod};

use crate::dimension_params::DimensionParams;

use super::field::Particle;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct AngleLine {
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

pub struct SimulatorState {
    device: Device,
    bind_group_layout: BindGroupLayout,
    pipeline: ComputePipeline,
    queue: Queue,
}

pub struct SimulatedRay {
    pub angle_dist: Vec<f32>,
    pub final_pos: [f32; 2],
    pub final_dir: [f32; 2],
}

impl SimulatorState {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let adapter = instance.request_adapter(&Default::default()).await.unwrap();
        let features = adapter.features();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Particle Device Descriptor"),
                    features: features & wgpu::Features::TIMESTAMP_QUERY,
                    limits: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Particle Bind Groupd Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Particle Layout Descriptor"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Simulate Particles"),
            layout: Some(&compute_pipeline_layout),
            module: &module,
            entry_point: "main",
        });

        Self {
            device,
            bind_group_layout,
            pipeline,
            queue,
        }
    }

    async fn retrieve_values<T: Pod>(&self, source_buffer: &Buffer, len: u64) -> Vec<T> {
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let staging_particle_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: len,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(&source_buffer, 0, &staging_particle_buffer, 0, len);
        let index = self.queue.submit(Some(encoder.finish()));
        println!("Extracting final dir");

        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        let buffer_slice = staging_particle_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        self.device
            .poll(wgpu::Maintain::WaitForSubmissionIndex(index));

        if let Some(Ok(())) = receiver.receive().await {
            let data_raw = &*buffer_slice.get_mapped_range();
            return bytemuck::cast_slice(data_raw).to_vec();
        } else {
            panic!("Couldn't retrieve values!");
        }
    }

    pub async fn simulate_particles(
        &self,
        particles: &[Particle],
        angles: &DimensionParams,
        distances: &DimensionParams,
    ) -> Vec<SimulatedRay> {
        let device = &self.device;
        let bind_group_layout = &self.bind_group_layout;
        let pipeline = &self.pipeline;
        let queue = &self.queue;

        let particle_bytes = bytemuck::cast_slice(&particles);
        let particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &particle_bytes,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let output_size = 4 * angles.size as u64 * particles.len() as u64;

        println!("Output size: {}", output_size);
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: output_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let (min, delta) = angles.min_delta();
        let angle_lines: Vec<AngleLine> = (0..=angles.size)
            .map(|a| {
                let angle = min + delta * a as f32 / (angles.size - 1) as f32;
                AngleLine::new(angle)
            })
            .collect();
        let angle_lines_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &bytemuck::cast_slice(&angle_lines),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let _start = SystemTime::now();
        let step_count = 1 << 14;
        let pieces = i32::max(step_count >> 10, 1);
        for step in 0..step_count {
            let mut encoder = device.create_command_encoder(&Default::default());
            {
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: output_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: particle_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: angle_lines_buffer.as_entire_binding(),
                        },
                    ],
                });
                let mut cpass = encoder.begin_compute_pass(&Default::default());
                cpass.set_pipeline(&pipeline);
                cpass.set_bind_group(0, &bind_group, &[]);
                cpass.dispatch_workgroups(256, 1, 1);
            }
            let index = queue.submit(Some(encoder.finish()));
            if (step + 1) % (step_count / pieces) == 0 {
                println!(
                    "Waiting on GPU, ({}/{})",
                    (step + 1) / (step_count / pieces),
                    pieces
                );
                device.poll(wgpu::Maintain::WaitForSubmissionIndex(index));
            }
        }
        let particles: Vec<Particle> = self
            .retrieve_values(&particle_buffer, particle_bytes.len() as u64)
            .await;
        let distances: Vec<f32> = self.retrieve_values(&output_buffer, output_size).await;
        particles
            .iter()
            .enumerate()
            .map(|(i, p)| SimulatedRay {
                angle_dist: (0..angles.size)
                    .map(|a| distances[i + a as usize * particles.len()])
                    .collect(),
                final_pos: [p.pv[0], p.pv[1]],
                final_dir: [p.pv[2], p.pv[3]],
            })
            .collect()
    }
}

// This is about 4 million points, each of which takes ~2 bytes. If we do more, it can crash.
// Todo: try a backout technique?
const MAX_PROBLEM_SIZE: usize = 1 << 22;
async fn run(
    particles: Vec<Particle>,
    angles: &DimensionParams,
    distances: &DimensionParams,
) -> Vec<SimulatedRay> {
    let simulator = SimulatorState::new().await;
    let mut rays = Vec::new();

    let particles_per_problem = MAX_PROBLEM_SIZE / angles.size;

    let mut problem_count = particles.len() / particles_per_problem;
    if particles.len() != problem_count * particles_per_problem {
        problem_count += 1;
    }
    for i in 0..problem_count {
        let min = i * particles_per_problem;
        let max = usize::min(min + particles_per_problem, particles.len());
        let particles = &particles[min..max];

        println!("Generating rays, partition: {}/{}", i + 1, problem_count);
        let mut ray_part = simulator
            .simulate_particles(particles, angles, distances)
            .await;
        rays.append(&mut ray_part);
    }
    return rays;
}

pub fn simulate_particles(
    particles: Vec<Particle>,
    angles: &DimensionParams,
    distances: &DimensionParams,
) -> Vec<SimulatedRay> {
    return pollster::block_on(run(particles, angles, distances));
}
