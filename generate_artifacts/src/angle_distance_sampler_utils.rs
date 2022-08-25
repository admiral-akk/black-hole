use test_utils::plot_with_title;
use wire_structs::sampler::{
    angle_distance_sampler::AngleDistanceSampler, dimension_params::DimensionParams,
};

pub fn plot_paths(
    sampler: &AngleDistanceSampler,
    dist: &DimensionParams,
    view: &DimensionParams,
    angle: &DimensionParams,
) {
    let mut paths = Vec::new();
    for dist in dist.generate_list() {
        for view in view.generate_list() {
            let mut path = Vec::new();
            for angle in angle.generate_list() {
                let d = sampler.get_dist(dist, view, angle);
                if d < 4. && d > 1.5 {
                    path.push((angle.sin() * d, -angle.cos() * d));
                }
            }
            paths.push(path);
        }
        plot_with_title(
            &format!("Sampler Paths at dist = {}", dist),
            &format!("generate_artifacts/output/sampler_plot_{}.png", dist),
            &paths,
            ((-30., 30.), (-30., 30.)),
        );
    }
}
