use glam::DVec3;

use super::{
    response::Response,
    structs::{
        field::Field,
        step::{hit, step_particle},
    },
};
type TooClosePredicate = dyn Fn(Response) -> bool;

pub const RAY_START_DIR: DVec3 = DVec3::new(0.0, 0.0, -1.0);
// Takes in a ray and a parameterization of the black hole; returns the path taken.
// Also returns the final direction if it doesn't hit the black hole.
pub fn cast_ray_steps(
    camera_distance: f64,
    start_dir: DVec3,
    field: &Field,
    escape_radius: f64,
    max_distance: f64,
) -> (Vec<DVec3>, Option<DVec3>) {
    let mut particle = field.spawn_particle(camera_distance * RAY_START_DIR, start_dir);
    let mut distance = 0.0;
    let mut steps = Vec::new();
    while particle.p.length() < escape_radius && distance < max_distance {
        steps.push(particle.p);
        if hit(&particle, field) {
            return (steps, None);
        }
        let prev = particle.p;
        step_particle(&mut particle, field);
        distance += (particle.p - prev).length();
    }
    if distance >= max_distance {
        return (steps, None);
    }
    steps.push(particle.p);
    (steps, Some(particle.v))
}

// Takes in a ray and a parameterization of the black hole; returns the path taken.
// Also returns the final direction if it doesn't hit the black hole.
pub fn cast_ray_steps_response(z: f64, camera_distance: f64, black_hole_radius: f64) -> Response {
    let test = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
    let field = Field::new(black_hole_radius, camera_distance);
    let mut particle = field.spawn_particle(camera_distance * RAY_START_DIR, test);
    let mut distance = 0.0;
    let mut steps = Vec::new();
    let escape_radius = 5. * camera_distance;
    let max_distance = 20. * escape_radius;
    while particle.p.length() < escape_radius && distance < max_distance {
        steps.push(particle.p);
        if hit(&particle, &field) {
            return Response::new(steps, None);
        }
        let prev = particle.p;
        step_particle(&mut particle, &field);
        distance += (particle.p - prev).length();
    }
    if distance >= max_distance {
        return Response::new(steps, None);
    }
    steps.push(particle.p);
    Response::new(steps, Some(particle.v.normalize()))
}

pub fn find_optimal_z(
    camera_distance: f32,
    black_hole_radius: f32,
    epsilon: f64,
    z_bounds: (f64, f64),
    is_too_close: &TooClosePredicate,
) -> (f64, f64) {
    let mut z_bounds = z_bounds;
    while z_bounds.1 - z_bounds.0 > epsilon {
        let z = 0.5 * (z_bounds.0 + z_bounds.1);
        let response = cast_ray_steps_response(z, camera_distance as f64, black_hole_radius as f64);
        if is_too_close(response) {
            // too close
            z_bounds.1 = z;
        } else {
            z_bounds.0 = z;
        }
    }
    z_bounds
}

fn find_bound_with_grazing_distance(
    camera_distance: f32,
    field: &Field,
    epsilon: f64,
    max_distance: f64,
    grazing_distance: f64,
) -> f64 {
    let mut far_z = -1.0;
    let mut close_z = find_bound(camera_distance, field, epsilon, max_distance);
    while close_z - far_z > epsilon {
        let z = 0.5 * (close_z + far_z);
        let test = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
        let (path, _) = cast_ray_steps(
            camera_distance as f64,
            test,
            field,
            max_distance,
            10.0 * max_distance,
        );
        let min_dis = path.iter().map(|v| v.length()).reduce(f64::min).unwrap();
        if min_dis < grazing_distance {
            // too close
            close_z = test.z;
        } else {
            far_z = test.z;
        }
    }
    far_z
}

fn find_bound(camera_distance: f32, field: &Field, epsilon: f64, max_distance: f64) -> f64 {
    let (mut miss_z, mut hit_z) = (-1.0, 1.0);
    while hit_z - miss_z > epsilon {
        let z = 0.5 * (hit_z + miss_z);
        let test = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
        let final_dir = cast_ray_steps(
            camera_distance as f64,
            test,
            field,
            max_distance,
            10.0 * max_distance,
        )
        .1;
        if final_dir.is_none() {
            // hit the black hole
            hit_z = test.z;
        } else {
            miss_z = test.z;
        }
    }
    miss_z
}
