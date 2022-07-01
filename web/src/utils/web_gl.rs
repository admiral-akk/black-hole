use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

use super::shader_cache::{get_shaders, Exercise};

pub struct WebGLWrapper {
    gl: WebGl2RenderingContext,
    program: WebGlProgram,
    texture_count: i32,
}
impl WebGLWrapper {
    pub fn new(exercise: &Exercise) -> WebGLWrapper {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        let context = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();
        canvas.set_width(500);
        canvas.set_height(500);
        context.clear_color(0.5, 0.7, 0.6, 1.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        context.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        let context = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();

        let (vert, frag) = get_shaders(exercise);

        let vert_shader =
            compile_shader(&context, WebGl2RenderingContext::VERTEX_SHADER, &vert).unwrap();

        let frag_shader =
            compile_shader(&context, WebGl2RenderingContext::FRAGMENT_SHADER, &frag).unwrap();
        let program = link_program(&context, &vert_shader, &frag_shader).unwrap();
        context.use_program(Some(&program));
        add_vertices(&context, &program);
        WebGLWrapper {
            gl: context,
            program,
            texture_count: 0,
        }
    }

    pub fn add_texture(&mut self, colors: &[u8], width: i32, height: i32, name: &str) {
        let texture = self.gl.create_texture().unwrap();
        let location = self.gl.get_uniform_location(&self.program, name).unwrap();
        self.gl.uniform1i(Some(&location), self.texture_count);
        match self.texture_count {
            0 => {
                self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
            }
            1 => {
                self.gl.active_texture(WebGl2RenderingContext::TEXTURE1);
            }
            _ => {}
        }
        self.texture_count += 1;
        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        self.gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                width,
                height,
                0,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                Some(&colors),
            )
            .unwrap();
        self.gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        self.gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        self.gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        self.gl.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
    }

    pub fn draw(&self) {
        draw(&self.gl, 4);
    }
}

pub fn add_vertices(gl: &WebGl2RenderingContext, program: &WebGlProgram) {
    let vertices: [f32; 12] = [
        -1.0, -1.0, 0.0, 1.0, -1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, 0.0,
    ];

    let position_attribute_location = gl.get_attrib_location(program, "position");
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

    let vao = gl
        .create_vertex_array()
        .ok_or("Could not create vertex array object")
        .unwrap();
    gl.bind_vertex_array(Some(&vao));

    gl.vertex_attrib_pointer_with_i32(0, 3, WebGl2RenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(position_attribute_location as u32);

    gl.bind_vertex_array(Some(&vao));
}

fn draw(context: &WebGl2RenderingContext, vert_count: i32) {
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    context.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, vert_count);
}

pub fn compile_shader(
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

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
