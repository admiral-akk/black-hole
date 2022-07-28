use web_sys::WebGlTexture;

use crate::framework::{
    render_context::RenderContext, source_context::SourceContext, uniform_context::UniformContext,
};

pub fn exercise_1(gl: &RenderContext, cm: &mut WebGlTexture) {
    let frag = SourceContext::new(include_str!("shaders/fragment/1_color_map.glsl"));
    let cm_context = UniformContext::new_from_allocated_ref(&cm, "u_palette", 256, 1);
    gl.draw(None, &frag, &[&cm_context], None);
}
