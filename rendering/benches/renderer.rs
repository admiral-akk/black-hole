use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geometry::Vec3;
use rendering::{
    renderer::renderer::{RenderConfig, RenderType, Renderer},
    structs::dimensions::Dimensions,
};

fn get_renderer() -> Renderer {
    Renderer::new()
}

pub fn renderer_benchmark(c: &mut Criterion) {}

criterion_group!(benches, renderer_benchmark);
criterion_main!(benches);
