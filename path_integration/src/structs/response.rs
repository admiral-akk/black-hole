use std::f64::consts::TAU;

use glam::DVec3;

use super::utils::PolarAngle;

pub struct Response {
    pub path: Vec<DVec3>,
    pub final_dir: Option<DVec3>,
}

impl Response {
    pub fn new(path: Vec<DVec3>, final_dir: Option<DVec3>) -> Self {
        Response { path, final_dir }
    }
}

// helper properties
impl Response {
    pub fn hits_black_hole(&self) -> bool {
        self.final_dir.is_none()
    }

    pub fn reaches_angle(&self, target_angle: f64) -> bool {
        AnglePath::new(&self.path).get_max_angle() >= target_angle
    }

    pub fn get_angle_dist(&self) -> AnglePath {
        AnglePath::new(&self.path)
    }
}

pub struct AnglePath {
    angle_dist: Vec<AngleDist>,
}

impl AnglePath {
    pub fn new(path: &Vec<DVec3>) -> Self {
        let mut angle_dist: Vec<AngleDist> = path
            .iter()
            .map(|pos| AngleDist {
                angle: pos.get_angle(),
                distance: pos.length(),
            })
            .collect();
        let mut offset = 0.0;
        for i in 1..angle_dist.len() {
            if angle_dist[i - 1].angle > angle_dist[i].angle + offset {
                offset += TAU;
            }
            angle_dist[i].angle += offset;
        }

        AnglePath { angle_dist }
    }

    pub fn get_max_angle(&self) -> f64 {
        self.angle_dist.last().unwrap().angle
    }

    pub fn get_final_dist(&self) -> f64 {
        self.angle_dist.last().unwrap().distance
    }

    pub fn get_dist(&self, angle: f64) -> Option<f64> {
        if angle < 0. {
            return None;
        }
        if angle > self.angle_dist.last().unwrap().angle {
            return None;
            let left = &self.angle_dist[self.angle_dist.len() - 2];
            let right = &self.angle_dist[self.angle_dist.len() - 1];

            let dx = right.angle - left.angle;
            let dy = right.distance - left.distance;
            let t = angle - right.angle;
            return Some(t * dy / dx + right.distance);
        }

        let location = self.angle_dist.binary_search_by(|v| {
            v.angle
                .partial_cmp(&angle)
                .expect("Couldn't compare values")
        });

        let index = match location {
            Ok(i) => i,
            Err(i) => i,
        };
        let left = &self.angle_dist[index];

        if index == self.angle_dist.len() - 1 {
            return Some(left.distance);
        }

        let right = &self.angle_dist[index + 1];
        let t = (angle - left.angle) / (right.angle - left.angle);

        return Some(t * right.distance + (1. - t) * left.distance);
    }
}

pub struct AngleDist {
    pub angle: f64,
    pub distance: f64,
}
