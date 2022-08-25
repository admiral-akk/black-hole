use serde::{Deserialize, Serialize};

use super::{
    dimension_params::DimensionParams,
    path_sampler::PathSampler,
    ray_approximation::RayApproximation,
    texture::texture_2d::Texture2D,
    view_bound_sampler::{ViewBoundSampler, ViewType},
};

#[derive(Serialize, Deserialize)]
pub struct RayApproximationSampler {
    tex: Texture2D,
    distance: DimensionParams,
    angle: DimensionParams,
    view: DimensionParams,
    view_sampler: ViewBoundSampler,
}

impl RayApproximationSampler {
    pub fn sample(&self, dist: f32, view: f32) -> RayApproximation {
        let dist_01 = self.distance.val_to_01(dist);
        let (view_type, view_01) = self.view_sampler.get_view_01(dist, view);
        match view_type {
            ViewType::Far => {}
            _ => panic!("Near view!"),
        }
        return self.tex.get([dist_01, view_01]);
    }
}

impl RayApproximationSampler {
    pub fn generate(
        path_sampler: &PathSampler,
        distance: DimensionParams,
        angle: DimensionParams,
        view: DimensionParams,
        view_sampler: &ViewBoundSampler,
    ) -> Self {
        let dists = distance.generate_list();
        let dimensions = [path_sampler.far.len(), path_sampler.far[0].len()];
        let mut tex = Texture2D::new(dimensions);
        for (i, paths) in path_sampler.far.iter().enumerate() {
            for (j, path) in paths.iter().enumerate() {
                tex.insert(
                    [i, j],
                    RayApproximation::generate_optimal(path, dists[i], &angle),
                );
            }
        }
        Self {
            tex,
            distance,
            angle,
            view,
            view_sampler: view_sampler.clone(),
        }
    }
}
