use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glam::Vec3;
use rendering::structs::ray_cache::RayCache;

fn generate_ray_cache(size: usize, radius: f32, camera_distance: f32) -> RayCache {
    RayCache::compute_new(size, radius, camera_distance)
}

pub fn ray_cache_benchmark(c: &mut Criterion) {
    c.bench_function("ray cache, r = 1.5, d = 5.0, fov = 90, size = 100", |b| {
        let camera_pos = -5.0 * Vec3::Z;
        let radius = 1.5;

        let size = 100;
        b.iter(|| black_box(generate_ray_cache(size, radius, camera_pos.length())));
    });
    c.bench_function("ray cache, r = 1.5, d = 5.0, fov = 90, size = 10000", |b| {
        let camera_pos = -5.0 * Vec3::Z;
        let radius = 1.5;

        let size = 10000;
        b.iter(|| black_box(generate_ray_cache(size, radius, camera_pos.length())));
    });
}

criterion_group!(benches, ray_cache_benchmark);
criterion_main!(benches);
