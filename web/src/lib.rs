extern crate cfg_if;
extern crate wasm_bindgen;

mod exercises;
mod framework;
mod utils;

use exercises::exercise_1;
use exercises::exercise_10;
use exercises::exercise_2;
use exercises::exercise_3;
use exercises::exercise_4;
use exercises::exercise_8;
use exercises::exercise_9;
use framework::frame_buffer_context::FrameBufferContext;

use framework::program_context::ProgramContext;
use glam::IVec2;
use glam::Mat3;
use glam::Quat;
use glam::Vec2;
use glam::Vec3;

use image::DynamicImage;
use js_sys::Uint8Array;
use rendering::structs::image_data::ImageData;

use rendering::structs::observer::Observer;
use rendering::structs::ray_cache::RayCache;
use rendering::structs::stars::Stars;

use wasm_bindgen_futures::JsFuture;
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

#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (crate::log(&format_args!($($t)*).to_string()))
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
    Exercise8(ImageData, Stars, RayCache, Observer, BlackHoleParams),
    Exercise9(BlackHoleParams),
    Exercise10(BlackHoleParams, ProgramContext, WebGlTexture),
}

impl Default for ExerciseState {
    fn default() -> Self {
        ExerciseState::Exercise0
    }
}

pub struct BlackHoleParams {
    pub dimensions: IVec2,
    pub distance: f32,
    pub vertical_fov_degrees: f32,
    pub black_hole_radius: f32,
    pub cache_width: i32,
    pub normalized_pos: Vec3,
    pub normalized_dir: Vec3,
    pub normalized_up: Vec3,
    pub observer_mat: Mat3,
}

impl BlackHoleParams {
    fn new(
        dimensions: IVec2,
        distance: f32,
        vertical_fov_degrees: f32,
        black_hole_radius: f32,
        cache_width: i32,
        pos: Vec3,
        dir: Vec3,
        up: Vec3,
    ) -> Self {
        let observer_quat = Quat::from_rotation_arc(pos.normalize(), -Vec3::Z);
        let euler = Quat::to_euler(observer_quat, glam::EulerRot::XYZ);
        let observer_mat = Mat3::from_euler(glam::EulerRot::XYZ, euler.0, euler.1, euler.2);

        Self {
            dimensions,
            distance,
            vertical_fov_degrees,
            black_hole_radius,
            cache_width,
            normalized_pos: pos.normalize(),
            normalized_dir: dir.normalize(),
            normalized_up: up.normalize(),
            observer_mat,
        }
    }

    fn uniform_context(&self) -> Vec<UniformContext> {
        let mut v = Vec::new();
        v.push(UniformContext::ivec2(self.dimensions, "dimensions"));
        v.push(UniformContext::f32(self.distance, "distance"));
        v.push(UniformContext::f32(
            self.vertical_fov_degrees,
            "vertical_fov_degrees",
        ));
        v.push(UniformContext::f32(
            self.black_hole_radius,
            "black_hole_radius",
        ));
        v.push(UniformContext::i32(self.cache_width, "cache_width"));
        v.push(UniformContext::vec3(self.normalized_pos, "normalized_pos"));
        v.push(UniformContext::vec3(self.normalized_dir, "normalized_dir"));
        v.push(UniformContext::vec3(self.normalized_up, "normalized_up"));
        v.push(UniformContext::mat3x3(self.observer_mat, "observer_mat"));
        v
    }
}

impl ExerciseState {
    pub fn index(&self) -> u32 {
        match self {
            ExerciseState::Exercise0 => 0,
            ExerciseState::Exercise1(..) => 1,
            ExerciseState::Exercise2(..) => 2,
            ExerciseState::Exercise3(..) => 3,
            ExerciseState::Exercise4(..) => 4,
            ExerciseState::Exercise5(..) => 5,
            ExerciseState::Exercise6 => 6,
            ExerciseState::Exercise7(..) => 7,
            ExerciseState::Exercise8(..) => 8,
            ExerciseState::Exercise9(..) => 9,
            ExerciseState::Exercise10(..) => 10,
        }
    }
}

fn init_exercise(
    gl: &RenderContext,
    exercise_state: &mut ExerciseState,
    exercise_index: u32,
    im: &DynamicImage,
) {
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
            let distance = 3.0;
            let vertical_fov_degrees = 120.0;
            let black_hole_radius = 1.5;
            let cache_width: i32 = 1024;
            let (pos, dir, up) = (distance * Vec3::Z, -Vec3::Z, Vec3::Y);
            let params = BlackHoleParams::new(
                IVec2::new(1024, 1024),
                distance,
                vertical_fov_degrees,
                black_hole_radius,
                cache_width,
                pos,
                dir,
                up,
            );
            let uv = generate_uv(params.dimensions.x as u32, params.dimensions.y as u32);

            let mut stars =
                Stars::new_from_u8(uv, params.dimensions.x as u32, params.dimensions.y as u32);
            let ray_cache = RayCache::compute_new(
                params.cache_width as usize,
                params.black_hole_radius,
                params.distance,
            );

            let observer = Observer::new(
                params.normalized_pos,
                params.normalized_dir,
                params.normalized_up,
                params.vertical_fov_degrees,
            );
            stars.update_position(&&params.normalized_pos);
            let image_data =
                ImageData::new(params.dimensions.x as usize, params.dimensions.y as usize);
            *exercise_state =
                ExerciseState::Exercise8(image_data, stars, ray_cache, observer, params);
        }
        9 => {
            let distance = 3.0;
            let vertical_fov_degrees = 120.0;
            let black_hole_radius = 1.5;
            let cache_width: i32 = 1024;
            let (pos, dir, up) = (distance * Vec3::Z, -Vec3::Z, Vec3::Y);
            let params = BlackHoleParams::new(
                IVec2::new(1024, 1024),
                distance,
                vertical_fov_degrees,
                black_hole_radius,
                cache_width,
                pos,
                dir,
                up,
            );
            *exercise_state = ExerciseState::Exercise9(params);
        }
        10 => {
            let distance = 3.0;
            let vertical_fov_degrees = 120.0;
            let black_hole_radius = 1.5;
            let cache_width: i32 = 1024;
            let pos = distance * (Vec3::Z + 0.5 * Vec3::X);

            let (dir, up) = (-pos.normalize(), Vec3::Y);
            let params = BlackHoleParams::new(
                IVec2::new(1024, 1024),
                distance,
                vertical_fov_degrees,
                black_hole_radius,
                cache_width,
                pos,
                dir,
                up,
            );
            let stars = im.as_rgb8().unwrap().as_raw().clone();

            let mut s2 = Vec::new();
            for i in 0..stars.len() / 3 {
                s2.push(stars[3 * i]);
                s2.push(stars[3 * i + 1]);
                s2.push(stars[3 * i + 2]);
                s2.push(255);
            }
            console_log!("Stars len: {}", stars.len());

            let tex = generate_texture_from_u8(&gl.gl, &s2, 1024);
            let program = exercise_10::get_program(gl, &params, &tex);
            *exercise_state = ExerciseState::Exercise10(params, program, tex);
        }
        _ => {}
    }
}

pub struct RenderState {
    gl: RenderContext,
    prev_params: Cell<RenderParams>,
    exercise_state: RefCell<Box<ExerciseState>>,
    im: DynamicImage,
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
        ExerciseState::Exercise9(..) => {}
        ExerciseState::Exercise10(params, program, tex) => {
            gl.delete_texture(&tex);
        }
        _ => {}
    }
}

fn update_exercise_state(
    gl: &RenderContext,
    exercise_state: &mut ExerciseState,
    new_params: &RenderParams,
) {
    match exercise_state {
        ExerciseState::Exercise10(params, _program, stars) => {
            let distance = 3.0;
            let vertical_fov_degrees = 120.0;
            let black_hole_radius = 1.5;
            let cache_width: i32 = 1024;

            let mut pos = params.normalized_pos;
            if new_params.mouse_pos.is_some() {
                let x_angle =
                    std::f32::consts::TAU * (new_params.mouse_pos.unwrap().0 as f32) / 1024.;
                let y_angle =
                    std::f32::consts::PI * (new_params.mouse_pos.unwrap().1 as f32 - 512.) / 1024.;

                pos = distance
                    * (y_angle.cos() * x_angle.cos() * Vec3::Z
                        + y_angle.cos() * x_angle.sin() * Vec3::X
                        + y_angle.sin() * Vec3::Y);
            }

            pos = pos.normalize();
            let dir = -pos;
            let right = Vec3::cross(Vec3::Y, dir).normalize();
            let up = Vec3::cross(right, dir);

            *params = BlackHoleParams::new(
                IVec2::new(1024, 1024),
                distance,
                vertical_fov_degrees,
                black_hole_radius,
                cache_width,
                pos,
                dir,
                up,
            );
        }
        _ => {}
    }
}

fn update_exercise(
    gl: &RenderContext,
    exercise_state: &mut ExerciseState,
    new_params: &RenderParams,
    im: &DynamicImage,
) {
    if exercise_state.index() != new_params.select_index {
        clean_up_exercise(gl, exercise_state);
        init_exercise(gl, exercise_state, new_params.select_index, im);
    }
    update_exercise_state(gl, exercise_state, new_params);
}

fn render_exercise(gl: &RenderContext, exercise_state: &mut ExerciseState) {
    let mut frag;
    match exercise_state {
        ExerciseState::Exercise0 => {
            frag = SourceContext::new(include_str!("shaders/fragment/striped.glsl"));
            gl.draw(None, &frag, &[], None);
        }
        ExerciseState::Exercise1(cm) => {
            exercise_1::exercise_1(gl, cm);
        }
        ExerciseState::Exercise2(cm1, cm2) => {
            exercise_2::exercise_2(gl, cm1, cm2);
        }
        ExerciseState::Exercise3(fb) => {
            exercise_3::exercise_3(gl, fb);
        }
        ExerciseState::Exercise4(fb1, fb2, kernel) => {
            exercise_4::exercise_4(gl, fb1, fb2, kernel);
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

            let pos_seed = Vec2::new(52.912 * time, 11.30 * time);
            let color_seed = Vec3::new(10.10241 * time, 22.958 * time, 25.1 * time);

            frag = SourceContext::new(include_str!("shaders/fragment/psuedo_random.glsl"));
            let pos_seed_uniform = UniformContext::vec2(pos_seed, "pos_seed");
            let color_seed_uniform = UniformContext::vec3(color_seed, "color_seed");
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
        ExerciseState::Exercise8(image_data, stars, ray_cache, observer, params) => {
            exercise_8::exercise_8(gl, image_data, stars, ray_cache, observer, params);
        }
        ExerciseState::Exercise9(params) => {
            exercise_9::exercise_9(gl, params);
        }
        ExerciseState::Exercise10(params, program, stars) => {
            console_log!("Normalized pos: {}", params.normalized_pos);
            for ele in params.uniform_context() {
                ele.add_to_program(gl, program);
            }
            gl.run_program(program, None);
        }
    }
}

const EXERCISE_COUNT: u32 = 11;
const RENDER_TEXTURE_DEFAULT: &str = include_str!("shaders/fragment/render_texture.glsl");
impl RenderState {
    fn render(&self, params: &RenderParams) -> Result<(), JsValue> {
        console_log!("params: {:?}", params);
        let gl = &self.gl;

        update_exercise(gl, &mut *self.exercise_state.borrow_mut(), params, &self.im);
        render_exercise(gl, &mut *self.exercise_state.borrow_mut());
        self.prev_params.set(*params);
        Ok(())
    }
}

impl RenderState {
    pub fn new(width: u32, height: u32, im: DynamicImage) -> Result<RenderState, JsValue> {
        let gl = RenderContext::new(width, height);

        Ok(RenderState {
            gl,
            prev_params: Cell::default(),
            exercise_state: RefCell::default(),
            im,
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

#[wasm_bindgen]
pub async fn fetch_url_binary(url: String) -> Result<Uint8Array, JsValue> {
    let window = web_sys::window().unwrap(); // Browser window
    let promise = JsFuture::from(window.fetch_with_str(&url)); // File fetch promise
    let result = promise.await?; // Await fulfillment of fetch
    let response: web_sys::Response = result.dyn_into().unwrap(); // Type casting
    let image_data = JsFuture::from(response.array_buffer()?).await?; // Get text
    Ok(Uint8Array::new(&image_data))
}

const image_url: &str = "http://localhost:8080/starmap_2020_4k_gal_print.jpg";

fn to_image(u8: Uint8Array) -> DynamicImage {
    image::load_from_memory_with_format(&u8.to_vec(), image::ImageFormat::Jpeg).unwrap()
}

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let x = fetch_url_binary(image_url.to_string()).await?;
    let im = to_image(x);
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
    let renderer = Rc::new(RenderState::new(1024, 1024, im)?);
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
