use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct AngleTestPoint {
    pub view_port_coord: f64,
    pub target_angle: f64,
    pub dist: f64,
    pub dist_at_angle: Option<f64>,
}
