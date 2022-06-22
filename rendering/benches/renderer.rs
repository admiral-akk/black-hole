use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rendering::{
    renderer::{Renderer},
    structs::{
        config::Config,
        dimensions::{Dimensions},
    },
};

fn get_renderer() -> Renderer {
    Renderer::new(Config::new())
}

pub fn renderer_benchmark(c: &mut Criterion) {
    c.bench_function("renderer uv small", |b| {
        let dimensions = Dimensions::new(10, 10);
        let renderer = get_renderer();
        let mut buf = dimensions.get_buffer();
        b.iter(|| black_box(renderer.render(&mut buf, &dimensions)));
    });
}

criterion_group!(benches, renderer_benchmark);
criterion_main!(benches);
