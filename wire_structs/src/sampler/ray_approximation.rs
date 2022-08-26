use std::{
    f32::consts::{FRAC_PI_2, PI, TAU},
    ops::Mul,
};

use serde::{Deserialize, Serialize};

use super::{dimension_params::DimensionParams, gpu::simulated_ray::SimulatedRay};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct RayApproximation {
    pub final_angle: f32,
    pub curve_dist: f32,
    pub center_curve_dist: f32,
    pub start_dist: f32,
}

impl Mul<RayApproximation> for f32 {
    type Output = RayApproximation;
    fn mul(self, rhs: RayApproximation) -> RayApproximation {
        RayApproximation {
            final_angle: self * rhs.final_angle,
            curve_dist: self * rhs.curve_dist,
            start_dist: self * rhs.start_dist,
            center_curve_dist: self * rhs.center_curve_dist,
        }
    }
}

pub fn measure_error(
    approx: &RayApproximation,
    true_ray: &SimulatedRay,
    angle: &DimensionParams,
) -> f32 {
    let mut err_sq = 0.;
    let mut count = 0;
    for (i, &angle) in angle.generate_list().iter().enumerate() {
        if true_ray.angle_dist[i] == 0. {
            break;
        }
        count += 1;
        let err = true_ray.angle_dist[i] - approx.get_dist(angle);
        err_sq += err * err;
    }
    err_sq.sqrt() / count as f32
}

pub fn measure_pre_theta_1_error(
    approx: &RayApproximation,
    true_ray: &SimulatedRay,
    angle: &DimensionParams,
) -> f32 {
    let mut err_sq = 0.;
    let mut count = 0;
    for (i, &angle) in angle.generate_list().iter().enumerate() {
        if true_ray.angle_dist[i] == 0. {
            break;
        }
        if angle > approx.theta_1() {
            break;
        }
        count += 1;
        let err = true_ray.angle_dist[i] - approx.get_dist(angle);
        err_sq += err * err;
    }
    err_sq.sqrt() / count as f32
}

impl RayApproximation {
    pub fn generate_average(rays: &[RayApproximation]) -> RayApproximation {
        let ray_count = rays.len() as f32;
        RayApproximation {
            final_angle: rays.iter().map(|r| r.final_angle).sum::<f32>() / ray_count,
            curve_dist: rays.iter().map(|r| r.curve_dist).sum::<f32>() / ray_count,
            start_dist: rays.iter().map(|r| r.start_dist).sum::<f32>() / ray_count,
            center_curve_dist: rays.iter().map(|r| r.center_curve_dist).sum::<f32>() / ray_count,
        }
    }

    pub fn new(final_angle: f32, curve_dist: f32, start_dist: f32, center_curve_dist: f32) -> Self {
        Self {
            curve_dist,
            final_angle,
            start_dist,
            center_curve_dist,
        }
    }

    pub fn generate_optimal(ray: &SimulatedRay, start_dist: f32, angle: &DimensionParams) -> Self {
        let mut dist_bounds = [0.0, start_dist];
        let final_angle = ray.final_angle();
        while dist_bounds[1] - dist_bounds[0] > 0.0001 {
            let delta = dist_bounds[1] - dist_bounds[0];
            let d_1 = delta / 3. + dist_bounds[0];
            let d_2 = 2. * delta / 3. + dist_bounds[0];
            let r_1 = RayApproximation::new(final_angle, d_1, start_dist, 1.);
            let r_2 = RayApproximation::new(final_angle, d_2, start_dist, 1.);

            let e_1 = measure_pre_theta_1_error(&r_1, &ray, angle);
            let e_2 = measure_pre_theta_1_error(&r_2, &ray, angle);
            if e_1 > e_2 {
                dist_bounds[0] = d_1;
            } else {
                dist_bounds[1] = d_2;
            }
        }
        let curve_dist = (dist_bounds[0] + dist_bounds[1]) / 2.;
        let r = RayApproximation::new(final_angle, curve_dist, start_dist, 1.);

        let mut center_curve_dist = curve_dist;

        let (theta_1, theta_2) = (r.theta_1(), r.theta_2());
        let has_data = angle
            .generate_list()
            .iter()
            .find(|a| **a > theta_1 && **a < theta_2)
            .is_some();
        if has_data {
            let mut curve_center_bounds = [0.0, start_dist];
            while curve_center_bounds[1] - curve_center_bounds[0] > 0.0001 {
                let c_delta = curve_center_bounds[1] - curve_center_bounds[0];
                let c_1 = c_delta / 3. + curve_center_bounds[0];
                let c_2 = 2. * c_delta / 3. + curve_center_bounds[0];
                let r_1 = RayApproximation::new(final_angle, curve_dist, start_dist, c_1);
                let r_2 = RayApproximation::new(final_angle, curve_dist, start_dist, c_2);

                let e_1 = measure_error(&r_1, &ray, angle);
                let e_2 = measure_error(&r_2, &ray, angle);

                if e_1 > e_2 {
                    curve_center_bounds[0] = c_1;
                } else {
                    curve_center_bounds[1] = c_2;
                }
            }
            center_curve_dist = (curve_center_bounds[0] + curve_center_bounds[1]) / 2.;
        }
        RayApproximation::new(final_angle, curve_dist, start_dist, curve_dist)
    }

    pub fn theta_1(&self) -> f32 {
        f32::acos(self.curve_dist / self.start_dist)
    }
    pub fn theta_2(&self) -> f32 {
        self.final_angle - FRAC_PI_2
    }

    pub fn get_dist(&self, angle: f32) -> f32 {
        let theta_1 = self.theta_1();
        let theta_2 = self.theta_2();
        if angle < theta_1 {
            let angle = theta_1 - angle;
            return self.curve_dist / angle.cos();
        } else if angle < theta_2 {
            let center = (theta_2 + theta_1) / 2.;
            let edge_weight = (center - angle).abs() / (center - theta_1);

            return edge_weight * self.center_curve_dist + (1. - edge_weight) * self.curve_dist;
        } else {
            let angle = angle - theta_2;
            return self.curve_dist / angle.cos();
        }
    }
}

#[cfg(test)]
mod test {
    use std::f32::consts::{FRAC_PI_2, PI};

    use super::RayApproximation;

    #[test]
    fn simple_approx() {
        let approx = RayApproximation::new(PI, 5., 10., 15.);

        assert!(
            (approx.get_dist(0.00001) - 10.).abs() < 0.001,
            "Approx dist at angle = 0. is: {}",
            approx.get_dist(0.00001)
        );

        let theta_1 = f32::acos(5. / 10.);

        assert!(
            (approx.get_dist(theta_1) - 5.).abs() < 0.001,
            "Approx dist near angle = theta_1 is: {}",
            approx.get_dist(theta_1)
        );

        let theta_2 = FRAC_PI_2;

        assert!(
            (approx.get_dist(theta_2) - 5.).abs() < 0.001,
            "Approx dist near angle = theta_2 is: {}",
            approx.get_dist(theta_1)
        );
        assert!(
            (approx.get_dist((theta_2 + theta_1) / 2.) - 5.).abs() < 0.001,
            "Approx dist near angle = (theta_2 + theta_1) / 2 is: {}",
            approx.get_dist((theta_2 + theta_1) / 2.)
        );

        assert!(
            approx.get_dist(PI - 0.00001) > 400.,
            "Approx dist near angle = PI is: {}",
            approx.get_dist(PI - 0.00001)
        );
    }
}
