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
    text.push(&images.ray_cache_tex);
    text.push(&images.max_z_tex);
    text.push(&images.angle_cache_tex);
    text.push(&images.angle_min_z_tex);
    text.push(&images.stars_tex);
    text.push(&images.galaxy_tex);
    text.push(&images.constellations_tex);

    let frag = SourceContext::new(include_str!("shaders/fragment/black_hole/complete.glsl"));
    gl.get_program(None, &frag, &text)
}
