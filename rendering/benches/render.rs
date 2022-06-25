use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geometry::DVec3;

use rendering::{
    render::render,
    structs::{black_hole::BlackHole, camera::Camera, dimensions::Dimensions, stars::Stars},
};

pub fn render_benchmark(c: &mut Criterion) {
    c.bench_function("black hole r=1.0, 10x10 px", |b| {
        let dimensions = Dimensions::new(10, 10);

        let pos = DVec3::ZERO;
        let dir = DVec3::Z;
        let vertical_fov = 50.0;

        let background = image::open("starmap_2020_4k_gal.exr").unwrap();

        let black_hole_pos = 5.0 * DVec3::Z;
        let radius = 1.0;

        let mut camera = Camera::new(dimensions, pos, dir, vertical_fov);
        let black_hole = BlackHole::new(black_hole_pos, radius);
        let stars = Stars::new(background);
        b.iter(|| black_box(render(&mut camera, &stars, &black_hole)));
    });
}

criterion_group!(benches, render_benchmark);
criterion_main!(benches);
