use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glam::DVec3;
use path_integration::{Field, RayCache};

fn generate_ray_cache(
    size: usize,
    field: &Field,
    camera_pos: &DVec3,
    fov_radians: f64,
) -> RayCache {
    RayCache::compute_new(size, field, camera_pos, fov_radians)
}

pub fn ray_cache_benchmark(c: &mut Criterion) {
    c.bench_function("ray cache, r = 1.5, d = 5.0, fov = 90, size = 100", |b| {
        let camera_pos = -5.0 * DVec3::Z;
        let radius = 1.5;
        let field = Field::new(radius, &camera_pos);
        let fov_radians = 90.0 * std::f64::consts::PI / 180.0;

        let size = 100;
        b.iter(|| black_box(generate_ray_cache(size, &field, &camera_pos, fov_radians)));
    });
    c.bench_function("ray cache, r = 1.5, d = 5.0, fov = 90, size = 10000", |b| {
        let camera_pos = -5.0 * DVec3::Z;
        let radius = 1.5;
        let field = Field::new(radius, &camera_pos);
        let fov_radians = 90.0 * std::f64::consts::PI / 180.0;

        let size = 10000;
        b.iter(|| black_box(generate_ray_cache(size, &field, &camera_pos, fov_radians)));
    });
}

criterion_group!(benches, ray_cache_benchmark);
criterion_main!(benches);
