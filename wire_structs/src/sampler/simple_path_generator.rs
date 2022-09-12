use super::{
    dimension_params::DimensionParams,
    gpu::gpu_state::{generate_particle, simulate_particles_groups},
    render_params::RenderParams,
    simulated_path::SimulatedPath,
};

pub fn generate_paths(
    dist: &DimensionParams,
    view: &DimensionParams,
    angle: &DimensionParams,
    render_params: &RenderParams,
) -> Vec<SimulatedPath> {
    let particles = generate_particle(dist, view, render_params);
    let rays = simulate_particles_groups(particles, angle, 40.);
    let mut ret = Vec::new();

    for (d_i, d) in dist.generate_list().iter().enumerate() {
        for (v_i, v) in view.generate_list().iter().enumerate() {
            let mut ray = rays[d_i][v_i].clone();
            ray.angle_dist = ray
                .angle_dist
                .into_iter()
                .map(|v| {
                    if f32::is_nan(v) {
                        return 0.;
                    }
                    v
                })
                .collect();
            ret.push(SimulatedPath {
                ray: ray,
                dist: *d,
                view: *v,
            });
        }
    }

    ret
}
