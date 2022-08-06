use crate::{
    framework::{
        program_context::ProgramContext, render_context::RenderContext,
        source_context::SourceContext, uniform_context::UniformContext,
    },
    BlackHoleParams, ImageCache,
};

pub fn get_program(
    gl: &RenderContext,
    params: &BlackHoleParams,
    images: &ImageCache,
) -> ProgramContext {
    let mut text: Vec<&UniformContext> = Vec::new();
    let stars = UniformContext::new_from_allocated_ref(
        &images.stars_tex,
        "stars",
        images.stars_dim.0,
        images.stars_dim.1,
    );
    let constellations = UniformContext::new_from_allocated_ref(
        &images.constellations_tex,
        "constellations",
        images.constellations_dim.0,
        images.constellations_dim.1,
    );
    let galaxy = UniformContext::new_from_allocated_ref(
        &images.galaxy_tex,
        "galaxy",
        images.galaxy_dim.0,
        images.galaxy_dim.1,
    );
    let cache = UniformContext::new_from_allocated_ref(
        &images.ray_cache_tex,
        "cache",
        images.ray_cache_dim.0,
        images.ray_cache_dim.1,
    );
    let z_max_cache = UniformContext::new_from_allocated_ref(
        &images.max_z_tex,
        "z_max_cache",
        images.max_z_dim.0,
        images.max_z_dim.1,
    );
    let angle_cache = UniformContext::new_from_allocated_ref(
        &images.angle_cache_tex,
        "angle_cache",
        images.angle_cache_dim.0,
        images.angle_cache_dim.1,
    );
    let angle_z_max_cache = UniformContext::new_from_allocated_ref(
        &images.angle_min_z_tex,
        "angle_z_max_cache",
        images.angle_min_z_dim.0,
        images.angle_min_z_dim.1,
    );
    text.push(&cache);
    text.push(&z_max_cache);
    text.push(&angle_cache);
    text.push(&angle_z_max_cache);
    text.push(&stars);
    text.push(&galaxy);
    text.push(&constellations);

    let frag = SourceContext::new(include_str!("shaders/fragment/black_hole/complete.glsl"));
    gl.get_program(None, &frag, &text)
}
