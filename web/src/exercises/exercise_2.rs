use web_sys::WebGlTexture;

use crate::framework::{
    render_context::RenderContext, source_context::SourceContext, uniform_context::UniformContext,
};

pub fn exercise_2(gl: &RenderContext, cm1: &mut WebGlTexture, cm2: &mut WebGlTexture) {
    let frag = SourceContext::new(include_str!("shaders/fragment/2_color_map.glsl"));
    let cm_context1 = UniformContext::new_from_allocated_ref(&cm1, "u_palette_1");
    let cm_context2 = UniformContext::new_from_allocated_ref(&cm2, "u_palette_2");
    gl.draw(None, &frag, &[&cm_context1, &cm_context2], None);
}
