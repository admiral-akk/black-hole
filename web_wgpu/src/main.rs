use web_wgpu::run;

fn main() {
    //  generate_test_points();
    //generate_angle_texture(&distance_mapping, &z_mapping);
    //plot_angle_texture_stats(&distance_mapping, &z_mapping);
    pollster::block_on(run());
    //regenerate_black_hole_cache();
}
