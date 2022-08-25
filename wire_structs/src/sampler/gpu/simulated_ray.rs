use std::f32::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedRay {
    pub angle_dist: Vec<f32>,
    pub final_pos: [f32; 2],
    pub final_dir: [f32; 2],
}

impl SimulatedRay {
    pub fn final_angle(&self) -> f32 {
        (-f32::atan2(self.final_dir[0], self.final_dir[1]) + PI) % TAU
    }
}

#[cfg(test)]
mod test {
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    use super::SimulatedRay;

    #[test]
    fn final_dir() {
        let ray = SimulatedRay {
            angle_dist: Vec::new(),
            final_pos: [0., 0.],
            final_dir: [0., -1.],
        };

        assert!(
            (ray.final_angle()).abs() < 0.01,
            "final angle was: {}",
            ray.final_angle()
        );
        let ray = SimulatedRay {
            angle_dist: Vec::new(),
            final_pos: [0., 0.],
            final_dir: [1., -1.],
        };

        assert!(
            (ray.final_angle() - FRAC_PI_4).abs() < 0.01,
            "final angle was: {}",
            ray.final_angle()
        );
        let ray = SimulatedRay {
            angle_dist: Vec::new(),
            final_pos: [0., 0.],
            final_dir: [1., 0.],
        };

        assert!(
            (ray.final_angle() - FRAC_PI_2).abs() < 0.01,
            "final angle was: {}",
            ray.final_angle()
        );
        let ray = SimulatedRay {
            angle_dist: Vec::new(),
            final_pos: [0., 0.],
            final_dir: [1., 1.],
        };

        assert!(
            (ray.final_angle() - FRAC_PI_2 - FRAC_PI_4).abs() < 0.01,
            "final angle was: {}",
            ray.final_angle()
        );
        let ray = SimulatedRay {
            angle_dist: Vec::new(),
            final_pos: [0., 0.],
            final_dir: [0., 1.],
        };

        assert!(
            (ray.final_angle() - PI).abs() < 0.01,
            "final angle was: {}",
            ray.final_angle()
        );
        let ray = SimulatedRay {
            angle_dist: Vec::new(),
            final_pos: [0., 0.],
            final_dir: [-1., 1.],
        };

        assert!(
            (ray.final_angle() - PI - FRAC_PI_4).abs() < 0.01,
            "final angle was: {}",
            ray.final_angle()
        );
        let ray = SimulatedRay {
            angle_dist: Vec::new(),
            final_pos: [0., 0.],
            final_dir: [-1., 0.],
        };

        assert!(
            (ray.final_angle() - PI - FRAC_PI_2).abs() < 0.01,
            "final angle was: {}",
            ray.final_angle()
        );
        let ray = SimulatedRay {
            angle_dist: Vec::new(),
            final_pos: [0., 0.],
            final_dir: [-1., -1.],
        };

        assert!(
            (ray.final_angle() - PI - FRAC_PI_2 - FRAC_PI_4).abs() < 0.01,
            "final angle was: {}",
            ray.final_angle()
        );
    }
}
