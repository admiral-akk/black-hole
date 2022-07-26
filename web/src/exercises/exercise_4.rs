

use crate::framework::{
    frame_buffer_context::FrameBufferContext, render_context::RenderContext,
    source_context::SourceContext, uniform_context::UniformContext,
};

const RENDER_TEXTURE_DEFAULT: &str = include_str!("shaders/fragment/render_texture.glsl");

pub fn exercise_4(
    gl: &RenderContext,
    fb1: &mut FrameBufferContext,
    fb2: &mut FrameBufferContext,
    kernel: &mut Vec<f32>,
) {
    let fb_texture = UniformContext::new_from_allocated_ref(&fb1.backing_texture, "rtt_sampler");
    let fb_texture2 = UniformContext::new_from_allocated_ref(&fb2.backing_texture, "rtt_sampler");
    let kernel_weights = UniformContext::array_f32(&kernel, "w");

    let mut frag = SourceContext::new(include_str!("shaders/fragment/checkered.glsl"));
    gl.draw(None, &frag, &[], Some(&fb1.frame_buffer));

    for _ in 0..10 {
        frag = SourceContext::new(include_str!("shaders/fragment/gaussian_blur.glsl"));
        frag.add_parameter("HORIZONTAL", "TRUE");
        frag.add_parameter("K", &kernel.len().to_string());
        gl.draw(
            None,
            &frag,
            &[&fb_texture, &kernel_weights],
            Some(&fb2.frame_buffer),
        );

        frag = SourceContext::new(include_str!("shaders/fragment/gaussian_blur.glsl"));
        frag.add_parameter("K", &kernel.len().to_string());
        frag.add_parameter("VERTICAL", "TRUE");
        gl.draw(
            None,
            &frag,
            &[&fb_texture2, &kernel_weights],
            Some(&fb1.frame_buffer),
        );
    }
    frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
    gl.draw(None, &frag, &[&fb_texture], None);
}
