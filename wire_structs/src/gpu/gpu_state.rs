use std::time::{Instant, SystemTime};

use glam::Vec2;
use wgpu::util::DeviceExt;

use bytemuck;

use crate::{
    gpu::{field::Field, particle::Particle},
    path_integration::path,
};

async fn run(particle_count: u32, steps: u32, samples: u32) -> Vec<Vec<[[f32; 2]; 2]>> {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let adapter = instance.request_adapter(&Default::default()).await.unwrap();
    let features = adapter.features();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: features & wgpu::Features::TIMESTAMP_QUERY,
                limits: Default::default(),
            },
            None,
        )
        .await
        .unwrap();

    let start_instant = Instant::now();
    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });
    println!("shader compilation {:?}", start_instant.elapsed());
    let input_f = &[1.0f32, 2.0f32];
    let input: &[u8] = bytemuck::bytes_of(input_f);
    let input_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: input,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let field = Field::new(1.5, 20.);
    let (field_buffer, field_buffer_len) = field.to_buffer(&device);

    let particles: Vec<crate::gpu::particle::Particle> = (0..particle_count)
        .into_iter()
        .map(|i| i as f32 / (particle_count - 1) as f32)
        .map(|i_01| Vec2::new(i_01, 1.).normalize())
        .map(|v| field.spawn_particle(20. * Vec2::NEG_Y, v))
        .collect();

    let particle_bytes = bytemuck::cast_slice(&particles);
    let particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &particle_bytes,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
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
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&compute_pipeline_layout),
        module: &cs_module,
        entry_point: "main",
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: field_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: particle_buffer.as_entire_binding(),
            },
        ],
    });

    let start = SystemTime::now();
    let mut vec = Vec::new();
    for i in 0..steps {
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut cpass = encoder.begin_compute_pass(&Default::default());
            cpass.set_pipeline(&pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(256, 1, 1);
        }

        if (i + 1) % (steps / samples) == 0 {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: particle_bytes.len() as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            encoder.copy_buffer_to_buffer(
                &particle_buffer,
                0,
                &buffer,
                0,
                particle_bytes.len() as u64,
            );
            let index = queue.submit(Some(encoder.finish()));

            vec.push((index, buffer));
        } else {
            queue.submit(Some(encoder.finish()));
        }
    }

    let mut paths = Vec::new();
    for p in particles {
        paths.push(Vec::from([[[p.pv[0], p.pv[1]], [p.pv[2], p.pv[3]]]]));
    }

    for (index, buffer) in vec {
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        let buffer_slice = buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        device.poll(wgpu::Maintain::WaitForSubmissionIndex(index));
        println!("time taken (ms): {}", start.elapsed().unwrap().as_millis());

        if let Some(Ok(())) = receiver.receive().await {
            let data_raw = &*buffer_slice.get_mapped_range();
            let data: &[Particle] = bytemuck::cast_slice(data_raw);
            for (i, p) in data.iter().enumerate() {
                paths[i].push([[p.pv[0], p.pv[1]], [p.pv[2], p.pv[3]]]);
            }
        }
    }

    return paths;
}

pub fn run_main(particle_count: u32, steps: u32, samples: u32) -> Vec<Vec<[[f32; 2]; 2]>> {
    return pollster::block_on(run(particle_count, steps, samples));
}
