use std::collections::HashMap;

use wasm_bindgen::JsCast;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlFramebuffer, WebGlProgram, WebGlShader,
    WebGlTexture,
};

use super::{shader_cache::Exercise, texture::Texture};

pub struct WebGLWrapper {
    gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
    texture_count: u32,
}

// Create context

// Get frame buffer

// Set up program

// Load textures and compute, if has framebuffer then output to that. If not, then to screen.

// Clear

impl WebGLWrapper {
    pub fn new(exercise: &Exercise) -> WebGLWrapper {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        let gl = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();
        canvas.set_width(500);
        canvas.set_height(500);
        gl.clear_color(0.5, 0.7, 0.6, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        add_vertices(&gl);
        WebGLWrapper {
            gl,
            canvas,
            texture_count: 0,
        }
    }

    fn add_texture(&mut self, program: &WebGlProgram, colors: &[u8], width: i32, name: &str) {
        let gl = &self.gl;
        let texture = gl.create_texture();
        gl.active_texture(WebGl2RenderingContext::TEXTURE0 + self.texture_count);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());
        let loc = gl.get_uniform_location(&program, name);
        gl.uniform1i(loc.as_ref(), self.texture_count as i32);

        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            width,
            (colors.len() / (4 * width) as usize) as i32,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&colors),
        )
        .unwrap();
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
        self.texture_count += 1;
    }

    pub fn draw(
        &mut self,
        vertex_source: &str,
        fragment_source: &str,
        textures: &[Texture],
        out_buffer: Option<WebGlFramebuffer>,
    ) {
        let vert_shader = compile_shader(
            &self.gl,
            WebGl2RenderingContext::VERTEX_SHADER,
            vertex_source,
        )
        .unwrap();
        let frag_shader = compile_shader(
            &self.gl,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            fragment_source,
        )
        .unwrap();
        let program = link_program(&self.gl, &vert_shader, &frag_shader).unwrap();
        self.gl.use_program(Some(&program));

        for texture in textures {
            self.add_texture(&program, &texture.arr, texture.width, &texture.name);
        }

        let position_attribute_location = self.gl.get_attrib_location(&program, "position");
        let vao = self
            .gl
            .create_vertex_array()
            .ok_or("Could not create vertex array object")
            .unwrap();
        self.gl.bind_vertex_array(Some(&vao));

        self.gl
            .vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
        self.gl
            .enable_vertex_attrib_array(position_attribute_location as u32);

        self.gl.bind_vertex_array(Some(&vao));
        self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        if out_buffer.is_some() {
            self.gl
                .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, out_buffer.as_ref());
        }

        self.gl
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);

        if out_buffer.is_some() {
            self.gl
                .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        }
    }
}

fn add_vertices(gl: &WebGl2RenderingContext) {
    let vertices: [f32; 12] = [
        -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, 0.0,
    ];

    let buffer = gl.create_buffer().ok_or("Failed to create buffer").unwrap();
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        // Safe as long as there's no memory allocation between this and buffering the data to webgl.
        let positions_array_buf_view = js_sys::Float32Array::view(&vertices);

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }
}

fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
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

// pub fn render_texture(&self) -> Result<(), JsValue> {
//     let gl = &self.gl;
//     let frame_buffer = gl.create_framebuffer();
//     gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, frame_buffer.as_ref());
//     let texture = gl.create_texture();
//     gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());
//     gl.tex_parameteri(
//         WebGl2RenderingContext::TEXTURE_2D,
//         WebGl2RenderingContext::TEXTURE_MAG_FILTER,
//         WebGl2RenderingContext::LINEAR as i32,
//     );
//     gl.tex_parameteri(
//         WebGl2RenderingContext::TEXTURE_2D,
//         WebGl2RenderingContext::TEXTURE_MIN_FILTER,
//         WebGl2RenderingContext::LINEAR as i32,
//     );
//     gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
//         WebGl2RenderingContext::TEXTURE_2D,
//         0,
//         WebGl2RenderingContext::RGBA as i32,
//         self.canvas.width() as i32,
//         self.canvas.height() as i32,
//         0,
//         WebGl2RenderingContext::RGBA,
//         WebGl2RenderingContext::UNSIGNED_BYTE,
//         None,
//     )?;
//     gl.framebuffer_texture_2d(
//         WebGl2RenderingContext::FRAMEBUFFER,
//         WebGl2RenderingContext::COLOR_ATTACHMENT0,
//         WebGl2RenderingContext::TEXTURE_2D,
//         texture.as_ref(),
//         0,
//     );

//     gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
//     gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
//     gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

//     Ok(())
// }
