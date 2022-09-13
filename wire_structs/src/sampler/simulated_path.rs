use std::f32::consts::{FRAC_PI_2, PI, TAU};

use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::gpu::simulated_ray::SimulatedRay;

#[derive(Serialize, Deserialize)]
pub struct SimulatedPath {
    pub ray: SimulatedRay,
    pub dist: f32,
    pub view: f32,
}

impl SimulatedPath {
    pub fn min_dist(&self) -> f32 {
        match self
            .ray
            .angle_dist
            .iter()
            .filter(|d| **d > 0.)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
        {
            Some(v) => *v,
            None => 1.5,
        }
    }

    pub fn grazing_distance(&self) -> Option<f32> {
        let angle_dist = &self.ray.angle_dist;
        let i = match angle_dist
            .iter()
            .enumerate()
            .filter(|(_, d)| **d > 0.)
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        {
            Some((i, _)) => i,
            None => 0,
        };
        if i == 0 || i == angle_dist.len() - 1 {
            return None;
        }
        if angle_dist[i - 1] < angle_dist[i] {
            return None;
        }
        if angle_dist[i + 1] < angle_dist[i] {
            return None;
        }
        return Some(angle_dist[i]);
    }

    pub fn start_angle(&self, angles: &Vec<f32>) -> f32 {
        let d_1 = self.ray.angle_dist[1];
        if d_1 == 0. {
            return 0.;
        }
        -f32::atan2(angles[1].sin(), -d_1 * angles[1].cos() + self.dist)
    }

    pub fn projected_min_angle(&self, angles: &Vec<f32>) -> f32 {
        (self.final_angle(&angles) + self.start_angle(&angles)) / 2.
    }

    pub fn final_angle_point(&self, angles: &Vec<f32>) -> f32 {
        let final_index = self
            .ray
            .angle_dist
            .iter()
            .enumerate()
            .filter(|(i, dist)| **dist > 0.)
            .last();
        let final_index = match final_index {
            Some((i, _)) => i,
            None => 0,
        };
        angles[final_index]
    }

    pub fn final_angle(&self, angles: &Vec<f32>) -> f32 {
        let final_index = self
            .ray
            .angle_dist
            .iter()
            .enumerate()
            .filter(|(i, dist)| **dist > 0.)
            .last();
        let final_index = match final_index {
            Some((i, _)) => i,
            None => 0,
        };
        let final_dir = self.ray.final_dir;
        let mut final_angle = (f32::atan2(final_dir[1], final_dir[0]) + TAU + FRAC_PI_2) % TAU;
        let quarter_index = self.ray.angle_dist.len() / 4;
        let quarter_dist = self.ray.angle_dist[quarter_index];
        let final_dist = Vec2::from_array(self.ray.final_pos).length();

        if final_dist > 1.01 {
            if final_angle < FRAC_PI_2 {
                final_angle = final_angle + TAU;
            }
            return final_angle - FRAC_PI_2;
        } else {
            if final_angle < PI {
                final_angle = final_angle + TAU;
            }
            return 2. * TAU - final_angle;
        }

        // if it doesn't make it quarter way around,
        if quarter_dist == 0. {
            // hits black hole
            if final_angle >= PI {
                let final_angle = 2. * TAU - final_angle;
                return final_angle;
            } else {
                return final_angle;
            }
        } else {
            if final_dist > 1. {
                if final_angle < FRAC_PI_2 {
                    return final_angle + TAU;
                }
                return final_angle;
            } else {
                let final_angle = 2. * TAU - final_angle;
                return final_angle;
            }
        }
    }
}
