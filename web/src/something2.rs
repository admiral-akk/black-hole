use web_sys::{WebGl2RenderingContext, WebGlFramebuffer, WebGlTexture};

use crate::utils::{
    program_context::ProgramContext, render_context::RenderContext, texture::Texture,
};

pub fn draw(
    vertex_source: &str,
    fragment_source: &str,
    textures: &[Texture],
    out_buffer: Option<WebGlFramebuffer>,
) {
    let render_context = RenderContext::new(500, 500);
    let gl = &render_context.gl;

    let mut program_context = ProgramContext::new(gl, vertex_source, fragment_source);
    gl.use_program(Some(&program_context.program));

    for i in 0..textures.len() {}
}

fn create_texture(gl: &WebGl2RenderingContext, width: i32, height: i32) -> Option<WebGlTexture> {
    let tex = gl.create_texture();
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, tex.as_ref());
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        WebGl2RenderingContext::TEXTURE_2D,
        0,
        WebGl2RenderingContext::RGBA as i32,
        width,
        height,
        0,
        WebGl2RenderingContext::RGBA,
        WebGl2RenderingContext::UNSIGNED_BYTE,
        None,
    )
    .expect("Failed to generate texture for frame buffer");
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_S,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_T,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    tex
}

fn create_texture_and_frame_buffer(
    gl: &WebGl2RenderingContext,
    width: i32,
    height: i32,
) -> (WebGlTexture, WebGlFramebuffer) {
    let tex = create_texture(gl, width, height);
    let frame_buffer = gl
        .create_framebuffer()
        .expect("Couldn't create frame buffer");
    gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&frame_buffer));
    gl.framebuffer_texture_2d(
        WebGl2RenderingContext::FRAMEBUFFER,
        WebGl2RenderingContext::COLOR_ATTACHMENT0,
        WebGl2RenderingContext::TEXTURE_2D,
        tex.as_ref(),
        0,
    );
    (
        tex.expect("Texture for frame buffer wasn't created"),
        frame_buffer,
    )
}

const VERT_POS: &str = include_str!("shaders/vertex/position.glsl");
const FRAG_CHECKERED: &str = include_str!("shaders/fragment/checkered.glsl");
const FRAG_BLUR: &str = include_str!("shaders/fragment/blur.glsl");
pub fn draw2() {
    let vertex_source = VERT_POS;
    let fragment_source = FRAG_CHECKERED;
    let render_context = RenderContext::new(500, 500);
    let gl = &render_context.gl;

    let mut program_context = ProgramContext::new(gl, vertex_source, fragment_source);
    gl.use_program(Some(&program_context.program));

    let (out_tex, out_buffer) = create_texture_and_frame_buffer(&render_context.gl, 500, 500);

    let fragment_source = FRAG_BLUR;

    let mut program_context = ProgramContext::new(gl, vertex_source, fragment_source);
    gl.use_program(Some(&program_context.program));

    program_context.add_texture(gl, &out_tex, "rtt_texture");
}
