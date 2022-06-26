use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glam::DVec3;

use path_integration::BlackHole;
use rendering::{
    render::render,
    structs::{
        camera::Camera, dimensions::Dimensions, image_data::ImageData, observer::Observer,
        stars::Stars,
    },
};

pub fn render_benchmark(c: &mut Criterion) {
    c.bench_function("black hole r=1.0, 10x10 px", |b| {
        let dimensions = Dimensions::new(10, 10);

        let pos = -5.0 * DVec3::Z;
        let vertical_fov = 50.0;

        let background = image::open("starmap_2020_4k_gal.exr").unwrap();

        let radius = 1.0;

        let observer = Observer::new(pos, DVec3::Z, DVec3::Y, vertical_fov);
        let mut image_data = ImageData::new(dimensions.width, dimensions.height);
        let black_hole = BlackHole::new(radius, &pos, vertical_fov * std::f64::consts::PI / 180.0);
        let stars = Stars::new(background);
        b.iter(|| black_box(render(&mut image_data, &observer, &stars, &black_hole)));
    });
}

criterion_group!(benches, render_benchmark);
criterion_main!(benches);
