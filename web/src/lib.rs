extern crate cfg_if;
extern crate wasm_bindgen;

mod framework;
mod utils;

use cfg_if::cfg_if;
use framework::render_context::RenderContext;
use framework::source_context::SourceContext;
use framework::uniform_context::UniformContext;
use utils::color_map::colormap1;
use utils::color_map::colormap2;
use utils::gaussian::generate_gaussian_weights;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlOptionElement;
use web_sys::HtmlSelectElement;

// https://rustwasm.github.io/wasm-bindgen/exbuild/webgl/
// https://webglfundamentals.org/webgl/lessons/webgl-fundamentals.html
// https://michaelerule.github.io/webgpgpu/examples/Example_1_hello_gpu.html
// https://github.com/michaelerule/webgpgpu/blob/master/examples/Example_1_hello_gpu.html
// https://github.com/michaelerule/webgpgpu
cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[wasm_bindgen]
pub fn get_renderer() -> Result<RenderState, JsValue> {
    RenderState::new(512, 512)
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    fn setInterval(closure: &Closure<dyn FnMut()>, millis: u32) -> f64;
    fn cancelInterval(token: f64);
    fn requestAnimationFrame(closure: &Closure<dyn FnMut()>) -> u32;
    fn cancelAnimationFrame(id: u32);
}

fn get_select() -> Result<HtmlSelectElement, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    Ok(document
        .get_element_by_id("input")
        .unwrap()
        .dyn_into::<web_sys::HtmlSelectElement>()?)
}

pub fn init_select() -> Result<HtmlSelectElement, JsValue> {
    let select = get_select()?;

    for i in 1..=EXERCISE_COUNT {
        let option = HtmlOptionElement::new_with_text(&format!("Exercise {}", i))?;
        select.append_child(&option)?;
    }
    Ok(select)
}

#[wasm_bindgen]
pub struct RenderState {
    gl: RenderContext,
    select: HtmlSelectElement,
}

const VERTEX_DEFAULT: &str = include_str!("shaders/vertex/position.glsl");
const RENDER_TEXTURE_DEFAULT: &str = include_str!("shaders/fragment/render_texture.glsl");

const EXERCISE_COUNT: u32 = 7;
#[wasm_bindgen]
impl RenderState {
    pub fn render(&self) -> Result<(), JsValue> {
        console_log!("selected index: {}", self.select.selected_index());
        let exercise = self.select.selected_index() + 1;

        let gl = &self.gl;
        let vertex = SourceContext::new(VERTEX_DEFAULT);
        let mut frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
        let frame_buffer;
        let backing_texture;
        let frame_buffer2;
        let backing_texture2;
        let color_map_1 = colormap1();
        let color_map_2 = colormap2();
        let kernel = generate_gaussian_weights(1.0, 3);
        let r = generate_gaussian_weights(1.0, 3);
        let g = generate_gaussian_weights(2.0, 3);
        let b = generate_gaussian_weights(3.0, 3);
        match exercise {
            1 => {
                frag = SourceContext::new(include_str!("shaders/fragment/striped.glsl"));
                self.gl.draw(&vertex, &frag, &[], None);
            }
            2 => {
                frag = SourceContext::new(include_str!("shaders/fragment/1_color_map.glsl"));
                let cm = UniformContext::new_from_u8(gl, &color_map_1, 256, "u_palette");
                self.gl.draw(&vertex, &frag, &[&cm], None);
            }
            3 => {
                frag = SourceContext::new(include_str!("shaders/fragment/2_color_map.glsl"));
                let cm_1 = UniformContext::new_from_u8(gl, &color_map_1, 256, "u_palette_1");
                let cm_2 = UniformContext::new_from_u8(gl, &color_map_2, 256, "u_palette_2");
                self.gl.draw(&vertex, &frag, &[&cm_1, &cm_2], None);
            }
            4 => {
                frag = SourceContext::new(include_str!("shaders/fragment/checkered.glsl"));
                (frame_buffer, backing_texture) = self.gl.create_framebuffer();
                let fb_texture = UniformContext::new_from_allocated(backing_texture, "rtt_sampler");
                self.gl
                    .draw(&vertex, &frag, &[], Some(&frame_buffer.frame_buffer));

                frag = SourceContext::new(include_str!("shaders/fragment/blur.glsl"));
                self.gl.draw(&vertex, &frag, &[&fb_texture], None);
            }
            5 => {
                (frame_buffer, backing_texture) = self.gl.create_framebuffer();
                let fb_texture = UniformContext::new_from_allocated(backing_texture, "rtt_sampler");
                (frame_buffer2, backing_texture2) = self.gl.create_framebuffer();
                let fb_texture2 =
                    UniformContext::new_from_allocated(backing_texture2, "rtt_sampler");
                let kernel_weights = UniformContext::array_f32(&kernel, "w");

                frag = SourceContext::new(include_str!("shaders/fragment/checkered.glsl"));
                self.gl
                    .draw(&vertex, &frag, &[], Some(&frame_buffer.frame_buffer));

                for _ in 0..10 {
                    frag = SourceContext::new(include_str!("shaders/fragment/gaussian_blur.glsl"));
                    frag.add_parameter("HORIZONTAL", "TRUE");
                    frag.add_parameter("K", &kernel.len().to_string());
                    self.gl.draw(
                        &vertex,
                        &frag,
                        &[&fb_texture, &kernel_weights],
                        Some(&frame_buffer2.frame_buffer),
                    );

                    frag = SourceContext::new(include_str!("shaders/fragment/gaussian_blur.glsl"));
                    frag.add_parameter("K", &kernel.len().to_string());
                    frag.add_parameter("VERTICAL", "TRUE");
                    self.gl.draw(
                        &vertex,
                        &frag,
                        &[&fb_texture2, &kernel_weights],
                        Some(&frame_buffer.frame_buffer),
                    );
                }
                frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
                self.gl.draw(&vertex, &frag, &[&fb_texture], None);
            }
            6 => {
                (frame_buffer, backing_texture) = self.gl.create_framebuffer();
                let fb_texture = UniformContext::new_from_allocated(backing_texture, "rtt_sampler");
                (frame_buffer2, backing_texture2) = self.gl.create_framebuffer();
                let fb_texture2 =
                    UniformContext::new_from_allocated(backing_texture2, "rtt_sampler");
                let r_kernel_weights = UniformContext::array_f32(&r, "r");
                let g_kernel_weights = UniformContext::array_f32(&g, "g");
                let b_kernel_weights = UniformContext::array_f32(&b, "b");

                frag = SourceContext::new(include_str!("shaders/fragment/checkered.glsl"));
                self.gl
                    .draw(&vertex, &frag, &[], Some(&frame_buffer.frame_buffer));

                for _ in 0..10 {
                    frag = SourceContext::new(include_str!(
                        "shaders/fragment/multi_gaussian_blur.glsl"
                    ));
                    frag.add_parameter("HORIZONTAL", "TRUE");
                    frag.add_parameter("K", &kernel.len().to_string());
                    self.gl.draw(
                        &vertex,
                        &frag,
                        &[
                            &fb_texture,
                            &r_kernel_weights,
                            &g_kernel_weights,
                            &b_kernel_weights,
                        ],
                        Some(&frame_buffer2.frame_buffer),
                    );

                    frag = SourceContext::new(include_str!(
                        "shaders/fragment/multi_gaussian_blur.glsl"
                    ));
                    frag.add_parameter("K", &kernel.len().to_string());
                    frag.add_parameter("VERTICAL", "TRUE");
                    self.gl.draw(
                        &vertex,
                        &frag,
                        &[
                            &fb_texture2,
                            &r_kernel_weights,
                            &g_kernel_weights,
                            &b_kernel_weights,
                        ],
                        Some(&frame_buffer.frame_buffer),
                    );
                }
                frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
                self.gl.draw(&vertex, &frag, &[&fb_texture], None);
            }
            7 => {
                let time = 1.0;

                let pos_seed = [52.912 * time, 11.30 * time];
                let color_seed = [10.5121 * time, 22.958 * time, 25.1 * time];

                frag = SourceContext::new(include_str!("shaders/fragment/psuedo_random.glsl"));
                let pos_seed_uniform = UniformContext::array_f32(&pos_seed, "pos_seed");
                let color_seed_uniform = UniformContext::array_f32(&color_seed, "color_seed");
                self.gl.draw(
                    &vertex,
                    &frag,
                    &[&pos_seed_uniform, &color_seed_uniform],
                    None,
                );
            }
            _ => {}
        }
        Ok(())
    }
}

impl RenderState {
    pub fn new(width: u32, height: u32) -> Result<RenderState, JsValue> {
        Ok(RenderState {
            gl: RenderContext::new(width, height),
            select: get_select()?,
        })
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    init_select();
    Ok(())
}
