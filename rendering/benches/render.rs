use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glam::Vec3;

use rendering::{
    render::render,
    structs::{
        dimensions::Dimensions, image_data::ImageData, observer::Observer, ray_cache::RayCache,
        stars::Stars,
    },
};

pub fn render_benchmark(c: &mut Criterion) {
    c.bench_function("black hole r=1.0, 10x10 px", |b| {
        let dimensions = Dimensions::new(10, 10);

        let pos = -5.0 * Vec3::Z;
        let vertical_fov = 50.0;

        let background = image::open("starmap_2020_4k_gal.exr").unwrap();

        let radius = 1.0;

        let observer = Observer::new(pos, -pos, Vec3::Y, vertical_fov);
        let mut image_data = ImageData::new(dimensions.width, dimensions.height);
        let ray_cache = RayCache::compute_new(10000, radius, pos.length());
        let stars = Stars::new(background);
        b.iter(|| black_box(render(&mut image_data, &observer, &stars, &ray_cache)));
    });
}

criterion_group!(benches, render_benchmark);
criterion_main!(benches);
