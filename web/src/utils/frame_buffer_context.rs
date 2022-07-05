use web_sys::{WebGl2RenderingContext, WebGlFramebuffer, WebGlTexture};

use super::texture_utils::create_texture;

pub struct FrameBufferContext {
    pub frame_buffer: WebGlFramebuffer,
}

impl FrameBufferContext {
    pub fn new(
        gl: &WebGl2RenderingContext,
        width: i32,
        height: i32,
    ) -> (FrameBufferContext, WebGlTexture) {
        let backing_texture = create_texture(gl, width, height).unwrap();
        let frame_buffer = gl
            .create_framebuffer()
            .expect("Couldn't create frame buffer");
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&frame_buffer));
        gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&backing_texture),
            0,
        );
        (FrameBufferContext { frame_buffer }, backing_texture)
    }
}
