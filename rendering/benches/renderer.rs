use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geometry::Vec3;
use rendering::{
    renderer::renderer::{RenderConfig, RenderType, Renderer},
    structs::dimensions::Dimensions,
};

fn get_renderer() -> Renderer {
    Renderer::new()
}

pub fn renderer_benchmark(c: &mut Criterion) {
    c.bench_function("renderer uv small", |b| {
        let renderer = get_renderer();
        let config = RenderConfig {
            dimensions: Dimensions::new(10, 10),
            render_type: RenderType::UV,
        };
        let mut buf = config.dimensions.get_buffer();
        b.iter(|| black_box(renderer.render(&mut buf, &config)));
    });
    c.bench_function("renderer skybox small", |b| {
        let renderer = get_renderer();
        let config = RenderConfig {
            dimensions: Dimensions::new(10, 10),
            render_type: RenderType::Skybox {
                vertical_fov_degrees: 20.0,
            },
        };
        let mut buf = config.dimensions.get_buffer();
        b.iter(|| black_box(renderer.render(&mut buf, &config)));
    });
    c.bench_function("stars small", |b| {
        let renderer = get_renderer();
        let config = RenderConfig {
            dimensions: Dimensions::new(10, 10),
            render_type: RenderType::Image {
                image: image::open("space-background.jpg").unwrap(),
            },
        };
        let mut buf = config.dimensions.get_buffer();
        b.iter(|| black_box(renderer.render(&mut buf, &config)));
    });
    c.bench_function("ray stars small", |b| {
        let renderer = get_renderer();
        let config = RenderConfig {
            dimensions: Dimensions::new(10, 10),
            render_type: RenderType::RaySkybox {
                vertical_fov_degrees: 20.0,
                image: image::open("space-background.jpg").unwrap(),
            },
        };
        let mut buf = config.dimensions.get_buffer();
        b.iter(|| black_box(renderer.render(&mut buf, &config)));
    });
    c.bench_function("ray black hole small", |b| {
        let renderer = get_renderer();
        let config = RenderConfig {
            dimensions: Dimensions::new(10, 10),
            render_type: RenderType::BlackHole {
                vertical_fov_degrees: 20.0,
                background: image::open("space-background.jpg").unwrap(),
                pos: Vec3::new(0.0, 0.0, 10.0),
                rad: 9.0,
            },
        };
        let mut buf = config.dimensions.get_buffer();
        b.iter(|| black_box(renderer.render(&mut buf, &config)));
    });
}

criterion_group!(benches, renderer_benchmark);
criterion_main!(benches);
