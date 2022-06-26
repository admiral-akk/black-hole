use criterion::{black_box, criterion_group, criterion_main, Criterion};
use glam::DVec3;
use path_integration::{cast_ray_steps, Field, Ray};

// A full 10x10 image
fn cast_ray_steps_repeated(ray: &Ray, field: &Field, pixel_count: u32) {
    for _i in 0..pixel_count {
        black_box(cast_ray_steps(&ray, &field, 40.0));
    }
}

pub fn path_benchmark(c: &mut Criterion) {
    c.bench_function("black hole r=0.0, 10x10 px", |b| {
        let ray = Ray::new(-5.0 * DVec3::Z, DVec3::Z);
        let field = Field::zero_new(0.0, &ray.pos);
        b.iter(|| black_box(cast_ray_steps_repeated(&ray, &field, 100)));
    });
    c.bench_function("black hole r=1.0, miss, 10x10 px", |b| {
        let ray = Ray::new(-5.0 * DVec3::Z, DVec3::X);
        let field = Field::zero_new(1.0, &ray.pos);
        b.iter(|| black_box(cast_ray_steps_repeated(&ray, &field, 100)));
    });
    c.bench_function("black hole r=1.0, near, 10x10 px", |b| {
        let ray = Ray::new(-5.0 * DVec3::Z, DVec3::X + DVec3::Z);
        let ray = Ray::new(DVec3::ZERO, DVec3::Z + DVec3::X);
        let field = Field::zero_new(1.0, &ray.pos);
        b.iter(|| black_box(cast_ray_steps_repeated(&ray, &field, 100)));
    });
    c.bench_function("black hole r=1.0, hit, 10x10 px", |b| {
        let ray = Ray::new(-5.0 * DVec3::Z, DVec3::Z);
        let field = Field::zero_new(1.0, &ray.pos);
        b.iter(|| black_box(cast_ray_steps_repeated(&ray, &field, 100)));
    });
    c.bench_function("black hole r=0.0, 100x100 px", |b| {
        let ray = Ray::new(-5.0 * DVec3::Z, DVec3::Z);
        let field = Field::zero_new(0.0, &ray.pos);
        b.iter(|| black_box(cast_ray_steps_repeated(&ray, &field, 10000)));
    });
    c.bench_function("black hole r=1.0, miss, 100x100 px", |b| {
        let ray = Ray::new(-5.0 * DVec3::Z, DVec3::X);
        let field = Field::zero_new(1.0, &ray.pos);
        b.iter(|| black_box(cast_ray_steps_repeated(&ray, &field, 10000)));
    });
    c.bench_function("black hole r=1.0, near, 100x100 px", |b| {
        let ray = Ray::new(-5.0 * DVec3::Z, DVec3::X + DVec3::Z);
        let field = Field::zero_new(1.0, &ray.pos);
        b.iter(|| black_box(cast_ray_steps_repeated(&ray, &field, 10000)));
    });
    c.bench_function("black hole r=1.0, hit, 100x100 px", |b| {
        let ray = Ray::new(-5.0 * DVec3::Z, DVec3::Z);
        let field = Field::zero_new(1.0, &ray.pos);
        b.iter(|| black_box(cast_ray_steps_repeated(&ray, &field, 10000)));
    });
}

criterion_group!(benches, path_benchmark);
criterion_main!(benches);
