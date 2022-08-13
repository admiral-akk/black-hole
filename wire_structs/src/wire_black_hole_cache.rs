pub struct WireBlackHoleCache {
    pub distance_bounds: (f32, f32),
    pub black_hole_radius: f32,
    pub direction_cache: WireDirectionCache,
}
pub struct WireDirectionCache {
    pub cache_size: (usize, usize),
    pub distance_bounds: (f32, f32),
    pub black_hole_radius: f32,
    pub distance_angle_to_z_to_distance: Vec<WireFixedDistanceDirectionCache>,
}

pub struct WireFixedDistanceDirectionCache {
    pub z_bounds: (f32, f32),
    pub z_to_final_dir: Vec<(f32, f32)>,
}

pub struct WireDistanceCache {
    pub cache_size: (usize, usize, usize),
    pub disc_bounds: (f32, f32),
    pub min_angle: f32,
    pub distance_to_angle_to_z_to_distance: Vec<WireFixedDistanceDistanceCache>,
}
pub struct WireFixedDistanceDistanceCache {
    pub min_z: f32,
    pub angle_to_z_to_distance: Vec<WireFixedDistanceFixedAngleDistanceCache>,
}
pub struct WireFixedDistanceFixedAngleDistanceCache {
    pub z_bounds: (f32, f32),
    pub z_to_distance: Vec<f32>,
}
