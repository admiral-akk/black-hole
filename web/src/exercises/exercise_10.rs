use crate::{
    framework::{
        program_context::ProgramContext, render_context::RenderContext,
        source_context::SourceContext,
    },
    BlackHoleParams, ImageCache,
};

pub fn get_program(
    gl: &RenderContext,
    _params: &BlackHoleParams,
    images: &ImageCache,
) -> ProgramContext {
    let frag = SourceContext::new(include_str!("shaders/fragment/black_hole/complete.glsl"));
    gl.get_program(None, &frag, &images.textures)
}
