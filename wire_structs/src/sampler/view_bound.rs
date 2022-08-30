use serde::{Deserialize, Serialize};

use super::approximation_function::ApproximationFunction;

#[derive(Serialize, Deserialize)]
pub struct ViewBound {
    pub dist_to_view_bound: Vec<f32>,
}

impl ViewBound {
    pub fn generate(paths: &Vec<ApproximationFunction>) -> Self {
        let mut dist_to_view_bound = vec![0.; 1];
        let mut new_dist = true;
        for i in 1..paths.len() {
            if paths[i].initial_dist != paths[i - 1].initial_dist {
                new_dist = true;
                dist_to_view_bound.push(paths[i].view);
            }
            if !new_dist {
                continue;
            }
            if paths[i].min_distance < 1.5 {
                let len = dist_to_view_bound.len();
                dist_to_view_bound[len - 1] = paths[i].view;
            }
        }

        ViewBound { dist_to_view_bound }
    }
}
