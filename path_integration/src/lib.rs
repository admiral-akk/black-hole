use glam::{DVec3, Vec3};
use structs::{particle::Particle, response::Response};

pub mod cache;
mod structs;
pub use structs::ray::Ray;

pub use structs::field::Field;
use structs::step::{hit, step_particle};
type TooClosePredicate = dyn Fn(Response) -> bool;
// Takes in a ray and a parameterization of the black hole; returns the path taken.
// Also returns the final direction if it doesn't hit the black hole.
pub fn cast_ray_steps(
    ray: &Ray,
    field: &Field,
    escape_radius: f64,
    max_distance: f64,
) -> (Vec<DVec3>, Option<DVec3>) {
    let mut particle = Particle::new(ray, field);
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

fn find_z_bounds_for_angle(
    camera_distance: f64,
    black_hole_radius: f64,
    epsilon: f64,
    distance_bounds: (f64, f64),
    target_angle: f64,
) -> (f64, f64) {
    let bound_predicate = |r: Response| r.hits_black_hole();
    let valid_z = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        epsilon,
        (-1., 1.),
        &bound_predicate,
    );

    let is_too_close = move |r: Response| {
        let dist = r.get_angle_dist().get_dist(target_angle);
        return !dist.is_none() && dist.unwrap() <= distance_bounds.1;
    };
    let lower_1 = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        epsilon,
        (-1., valid_z.0),
        &is_too_close,
    );
    let lower_test_1 =
        cast_ray_steps_response(lower_1.1, camera_distance as f64, black_hole_radius as f64)
            .get_angle_dist()
            .get_dist(target_angle)
            .unwrap_or(100.);
    let is_too_close = move |r: Response| {
        let dist = r.get_angle_dist().get_dist(target_angle);
        dist.is_none() || dist.unwrap() <= distance_bounds.1
    };
    let lower_2 = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        epsilon,
        (valid_z.0, 1.0),
        &is_too_close,
    );
    let lower_test_2 =
        cast_ray_steps_response(lower_2.1, camera_distance as f64, black_hole_radius as f64)
            .get_angle_dist()
            .get_dist(target_angle)
            .unwrap_or(100.);

    let mut lower = lower_1.1;
    if (lower_test_2 - distance_bounds.1).abs() < (lower_test_1 - distance_bounds.1).abs() {
        lower = lower_2.1;
    }

    let _is_too_close = move |r: Response| {
        let angle_d = r.get_angle_dist();
        if angle_d.get_max_angle() < target_angle {
            return true;
        }
        if angle_d.get_dist(target_angle).unwrap() < distance_bounds.0 {
            return true;
        }
        false
    };

    let is_too_close = move |r: Response| {
        let dist = r.get_angle_dist().get_dist(target_angle);
        return !dist.is_none() && dist.unwrap() <= distance_bounds.0;
    };
    let upper_1 = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        epsilon,
        (-1., valid_z.0),
        &is_too_close,
    );
    let upper_test_1 =
        cast_ray_steps_response(upper_1.1, camera_distance as f64, black_hole_radius as f64)
            .get_angle_dist()
            .get_dist(target_angle)
            .unwrap_or(100.);
    let is_too_close = move |r: Response| {
        let dist = r.get_angle_dist().get_dist(target_angle);
        dist.is_none() || dist.unwrap() < distance_bounds.0
    };
    let upper_2 = find_optimal_z(
        camera_distance as f32,
        black_hole_radius as f32,
        epsilon,
        (valid_z.0, 1.0),
        &is_too_close,
    );
    let upper_test_2 =
        cast_ray_steps_response(upper_2.1, camera_distance as f64, black_hole_radius as f64)
            .get_angle_dist()
            .get_dist(target_angle)
            .unwrap_or(100.);

    let mut upper = upper_1.0;
    if (upper_test_2 - distance_bounds.0).abs() < (upper_test_1 - distance_bounds.0).abs() {
        upper = upper_2.0;
    }

    (lower, upper)
}

// Takes in a ray and a parameterization of the black hole; returns the path taken.
// Also returns the final direction if it doesn't hit the black hole.
pub fn cast_ray_steps_response(z: f64, camera_distance: f64, black_hole_radius: f64) -> Response {
    let test = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
    let field = Field::new(black_hole_radius, camera_distance);
    let ray = Ray::new((camera_distance as f32 * RAY_START_DIR).as_dvec3(), test);
    let mut particle = Particle::new(&ray, &field);
    let mut distance = 0.0;
    let mut steps = Vec::new();
    let escape_radius = 2. * camera_distance;
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
    Response::new(steps, Some(particle.v))
}

pub const RAY_START_DIR: Vec3 = Vec3::new(0.0, 0.0, -1.0);
fn find_optimal_z(
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
    camera_pos: &Vec3,
    field: &Field,
    epsilon: f64,
    max_distance: f64,
    grazing_distance: f64,
) -> f64 {
    let mut far_z = -1.0;
    let mut close_z = find_bound(camera_pos, field, epsilon, max_distance);
    while close_z - far_z > epsilon {
        let z = 0.5 * (close_z + far_z);
        let test = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
        let ray = Ray::new(camera_pos.as_dvec3(), test);
        let (path, _) = cast_ray_steps(&ray, field, max_distance, 10.0 * max_distance);
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

fn find_bound(camera_pos: &Vec3, field: &Field, epsilon: f64, max_distance: f64) -> f64 {
    let (mut miss_z, mut hit_z) = (-1.0, 1.0);
    while hit_z - miss_z > epsilon {
        let z = 0.5 * (hit_z + miss_z);
        let test = DVec3::new((1.0 - z * z).sqrt(), 0.0, z);
        let ray = Ray::new(camera_pos.as_dvec3(), test);
        let final_dir = cast_ray_steps(&ray, field, max_distance, 10.0 * max_distance).1;
        if final_dir.is_none() {
            // hit the black hole
            hit_z = test.z;
        } else {
            miss_z = test.z;
        }
    }
    miss_z
}
