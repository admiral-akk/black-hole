use std::fmt::Error;

use wasm_bindgen::JsCast;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlShader, WebGlTexture,
};

fn f32_buf(gl: &WebGl2RenderingContext, arr: &[f32]) -> Result<(), Error> {
    let buffer = gl
        .create_buffer()
        .ok_or("Failed to create f32 buffer.")
        .unwrap();
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    unsafe {
        // Safe as long as there's no memory allocation between this and buffering the data to webgl.
        let positions_array_buf_view = js_sys::Float32Array::view(&arr);

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }
    Ok(())
}

fn set_attributes(gl: &WebGl2RenderingContext, position_loc: u32) {
    gl.enable_vertex_attrib_array(position_loc);
    gl.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);

    let vao = gl
        .create_vertex_array()
        .ok_or("Could not create vertex array object")
        .unwrap();
    gl.bind_vertex_array(Some(&vao));
}

fn create_texture_and_frame_buffer(
    gl: &WebGl2RenderingContext,
    width: i32,
    height: i32,
) -> Result<(WebGlTexture, WebGlFramebuffer), Error> {
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
    Ok((
        tex.expect("Texture for frame buffer wasn't created"),
        frame_buffer,
    ))
}

fn create_program_from_source(
    gl: &WebGl2RenderingContext,
    vert_source: &str,
    frag_source: &str,
) -> Result<WebGlProgram, Error> {
    let vert_shader = compile_shader(gl, WebGl2RenderingContext::VERTEX_SHADER, vert_source)
        .expect("Vertex shader failed to compile");
    let frag_shader = compile_shader(gl, WebGl2RenderingContext::FRAGMENT_SHADER, frag_source)
        .expect("Fragment shader failed to compile");

    Ok(link_program(gl, &vert_shader, &frag_shader).expect("Failed to link shader program"))
}

fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .expect("Unable to create shader");
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}
fn link_program(
    gl: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

const RASTER_VERTICES: [f32; 12] = [
    -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, 0.0,
];

const POSITION_ATTRIBUTE_NAME: &str = "position";
const VERT_POS: &str = include_str!("shaders/vertex/position.glsl");
const FRAG_CHECKERED: &str = include_str!("shaders/fragment/checkered.glsl");
const FRAG_BLUR: &str = include_str!("shaders/fragment/blur.glsl");

// https://stackoverflow.com/questions/41824631/how-to-work-with-framebuffers-in-webgl
pub fn testing() -> Result<(), Error> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement =
        canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    canvas.set_width(500);
    canvas.set_height(500);

    let gl = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()
        .unwrap();

    let color_program = create_program_from_source(&gl, VERT_POS, FRAG_BLUR)
        .expect("Failed to create color program");
    // let blur_program = create_program_from_source(&gl, VERT_POS, FRAG_BLUR)
    //     .expect("Failed to create blur program");

    // let tex_frame_buffer1 =
    //     create_texture_and_frame_buffer(&gl, 64, 64).expect("failed to create frame buffer 1");
    // let tex_frame_buffer2 =
    //     create_texture_and_frame_buffer(&gl, 64, 64).expect("failed to create frame buffer 2");

    f32_buf(&gl, &RASTER_VERTICES).expect("Expected buffer");
    gl.use_program(Some(&color_program));

    let color_program_position_loc =
        gl.get_attrib_location(&color_program, POSITION_ATTRIBUTE_NAME);
    set_attributes(&gl, color_program_position_loc as u32);

    // gl.bind_framebuffer(
    //     WebGl2RenderingContext::FRAMEBUFFER,
    //     Some(&tex_frame_buffer1.1),
    // );
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    gl.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);
    return Ok(());
    // gl.bind_framebuffer(
    //     WebGl2RenderingContext::FRAMEBUFFER,
    //     Some(&tex_frame_buffer2.1),
    // );
    // gl.viewport(0, 0, 64, 64);
    // gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
    // gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
    // gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
    // let mix_position_loc = gl.get_attrib_location(&blur_program, POSITION_ATTRIBUTE_NAME);
    // set_attributes(&gl, mix_position_loc as u32);
    // let tex1_loc = gl.get_uniform_location(&blur_program, "rtt_sampler");
    // let tex2_loc = gl.get_uniform_location(&blur_program, "tex2");
    // gl.use_program(Some(&blur_program));

    // gl.uniform1i(tex1_loc.as_ref(), 0);
    // gl.active_texture(WebGl2RenderingContext::TEXTURE0 + 0);
    // gl.bind_texture(
    //     WebGl2RenderingContext::TEXTURE_2D,
    //     Some(&tex_frame_buffer1.0),
    // );
    //gl.uniform1i(tex2_loc.as_ref(), 1);
    // gl.active_texture(WebGl2RenderingContext::TEXTURE0 + 1);
    // gl.bind_texture(
    //     WebGl2RenderingContext::TEXTURE_2D,
    //     Some(&tex_frame_buffer2.0),
    // );

    // gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6);
    // Ok(())
}

// Draw both textures to the canvas
