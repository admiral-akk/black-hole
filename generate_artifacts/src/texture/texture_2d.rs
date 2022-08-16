use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::path_integration2::{
    path::cast_ray_steps_response,
    response::{Response, ToAngle},
};

#[derive(Serialize, Deserialize)]
pub struct Texture2D {
    dimensions: (usize, usize),
    val: Vec<Vec<f32>>,
    z_bounds: Vec<(f32, f32)>,
}

struct TextureIndex {
    pub left_index: usize,
    pub left_weight: f32,
}

fn get_texture_index(i_01: f32, vec_len: usize) -> TextureIndex {
    let float_index = i_01 * (vec_len - 1) as f32;
    let left_index = (float_index as usize).clamp(0, vec_len - 2);
    let left_weight = 1. - (float_index - left_index as f32);
    TextureIndex {
        left_index,
        left_weight,
    }
}

impl Texture2D {
    pub fn initialize(dimensions: (usize, usize)) -> Self {
        let mut val = Vec::new();
        let mut z_bounds = Vec::new();
        for _ in 0..dimensions.0 {
            z_bounds.push((f32::INFINITY, f32::NEG_INFINITY));
            let mut vec = Vec::new();
            for _ in 0..dimensions.1 {
                vec.push(0.);
            }
            val.push(vec);
        }
        Texture2D {
            dimensions,
            val,
            z_bounds,
        }
    }

    pub fn get_z_bounds(&self, d_01: f32) -> (f32, f32) {
        let index = get_texture_index(d_01, self.dimensions.0);
        let left = self.z_bounds[index.left_index];
        let right = self.z_bounds[index.left_index + 1];
        let t = index.left_weight;
        (
            right.0 * (1. - t) + t * left.0,
            right.1 * (1. - t) + t * left.1,
        )
    }

    pub fn fetch(&self, d_01: f32, z_01: f32) -> f32 {
        let d_index = get_texture_index(d_01, self.dimensions.0);
        let z_index = get_texture_index(z_01, self.dimensions.1);
        let (v_00, v_01, v_10, v_11) = (
            self.val[d_index.left_index][z_index.left_index],
            self.val[d_index.left_index][z_index.left_index + 1],
            self.val[d_index.left_index + 1][z_index.left_index],
            self.val[d_index.left_index + 1][z_index.left_index + 1],
        );
        println!("Fetch values: {:?}", (v_00, v_01, v_10, v_11));
        d_index.left_weight * (z_index.left_weight * v_00 + (1. - z_index.left_weight) * v_01)
            + (1. - d_index.left_weight)
                * (z_index.left_weight * v_10 + (1. - z_index.left_weight) * v_11)
    }

    pub fn insert(&mut self, distance_index: usize, z_index: usize, z: f32, val: f32) {
        self.val[distance_index][z_index] = val;
        self.z_bounds[distance_index] = (
            f32::min(z, self.z_bounds[distance_index].0),
            f32::max(z, self.z_bounds[distance_index].1),
        );
    }
}
pub struct IndexMapping {
    pub i_01_to_dist_01: fn(f32) -> f32,
    pub dist_01_to_i_01: fn(f32) -> f32,
}

impl IndexMapping {
    pub fn i_to_val(&self, i: usize, len: usize, bounds: (f32, f32)) -> f32 {
        let i_01 = i as f32 / (len - 1) as f32;
        let dist_01 = (self.i_01_to_dist_01)(i_01);
        (bounds.1 - bounds.0) * dist_01 + bounds.0
    }
    pub fn val_to_i_01(&self, v: f32, bounds: (f32, f32)) -> f32 {
        let dist_01 = (v - bounds.0) / (bounds.1 - bounds.0);
        (self.dist_01_to_i_01)(dist_01)
    }
}

const MAX_ANGLE: f64 = TAU;
use crate::path_integration2::path::find_optimal_z;
fn find_closest_z(camera_distance: f32, black_hole_radius: f32) -> f64 {
    let too_close = |r: Response| r.hits_black_hole() || r.get_final_angle().unwrap() > MAX_ANGLE;
    find_optimal_z(camera_distance, black_hole_radius, (-1., 1.), &too_close).0
}

const ANGLE_EPSILON: f64 = 0.1 * TAU / 360.;
// use this find z values where we don't have to apply anti-aliasing because the path is basically straight.
fn find_minimum_pertubation_z(camera_distance: f32, black_hole_radius: f32, max_z: f32) -> f64 {
    let too_close = |r: Response| {
        let initial_dir = (r.path[1] - r.path[0]).get_angle();
        r.hits_black_hole() || (r.get_final_angle().unwrap() - initial_dir) > ANGLE_EPSILON
    };
    find_optimal_z(
        camera_distance,
        black_hole_radius,
        (-1., max_z as f64),
        &too_close,
    )
    .1
}

pub fn generate_final_angle_texture(
    dimensions: (usize, usize),
    dist_bounds: (f32, f32),
    black_hole_radius: f32,
    distance_to_d_01: &IndexMapping,
    z_to_z_01: &IndexMapping,
) -> Texture2D {
    let mut tex = Texture2D::initialize(dimensions);
    for d_index in 0..dimensions.0 {
        let dist = distance_to_d_01.i_to_val(d_index, dimensions.0, dist_bounds);
        let max_z = find_closest_z(dist, black_hole_radius) as f32;
        let min_z = find_minimum_pertubation_z(dist, black_hole_radius, max_z) as f32;
        let z_bounds = (min_z, max_z);
        for z_index in 0..dimensions.1 {
            let z = z_to_z_01.i_to_val(z_index, dimensions.1, z_bounds);
            println!(
                "Generating dist: {}, z: {}, z_bounds: {:?}",
                dist, z, z_bounds
            );
            let angle = cast_ray_steps_response(z as f64, dist as f64, black_hole_radius as f64)
                .get_final_angle()
                .unwrap();
            tex.insert(d_index, z_index, z, angle as f32);
        }
    }
    tex
}

pub fn sample_final_angle_texture(
    tex: &Texture2D,
    distance_to_d_01: &IndexMapping,
    z_to_z_01: &IndexMapping,
    distance: f32,
    z: f32,
    distance_bounds: (f32, f32),
) -> f32 {
    let d_01 = distance_to_d_01.val_to_i_01(distance, distance_bounds);
    let z_bounds = tex.get_z_bounds(d_01);
    let z_01 = z_to_z_01.val_to_i_01(z, z_bounds);
    tex.fetch(d_01, z_01)
}
