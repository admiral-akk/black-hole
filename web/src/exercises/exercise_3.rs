use web_sys::WebGlTexture;

use crate::framework::{
    frame_buffer_context::FrameBufferContext, render_context::RenderContext,
    source_context::SourceContext, uniform_context::UniformContext,
};

pub fn exercise_3(gl: &RenderContext, fb: &mut FrameBufferContext) {
    let frag = SourceContext::new(include_str!("shaders/fragment/checkered.glsl"));
    let fb_texture = UniformContext::new_from_allocated_ref(&fb.backing_texture, "rtt_sampler");
    gl.draw(None, &frag, &[], Some(&fb.frame_buffer));

    let frag = SourceContext::new(include_str!("shaders/fragment/blur.glsl"));
    gl.draw(None, &frag, &[&fb_texture], None);
}
