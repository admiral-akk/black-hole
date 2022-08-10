use wasm_bindgen::JsCast;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderingContext,
    WebGlTexture,
};

use super::{
    frame_buffer_context::FrameBufferContext, program_context::ProgramContext,
    source_context::SourceContext, uniform_context::UniformContext,
};

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
        gl.get_extension("EXT_color_buffer_float").unwrap();
        gl.get_extension("OES_texture_float_linear").unwrap();
        gl.get_extension("OES_texture_float").unwrap();
        canvas.set_width(width);
        canvas.set_height(height);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        initialize_raster_vertices(&gl);
        RenderContext { gl, canvas }
    }
    pub fn create_framebuffer(&self) -> FrameBufferContext {
        self.create_framebuffer_with_size(self.canvas.width() as i32, self.canvas.height() as i32)
    }

    pub fn delete_texture(&self, uc: &WebGlTexture) {
        self.gl.delete_texture(Some(uc));
    }
    pub fn delete_framebuffer(&self, fb: &FrameBufferContext) {
        self.gl.delete_framebuffer(Some(&fb.frame_buffer));
    }

    pub fn create_framebuffer_with_size(&self, width: i32, height: i32) -> FrameBufferContext {
        FrameBufferContext::new(&self.gl, width, height)
    }

    pub fn read_from_frame_buffer(
        &self,
        fb: &FrameBufferContext,
        width: i32,
        height: i32,
    ) -> Vec<f32> {
        self.gl
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&fb.frame_buffer));
        let vals = vec![0.0_f32; 4 * (width * height) as usize];
        unsafe {
            // Safe as long as there's no memory allocation between this and buffering the data to webgl.
            let rays_view = js_sys::Float32Array::view(&vals);
            self.gl
                .read_pixels_with_opt_array_buffer_view(
                    0,
                    0,
                    1024,
                    1024,
                    WebGlRenderingContext::RGBA,
                    WebGlRenderingContext::FLOAT,
                    Some(&rays_view),
                )
                .unwrap();
        }
        self.gl
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

        vals
    }

    pub fn get_program(
        &self,
        fragment_source: &SourceContext,
        textures: &[UniformContext],
    ) -> ProgramContext {
        let gl = &self.gl;

        let mut program_context = ProgramContext::new(gl, None, fragment_source);
        gl.use_program(Some(&program_context.program));

        for i in 0..textures.len() {
            textures[i].add_to_program(self, &mut program_context);
        }
        gl.use_program(None);

        program_context
    }

    pub fn run_program(&self, program: &ProgramContext, out_buffer: Option<&WebGlFramebuffer>) {
        let gl = &self.gl;
        gl.use_program(Some(&program.program));
        if out_buffer.is_some() {
            gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, out_buffer);
        }

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);

        if out_buffer.is_some() {
            gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        }
        gl.use_program(None);
        gl.flush();
    }
}
