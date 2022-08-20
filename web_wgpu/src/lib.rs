use generate_artifacts::{black_hole_cache::BlackHoleCache, path_distance_cache::distance_cache};
use glam::Mat4;
use shader::{
    black_hole::BlackHole,
    full_float_texture::FullFloatTexture,
    half_float_texture::HalfFloatTexture,
    render_params::RenderParams,
    small_float_texture::SmallFloatTexture,
    texture::Texture,
    vertex::{Vertex, INDICES, VERTICES},
};
use wgpu::{
    util::DeviceExt, BindGroupLayoutDescriptor, BlendState, ColorWrites, DepthBiasState,
    DepthStencilState, Features, Operations, RenderPassDepthStencilAttachment, StencilFaceState,
    StencilOperation, StencilState, TextureFormat,
};
use winit::{
    dpi::LogicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::{self, WindowBuilder},
};

mod shader;
struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    stencil_pipeline: wgpu::RenderPipeline,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    black_hole_buffer: wgpu::Buffer,
    render_params_buffer: wgpu::Buffer,
    params: (BlackHole, RenderParams),
    num_indices: u32,
    stencil_bind_group: wgpu::BindGroup,
    diffuse_bind_group: wgpu::BindGroup,
    depth_texture: Texture,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let mut limits = wgpu::Limits::downlevel_webgl2_defaults();
        limits.max_texture_dimension_1d = 8192;
        limits.max_texture_dimension_2d = 8192;
        limits.max_texture_dimension_3d = 2048;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                        | Features::ADDRESS_MODE_CLAMP_TO_BORDER
                        | Features::TEXTURE_FORMAT_16BIT_NORM,
                    // We're aiming to support WebGl, so we should assume that we're using it.
                    limits: limits,
                    //  if cfg!(target_arch = "wasm32") {
                    //     wgpu::Limits::downlevel_webgl2_defaults()
                    // } else {
                    //     wgpu::Limits::default()
                    // },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let stencil_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Stencil Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("stencil.wgsl").into()),
        });

        let black_hole_cache =
            serde_json::from_slice::<BlackHoleCache>(include_bytes!("black_hole_cache.txt"))
                .unwrap();
        let direction_cache = black_hole_cache.direction_cache;
        let mut z_bounds = Vec::new();
        let mut final_dir_vec = Vec::new();
        let dir_dim = [
            direction_cache.cache_size.1 as u32,
            direction_cache.cache_size.0 as u32,
        ];
        for fixed_distance in direction_cache.distance_angle_to_z_to_distance {
            z_bounds.push([
                fixed_distance.min_z as f32,
                (fixed_distance.max_z - fixed_distance.min_z) as f32,
            ]);
            for (_, final_dir) in fixed_distance.z_to_final_dir {
                final_dir_vec.push([final_dir.0 as f32, final_dir.1 as f32]);
            }
        }

        let distance_cache = black_hole_cache.distance_cache;
        let mut z_bounds_distance = Vec::new();
        let mut z_min_distance = Vec::new();
        let mut z_max_distance = Vec::new();
        let mut angle_distance_vec = Vec::new();
        let dist_dim = [
            distance_cache.cache_size.2 as u32,
            distance_cache.cache_size.1 as u32,
            distance_cache.cache_size.0 as u32,
        ];
        let bounds = distance_cache.disc_bounds;
        for fixed_distance in distance_cache.distance_angle_to_z_to_distance {
            let mut is_decrease = false;
            let mut is_first = true;
            let mut max_is_increase = false;

            println!("dist: {}", fixed_distance.camera_distance);
            for fixed_angle in fixed_distance.angle_to_z_to_distance {
                let min = (fixed_angle.z_bounds.1 - fixed_angle.z_bounds.0) as f32;
                let max = fixed_angle.z_bounds.0 as f32;
                println!("min: {}", min);
                println!("max: {}", max);
                let len = z_min_distance.len();
                if is_first {
                    is_first = false;
                } else {
                    if is_decrease {
                        assert!(z_min_distance[len - 1] > min);
                    } else if z_min_distance[len - 1] > min {
                        is_decrease = true;
                    }
                    if max_is_increase {
                        assert!(z_max_distance[len - 1] < max);
                    }
                    if z_max_distance[len - 1] < max {
                        max_is_increase = true;
                    }
                }
                z_min_distance.push(min);

                z_max_distance.push(max);
                z_bounds_distance.push([
                    fixed_angle.z_bounds.0 as f32,
                    (fixed_angle.z_bounds.1 - fixed_angle.z_bounds.0) as f32,
                ]);
                for d in fixed_angle.z_to_distance {
                    angle_distance_vec.push(((d - bounds.0) / (bounds.1 - bounds.0)) as f32);
                }
            }
        }

        let dist_z_min_tex = SmallFloatTexture::from_f32(
            &device,
            &queue,
            &z_min_distance,
            [dist_dim[1], dist_dim[2]],
            "Distance z bounds",
        )
        .unwrap();
        let dist_z_min_tex_view = dist_z_min_tex.view;
        let dist_z_min_sampler = dist_z_min_tex.sampler;

        let dist_z_max_tex = SmallFloatTexture::from_f32(
            &device,
            &queue,
            &z_max_distance,
            [dist_dim[1], dist_dim[2]],
            "Distance z bounds",
        )
        .unwrap();
        let dist_z_max_tex_view = dist_z_max_tex.view;
        let dist_z_max_sampler = dist_z_max_tex.sampler;

        let dist_z_bounds_tex = FullFloatTexture::from_f32(
            &device,
            &queue,
            &z_bounds_distance,
            [dist_dim[1], dist_dim[2]],
            "Distance z bounds",
        )
        .unwrap();
        let dist_z_bounds_tex_view = dist_z_bounds_tex.view;
        let dist_z_bounds_sampler = dist_z_bounds_tex.sampler;

        let dist_tex = FullFloatTexture::from_f32(
            &device,
            &queue,
            &angle_distance_vec,
            dist_dim,
            "Distance tex",
        )
        .unwrap();

        let dist_tex_view = dist_tex.view;
        let dist_sampler = dist_tex.sampler;

        let dir_z_bounds_tex = FullFloatTexture::from_f32(
            &device,
            &queue,
            &z_bounds,
            z_bounds.len() as u32,
            "Direction z bounds",
        )
        .unwrap();

        let dir_z_bounds_tex_view = dir_z_bounds_tex.view;
        let dir_z_bounds_sampler = dir_z_bounds_tex.sampler;
        let final_dir_tex = FullFloatTexture::from_f32(
            &device,
            &queue,
            &final_dir_vec,
            dir_dim,
            "Final direction texture",
        )
        .unwrap();

        let final_dir_tex_view = final_dir_tex.view;
        let final_dir_sampler = final_dir_tex.sampler;
        let black_hole = BlackHole {
            disc_bounds: [
                distance_cache.disc_bounds.0 as f32,
                distance_cache.disc_bounds.1 as f32,
            ],
            distance_bounds: [
                direction_cache.distance_bounds.0 as f32,
                direction_cache.distance_bounds.1 as f32,
            ],
            radius: [1.5],
        };
        let (black_hole_buffer, _) = black_hole.to_buffer(&device);
        let render_params = RenderParams {
            observer_matrix: Mat4::IDENTITY.to_cols_array(),
            cursor_pos: [0., 0.],
            cache_dim: [dir_dim[0] as f32, dir_dim[1] as f32],
            resolution: [1., 1.],
            distance: [10.],
            time_s: [1.],
            view_width: [2. * f32::tan(std::f32::consts::PI * 60. / 360.)],
        };
        let (render_params_buffer, _) = render_params.to_buffer(&device);

        let depth_texture = Texture::create_depth_texture(&device, &config, "Stencil Texture");
        let galaxy_tex = Texture::from_bytes(
            &device,
            &queue,
            include_bytes!("combined.jpg"),
            "Galaxy backdrop",
        )
        .unwrap();
        let galaxy_tex_view = galaxy_tex.view;
        let galaxy_sampler = galaxy_tex.sampler;
        let noise_tex =
            Texture::from_bytes(&device, &queue, include_bytes!("noise.jpg"), "Noise").unwrap();
        let noise_tex_view = noise_tex.view;
        let noise_tex_sampler = noise_tex.sampler;
        let stencil_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[],
                label: Some("Stencil Bind Group Layout"),
            });

        let stencil_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &stencil_bind_group_layout,
            entries: &Vec::new(),
            label: Some("Stencil Bind Group"),
        });
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D1,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 9,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 10,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 11,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 12,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D3,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 13,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 14,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 15,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 16,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 17,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&galaxy_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&galaxy_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: black_hole_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: render_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&noise_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&noise_tex_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&dir_z_bounds_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&dir_z_bounds_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&final_dir_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::Sampler(&final_dir_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::TextureView(&dist_z_bounds_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: wgpu::BindingResource::Sampler(&dist_z_bounds_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: wgpu::BindingResource::TextureView(&dist_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 13,
                    resource: wgpu::BindingResource::Sampler(&dist_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 14,
                    resource: wgpu::BindingResource::TextureView(&dist_z_min_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 15,
                    resource: wgpu::BindingResource::Sampler(&dist_z_min_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 16,
                    resource: wgpu::BindingResource::TextureView(&dist_z_max_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 17,
                    resource: wgpu::BindingResource::Sampler(&dist_z_max_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });
        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::IncrementClamp,
        };
        let stencil_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Stencil Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &stencil_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &stencil_shader,
                entry_point: "fs_main",
                targets: &[],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: StencilState {
                    front: stencil_state,
                    back: stencil_state,
                    read_mask: 0xff,
                    write_mask: 0xff,
                },
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });
        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Keep,
        };
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: StencilState {
                    front: stencil_state,
                    back: stencil_state,
                    read_mask: 0xff,
                    write_mask: 0xff,
                },
                bias: Default::default(),
            }), // 1.
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let num_indices = INDICES.len() as u32;
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            stencil_pipeline,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            black_hole_buffer,
            render_params_buffer,
            params: (black_hole, render_params),
            num_indices,
            stencil_bind_group,
            diffuse_bind_group,
            depth_texture,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.params
                .1
                .update_resolution([new_size.width as f32, new_size.height as f32]);
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "Stencil Texture");
            self.update_params();
        }
    }

    fn update_params(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Params Encoder"),
            });
        let (black_hole_buffer, render_len) = self.params.0.to_buffer(&self.device);
        encoder.copy_buffer_to_buffer(
            &black_hole_buffer,
            0,
            &self.black_hole_buffer,
            0,
            render_len as u64,
        );
        let (render_params_buffer, render_len) = self.params.1.to_buffer(&self.device);
        encoder.copy_buffer_to_buffer(
            &render_params_buffer,
            0,
            &self.render_params_buffer,
            0,
            render_len as u64,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                modifiers,
            } => match delta {
                MouseScrollDelta::LineDelta(lines, rows) => {
                    let bounds = self.params.0.distance_bounds;
                    self.params.1.update_distance(*rows, bounds);
                    println!(
                        "Lines: {}, rows: {}, distance: {}",
                        lines, rows, self.params.1.distance[0]
                    );
                }
                MouseScrollDelta::PixelDelta(pixels) => {
                    let bounds = self.params.0.distance_bounds;
                    self.params
                        .1
                        .update_distance(pixels.y as f32 / 200., bounds);
                    println!(
                        "Pixels: {:?}, distance: {}",
                        pixels, self.params.1.distance[0]
                    );
                }
            },
            WindowEvent::CursorMoved {
                device_id,
                position,
                modifiers,
            } => {
                let pos = [position.x as f32, position.y as f32];

                self.params.1.update_cursor(pos);
                self.update_params();
            }
            _ => {
                return false;
            }
        }
        true
    }
    fn update(&mut self) {
        // remove `todo!()`
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut stencil_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Stencil Pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(Operations {
                        load: wgpu::LoadOp::Clear(0.),
                        store: true,
                    }),
                    stencil_ops: Some(Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: false,
                    }),
                }),
            });

            stencil_pass.set_pipeline(&self.stencil_pipeline);
            stencil_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            stencil_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            stencil_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            stencil_pass.draw_indexed(0..self.num_indices, 0, 0..1); // 3.
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: Some(Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                }),
            });
            // NEW!
            render_pass.set_pipeline(&self.render_pipeline); // 2.
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]); // NEW!
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_stencil_reference(1);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1); // 3.
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::dpi::PhysicalSize;
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Error).expect("Couldn't initialize logger");
            use wasm_bindgen::JsCast;
             use web_sys::WebGl2RenderingContext;

             window.set_inner_size(PhysicalSize::new(2048, 2048));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                let canvas: web_sys::HtmlCanvasElement =
                    canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

                let gl = canvas
                    .get_context("webgl2")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<WebGl2RenderingContext>()
                    .unwrap();
                gl.get_extension("EXT_color_buffer_float").unwrap();
                gl.get_extension("OES_texture_float_linear").unwrap();
                gl.get_extension("OES_texture_float").unwrap();

                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
        } else {
            env_logger::init();
            window.set_inner_size(PhysicalSize::new(1024, 1024));
        }
    }

    let mut state = State::new(&window).await;
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}
