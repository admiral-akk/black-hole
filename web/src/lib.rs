extern crate cfg_if;
extern crate wasm_bindgen;

mod framework;
mod utils;

use framework::frame_buffer_context::FrameBufferContext;
use glam::Vec3;
use rendering::render::render;
use rendering::structs::image_data::ImageData;
use rendering::structs::observer;
use rendering::structs::observer::Observer;
use rendering::structs::ray_cache::RayCache;
use rendering::structs::stars::Stars;
use wasm_timer::SystemTime;
use web_sys::WebGlTexture;

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

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

use crate::framework::texture_utils::generate_texture_from_u8;
use image::io::Reader;

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

fn get_selected_index() -> Result<u32, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    Ok(document
        .get_element_by_id("input")
        .unwrap()
        .dyn_into::<web_sys::HtmlSelectElement>()?
        .selected_index() as u32)
}

enum ExerciseState {
    Exercise0,
    Exercise1(WebGlTexture),
    Exercise2(WebGlTexture, WebGlTexture),
    Exercise3(FrameBufferContext),
    Exercise4(FrameBufferContext, FrameBufferContext, Vec<f32>),
    Exercise5(
        FrameBufferContext,
        FrameBufferContext,
        Vec<f32>,
        Vec<f32>,
        Vec<f32>,
    ),
    Exercise6,
    Exercise7(FrameBufferContext, FrameBufferContext),
    Exercise8(ImageData, Stars, RayCache, Observer),
}
fn init_exercise(gl: &RenderContext, exercise_state: &mut ExerciseState, exercise_index: u32) {
    match exercise_index {
        0 => {
            *exercise_state = ExerciseState::Exercise0;
        }
        1 => {
            let cm = generate_texture_from_u8(&gl.gl, &colormap1(), 256);
            *exercise_state = ExerciseState::Exercise1(cm);
        }
        2 => {
            let cm1 = generate_texture_from_u8(&gl.gl, &colormap1(), 256);
            let cm2 = generate_texture_from_u8(&gl.gl, &colormap2(), 256);
            *exercise_state = ExerciseState::Exercise2(cm1, cm2);
        }
        3 => {
            *exercise_state = ExerciseState::Exercise3(gl.create_framebuffer());
        }
        4 => {
            *exercise_state = ExerciseState::Exercise4(
                gl.create_framebuffer(),
                gl.create_framebuffer(),
                generate_gaussian_weights(1.0, 3),
            );
        }
        5 => {
            *exercise_state = ExerciseState::Exercise5(
                gl.create_framebuffer(),
                gl.create_framebuffer(),
                generate_gaussian_weights(1.0, 3),
                generate_gaussian_weights(2.0, 3),
                generate_gaussian_weights(3.0, 3),
            );
        }
        6 => {
            *exercise_state = ExerciseState::Exercise6;
        }
        7 => {
            *exercise_state =
                ExerciseState::Exercise7(gl.create_framebuffer(), gl.create_framebuffer());
        }
        8 => {
            let uv = generate_uv(512, 512);

            let distance = 3.0;
            let vertical_fov = 120.0;
            let radius = 1.5;

            let mut stars = Stars::new_from_u8(uv, 512, 512);
            let ray_cache = RayCache::compute_new(1024, radius, distance);

            let (pos, dir) = (distance * Vec3::Z, -Vec3::Z);
            let observer = Observer::new(pos, dir, Vec3::Y, vertical_fov);
            stars.update_position(&pos);
            let image_data = ImageData::new(512, 512);
            *exercise_state = ExerciseState::Exercise8(image_data, stars, ray_cache, observer);
        }
        _ => {}
    }
}

impl Default for ExerciseState {
    fn default() -> Self {
        ExerciseState::Exercise0
    }
}

pub struct RenderState {
    gl: RenderContext,
    prev_params: Cell<RenderParams>,
    exercise_state: RefCell<Box<ExerciseState>>,
}

fn clean_up_exercise(gl: &RenderContext, exercise_state: &mut ExerciseState) {
    match exercise_state {
        ExerciseState::Exercise0 => {}
        ExerciseState::Exercise1(cm) => {
            gl.delete_texture(&cm);
        }
        ExerciseState::Exercise2(cm1, cm2) => {
            gl.delete_texture(&cm1);
            gl.delete_texture(&cm2);
        }
        ExerciseState::Exercise3(fb) => {
            gl.delete_framebuffer(&fb);
        }
        ExerciseState::Exercise4(fb1, fb2, _) => {
            gl.delete_framebuffer(&fb1);
            gl.delete_framebuffer(&fb2);
        }
        ExerciseState::Exercise5(fb1, fb2, _, _, _) => {
            gl.delete_framebuffer(&fb1);
            gl.delete_framebuffer(&fb2);
        }
        ExerciseState::Exercise6 => {}
        ExerciseState::Exercise7(fb1, fb2) => {
            gl.delete_framebuffer(&fb1);
            gl.delete_framebuffer(&fb2);
        }
        ExerciseState::Exercise8(..) => {}
        _ => {}
    }
}

fn update_exercise(
    gl: &RenderContext,
    exercise_state: &mut ExerciseState,
    new_params: &RenderParams,
) {
    // Check clean up
    let exercise_index = new_params.select_index;
    let mut new_exercise = false;
    match exercise_state {
        ExerciseState::Exercise0 => {
            new_exercise = exercise_index != 0;
        }
        ExerciseState::Exercise1(..) => {
            new_exercise = exercise_index != 1;
        }
        ExerciseState::Exercise2(..) => {
            new_exercise = exercise_index != 2;
        }
        ExerciseState::Exercise3(..) => {
            new_exercise = exercise_index != 3;
        }
        ExerciseState::Exercise4(..) => {
            new_exercise = exercise_index != 4;
        }
        ExerciseState::Exercise5(..) => {
            new_exercise = exercise_index != 5;
        }
        ExerciseState::Exercise6 => {
            new_exercise = exercise_index != 6;
        }
        ExerciseState::Exercise7(..) => {
            new_exercise = exercise_index != 7;
        }
        ExerciseState::Exercise8(..) => {
            new_exercise = exercise_index != 8;
        }
    }
    if new_exercise {
        clean_up_exercise(gl, exercise_state);
        init_exercise(gl, exercise_state, exercise_index);
    }
}

fn render_exercise(gl: &RenderContext, exercise_state: &mut ExerciseState) {
    let mut frag;
    match exercise_state {
        ExerciseState::Exercise0 => {
            frag = SourceContext::new(include_str!("shaders/fragment/striped.glsl"));
            gl.draw(None, &frag, &[], None);
        }
        ExerciseState::Exercise1(cm) => {
            frag = SourceContext::new(include_str!("shaders/fragment/1_color_map.glsl"));
            let cm_context = UniformContext::new_from_allocated_ref(&cm, "u_palette");
            gl.draw(None, &frag, &[&cm_context], None);
        }
        ExerciseState::Exercise2(cm1, cm2) => {
            frag = SourceContext::new(include_str!("shaders/fragment/2_color_map.glsl"));
            let cm_context1 = UniformContext::new_from_allocated_ref(&cm1, "u_palette_1");
            let cm_context2 = UniformContext::new_from_allocated_ref(&cm2, "u_palette_2");
            gl.draw(None, &frag, &[&cm_context1, &cm_context2], None);
        }
        ExerciseState::Exercise3(fb) => {
            frag = SourceContext::new(include_str!("shaders/fragment/checkered.glsl"));
            let fb_texture =
                UniformContext::new_from_allocated_ref(&fb.backing_texture, "rtt_sampler");
            gl.draw(None, &frag, &[], Some(&fb.frame_buffer));

            frag = SourceContext::new(include_str!("shaders/fragment/blur.glsl"));
            gl.draw(None, &frag, &[&fb_texture], None);
        }
        ExerciseState::Exercise4(fb1, fb2, kernel) => {
            let fb_texture =
                UniformContext::new_from_allocated_ref(&fb1.backing_texture, "rtt_sampler");
            let fb_texture2 =
                UniformContext::new_from_allocated_ref(&fb2.backing_texture, "rtt_sampler");
            let kernel_weights = UniformContext::array_f32(&kernel, "w");

            frag = SourceContext::new(include_str!("shaders/fragment/checkered.glsl"));
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
        ExerciseState::Exercise5(fb1, fb2, r, g, b) => {
            let fb_texture =
                UniformContext::new_from_allocated_ref(&fb1.backing_texture, "rtt_sampler");
            let fb_texture2 =
                UniformContext::new_from_allocated_ref(&fb2.backing_texture, "rtt_sampler");
            let r_kernel_weights = UniformContext::array_f32(&r, "r");
            let g_kernel_weights = UniformContext::array_f32(&g, "g");
            let b_kernel_weights = UniformContext::array_f32(&b, "b");

            frag = SourceContext::new(include_str!("shaders/fragment/checkered.glsl"));
            gl.draw(None, &frag, &[], Some(&fb1.frame_buffer));

            for _ in 0..10 {
                frag =
                    SourceContext::new(include_str!("shaders/fragment/multi_gaussian_blur.glsl"));
                frag.add_parameter("HORIZONTAL", "TRUE");
                frag.add_parameter("K", &r.len().to_string());
                gl.draw(
                    None,
                    &frag,
                    &[
                        &fb_texture,
                        &r_kernel_weights,
                        &g_kernel_weights,
                        &b_kernel_weights,
                    ],
                    Some(&fb2.frame_buffer),
                );

                frag =
                    SourceContext::new(include_str!("shaders/fragment/multi_gaussian_blur.glsl"));
                frag.add_parameter("K", &r.len().to_string());
                frag.add_parameter("VERTICAL", "TRUE");
                gl.draw(
                    None,
                    &frag,
                    &[
                        &fb_texture2,
                        &r_kernel_weights,
                        &g_kernel_weights,
                        &b_kernel_weights,
                    ],
                    Some(&fb1.frame_buffer),
                );
            }
            frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
            gl.draw(None, &frag, &[&fb_texture], None);
        }
        ExerciseState::Exercise6 => {
            let time = 1.0;

            let pos_seed = [52.912 * time, 11.30 * time];
            let color_seed = [10.5121 * time, 22.958 * time, 25.1 * time];

            frag = SourceContext::new(include_str!("shaders/fragment/psuedo_random.glsl"));
            let pos_seed_uniform = UniformContext::array_f32(&pos_seed, "pos_seed");
            let color_seed_uniform = UniformContext::array_f32(&color_seed, "color_seed");
            gl.draw(None, &frag, &[&pos_seed_uniform, &color_seed_uniform], None);
        }
        ExerciseState::Exercise7(fb1, fb2) => {
            let state_texture =
                UniformContext::new_from_allocated_ref(&fb1.backing_texture, "rtt_sampler");
            frag = SourceContext::new(include_str!("shaders/fragment/add_white.glsl"));
            gl.draw(None, &frag, &[&state_texture], Some(&fb2.frame_buffer));

            let fb_texture =
                UniformContext::new_from_allocated_ref(&fb2.backing_texture, "rtt_sampler");
            gl.draw(None, &frag, &[&fb_texture], Some(&fb1.frame_buffer));
            gl.draw(None, &frag, &[&fb_texture], None);
        }
        ExerciseState::Exercise8(image_data, stars, ray_cache, observer) => {
            render(image_data, observer, stars, ray_cache);

            let image = generate_texture_from_u8(&gl.gl, image_data.get_image(), 512);
            let image_context = UniformContext::new_from_allocated_ref(&image, "rtt_sampler");
            frag = SourceContext::new(RENDER_TEXTURE_DEFAULT);
            gl.draw(None, &frag, &[&image_context], None);
            gl.delete_texture(&image);
        }
    }
}

const EXERCISE_COUNT: u32 = 9;
const RENDER_TEXTURE_DEFAULT: &str = include_str!("shaders/fragment/render_texture.glsl");
impl RenderState {
    fn render(&self, params: &RenderParams) -> Result<(), JsValue> {
        console_log!("params: {:?}", params);
        let gl = &self.gl;

        update_exercise(gl, &mut *self.exercise_state.borrow_mut(), params);
        render_exercise(gl, &mut *self.exercise_state.borrow_mut());
        self.prev_params.set(*params);
        Ok(())
    }
}

impl RenderState {
    pub fn new(width: u32, height: u32) -> Result<RenderState, JsValue> {
        let gl = RenderContext::new(width, height);

        Ok(RenderState {
            gl,
            prev_params: Cell::default(),
            exercise_state: RefCell::default(),
        })
    }
}

fn generate_uv(width: u32, height: u32) -> Vec<u8> {
    let mut uv = Vec::new();
    for x in 0..width {
        for y in 0..height {
            let r = 255 * x / width;
            let g = 255 * y / height;
            let b = 0;
            let a = 255;
            uv.push(r as u8);
            uv.push(g as u8);
            uv.push(b as u8);
            uv.push(a as u8);
        }
    }

    uv
}

use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[derive(Clone, Copy, Debug, Default)]
struct RenderParams {
    pub seconds_since_start: f32,
    pub select_index: u32,
    pub mouse_pos: Option<(i32, i32)>,
}

impl RenderParams {
    pub fn update_time(&self, seconds_since_start: f32) -> RenderParams {
        let mut c = self.clone();
        c.seconds_since_start = seconds_since_start;
        c
    }

    pub fn update_exercise(&self, select_index: u32) -> RenderParams {
        let mut c = self.clone();
        c.seconds_since_start = 0.0;
        c.select_index = select_index;
        c
    }

    pub fn update_mouse_pos(&self, mouse_pos: Option<(i32, i32)>) -> RenderParams {
        let mut c = self.clone();
        c.mouse_pos = mouse_pos;
        c
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let document = document();
    let select = document
        .get_element_by_id("input")
        .unwrap()
        .dyn_into::<web_sys::HtmlSelectElement>()?;
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    for i in 1..=EXERCISE_COUNT {
        let option = HtmlOptionElement::new_with_text(&format!("Exercise {}", i))?;
        select.append_child(&option)?;
    }

    let start_time = Rc::new(Cell::new(SystemTime::now()));
    let params = Rc::new(Cell::new(RenderParams::default()));
    let renderer = Rc::new(RenderState::new(512, 512)?);
    {
        let start_time = start_time.clone();
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            start_time.set(SystemTime::now());
            let exercise = get_selected_index().unwrap();
            params.set(params.get().update_exercise(exercise));
        }) as Box<dyn FnMut(_)>);
        select.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            params.set(
                params
                    .get()
                    .update_mouse_pos(Some((event.offset_x(), event.offset_y()))),
            );
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
            params.set(params.get().update_mouse_pos(None));
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseleave", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    let render_func = Rc::new(RefCell::new(None));
    let g = render_func.clone();
    {
        let renderer = renderer.clone();
        let start_time = start_time.clone();
        let params = params.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let seconds_since_start = SystemTime::now()
                .duration_since(start_time.get())
                .unwrap()
                .as_secs_f32();
            params.set(params.get().update_time(seconds_since_start));
            renderer.render(&params.get()).unwrap();
            requestAnimationFrame(render_func.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
        request_animation_frame(g.borrow().as_ref().unwrap());
    }

    Ok(())
}
