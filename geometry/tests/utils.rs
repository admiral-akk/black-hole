#[cfg(test)]

mod tests {
    use geometry::{utils::to_phi_theta, DVec3};

    #[test]
    fn theta_0() {
        let v = DVec3::new(1.0, 0.0, 0.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, 0.0);

        let v = DVec3::new(1.0, 1.0, 0.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, 0.0);

        let v = DVec3::new(0.0, 1.0, 0.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, 0.0);

        let v = DVec3::new(-1.0, 1.0, 0.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, 0.0);

        let v = DVec3::new(-1.0, 0.0, 0.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, 0.0);

        let v = DVec3::new(-1.0, -1.0, 0.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, 0.0);

        let v = DVec3::new(0.0, -1.0, 0.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, 0.0);

        let v = DVec3::new(-1.0, 1.0, 0.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, 0.0);
    }

    #[test]
    fn theta_pi_4() {
        let v = DVec3::new(1.0, 0.0, 1.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(1.0, 1.0, 2.0_f64.sqrt());
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(0.0, 1.0, 1.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(-1.0, 1.0, 2.0_f64.sqrt());
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(-1.0, 0.0, 1.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(-1.0, -1.0, 2.0_f64.sqrt());
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(0.0, -1.0, 1.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(-1.0, 1.0, 2.0_f64.sqrt());
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, std::f64::consts::FRAC_PI_4);
    }

    #[test]
    fn theta_minus_pi_4() {
        let v = DVec3::new(1.0, 0.0, -1.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, -std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(1.0, 1.0, -2.0_f64.sqrt());
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, -std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(0.0, 1.0, -1.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, -std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(-1.0, 1.0, -2.0_f64.sqrt());
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, -std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(-1.0, 0.0, -1.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, -std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(-1.0, -1.0, -2.0_f64.sqrt());
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, -std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(0.0, -1.0, -1.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, -std::f64::consts::FRAC_PI_4);

        let v = DVec3::new(-1.0, 1.0, -2.0_f64.sqrt());
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, -std::f64::consts::FRAC_PI_4);
    }

    #[test]
    fn theta_pi_2() {
        let v = DVec3::new(0.0, 0.0, 1.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, std::f64::consts::FRAC_PI_2);
    }
    #[test]
    fn theta_minus_pi_2() {
        let v = DVec3::new(0.0, 0.0, -1.0);
        let (_, theta) = to_phi_theta(&v);
        assert_eq!(theta, -std::f64::consts::FRAC_PI_2);
    }

    #[test]
    fn phi_0() {
        let v = DVec3::new(1.0, 0.0, 0.0);
        let (phi, _) = to_phi_theta(&v);
        assert_eq!(phi, 0.0);

        let v = DVec3::new(1.0, 0.0, -1.0);
        let (phi, _) = to_phi_theta(&v);
        assert_eq!(phi, 0.0);

        let v = DVec3::new(1.0, 0.0, 1.0);
        let (phi, _) = to_phi_theta(&v);
        assert_eq!(phi, 0.0);
    }

    #[test]
    fn phi_pi_2() {
        let v = DVec3::new(0.0, 1.0, 0.0);
        let (phi, _) = to_phi_theta(&v);
        assert_eq!(phi, std::f64::consts::FRAC_PI_2);

        let v = DVec3::new(0.0, 1.0, -1.0);
        let (phi, _) = to_phi_theta(&v);
        assert_eq!(phi, std::f64::consts::FRAC_PI_2);

        let v = DVec3::new(0.0, 1.0, 1.0);
        let (phi, _) = to_phi_theta(&v);
        assert_eq!(phi, std::f64::consts::FRAC_PI_2);
    }

    #[test]
    fn phi_pi() {
        let v = DVec3::new(-1.0, 0.0, 0.0);
        let (phi, _) = to_phi_theta(&v);
        assert_eq!(phi, std::f64::consts::PI);

        let v = DVec3::new(-1.0, 0.0, -1.0);
        let (phi, _) = to_phi_theta(&v);
        assert_eq!(phi, std::f64::consts::PI);

        let v = DVec3::new(-1.0, 0.0, 1.0);
        let (phi, _) = to_phi_theta(&v);
        assert_eq!(phi, std::f64::consts::PI);
    }

    #[test]
    fn phi_3_pi_2() {
        let v = DVec3::new(0.0, -1.0, 0.0);
        let (phi, _) = to_phi_theta(&v);
        assert_eq!(phi, std::f64::consts::PI + std::f64::consts::FRAC_PI_2);

        let v = DVec3::new(0.0, -1.0, -1.0);
        let (phi, _) = to_phi_theta(&v);
        assert_eq!(phi, std::f64::consts::PI + std::f64::consts::FRAC_PI_2);

        let v = DVec3::new(0.0, -1.0, 1.0);
        let (phi, _) = to_phi_theta(&v);
        assert_eq!(phi, std::f64::consts::PI + std::f64::consts::FRAC_PI_2);
    }
}
