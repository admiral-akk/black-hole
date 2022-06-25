use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geometry::{DVec3, Ray, Vec3};
use path_integration::{cast_ray_steps, Field};

// A full 10x10 image
fn cast_ray_steps_repeated(ray: &Ray, field: &Field) {
    for i in 0..100 {
        black_box(cast_ray_steps(&ray, &field, 40.0));
    }
}

pub fn path_benchmark(c: &mut Criterion) {
    c.bench_function("black hole f=0.0", |b| {
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);
        let field = Field::new(5.0 * DVec3::Z, 0.0);
        b.iter(|| black_box(cast_ray_steps_repeated(&ray, &field)));
    });
    c.bench_function("black hole f=1.0, miss", |b| {
        let ray = Ray::new(Vec3::ZERO, Vec3::X);
        let field = Field::new(5.0 * DVec3::Z, 0.0);
        b.iter(|| black_box(cast_ray_steps_repeated(&ray, &field)));
    });
    c.bench_function("black hole f=1.0, near", |b| {
        let ray = Ray::new(Vec3::ZERO, Vec3::Z + Vec3::X);
        let field = Field::new(5.0 * DVec3::Z, 0.0);
        b.iter(|| black_box(cast_ray_steps_repeated(&ray, &field)));
    });
    c.bench_function("black hole f=1.0, hit", |b| {
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);
        let field = Field::new(5.0 * DVec3::Z, 0.0);
        b.iter(|| black_box(cast_ray_steps_repeated(&ray, &field)));
    });
}

criterion_group!(benches, path_benchmark);
criterion_main!(benches);
