use wasm_bindgen::JsCast;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlFramebuffer, WebGlProgram, WebGlShader,
};

use crate::utils::texture::Texture;

pub struct RenderContext {
    pub gl: WebGl2RenderingContext,
    pub canvas: HtmlCanvasElement,
}

fn initialize_raster_vertices(gl: &WebGl2RenderingContext) {
    let vertices: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];

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

impl RenderContext {
    pub fn new(width: u32, height: u32) -> RenderContext {
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
        canvas.set_width(width);
        canvas.set_height(height);
        gl.clear_color(0.5, 0.7, 0.6, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        initialize_raster_vertices(&gl);
        RenderContext { gl, canvas }
    }

    pub fn draw(&self, out_buffer: Option<WebGlFramebuffer>) {
        let gl = &self.gl;

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        if out_buffer.is_some() {
            gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, out_buffer.as_ref());
        }

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);

        if out_buffer.is_some() {
            gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        }
    }
}

pub struct ProgramContext {
    pub program: WebGlProgram,
    texture_count: u32,
}

fn initalize_position(gl: &WebGl2RenderingContext, program: &WebGlProgram) {
    let position_attribute_location = gl.get_attrib_location(program, "position");

    gl.vertex_attrib_pointer_with_i32(
        position_attribute_location as u32,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(position_attribute_location as u32);
}

impl ProgramContext {
    pub fn new(
        gl: &WebGl2RenderingContext,
        vertex_source: &str,
        fragment_source: &str,
    ) -> ProgramContext {
        let shader = gl
            .create_shader(WebGl2RenderingContext::VERTEX_SHADER)
            .ok_or_else(|| String::from("Unable to create shader object"))
            .unwrap();
        gl.shader_source(&shader, vertex_source);
        gl.compile_shader(&shader);

        let vert_shader = shader;

        let shader = gl
            .create_shader(WebGl2RenderingContext::FRAGMENT_SHADER)
            .ok_or_else(|| String::from("Unable to create shader object"))
            .unwrap();
        gl.shader_source(&shader, fragment_source);
        gl.compile_shader(&shader);

        let frag_shader = shader;
        let program = gl
            .create_program()
            .ok_or_else(|| String::from("Unable to create shader object"))
            .unwrap();

        gl.attach_shader(&program, &vert_shader);
        gl.attach_shader(&program, &frag_shader);
        gl.link_program(&program);
        initalize_position(gl, &program);
        ProgramContext {
            program,
            texture_count: 0,
        }
    }

    pub fn add_texture_from_u8(
        &mut self,
        gl: &WebGl2RenderingContext,
        arr: &[u8],
        width: i32,
        name: &str,
    ) {
        let program = &self.program;
        let texture = gl.create_texture();
        gl.active_texture(WebGl2RenderingContext::TEXTURE0 + self.texture_count);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());
        let loc = gl.get_uniform_location(program, name);
        gl.uniform1i(loc.as_ref(), self.texture_count as i32);

        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            width,
            (arr.len() / (4 * width) as usize) as i32,
            0,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(arr),
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
}

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

    for i in 0..textures.len() {
        program_context.add_texture_from_u8(
            gl,
            &textures[i].arr,
            textures[i].width,
            &textures[i].name,
        );
    }

    render_context.draw(out_buffer);
}
