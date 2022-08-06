extern crate cfg_if;
extern crate wasm_bindgen;

mod exercises;
mod framework;
use exercises::exercise_10;

use framework::program_context::ProgramContext;
use framework::texture_utils::generate_rgb_texture_from_u8;
use framework::texture_utils::generate_texture_from_f32;
use framework::texture_utils::Format;
use glam::IVec2;
use glam::Mat3;
use glam::Quat;
use glam::Vec3;

use image::DynamicImage;
use js_sys::Uint8Array;
use path_integration::cache::fixed_distance_distance_cache::FixedDistanceDistanceCache;
use path_integration::cache::ray_cache::RayCache as PathRayCache;

use wasm_bindgen_futures::JsFuture;
use wasm_timer::SystemTime;

use web_sys::WebGlTexture;

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use cfg_if::cfg_if;
use framework::render_context::RenderContext;
use framework::uniform_context::UniformContext;
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

struct ExerciseState {
    pub params: BlackHoleParams,
    pub program: ProgramContext,
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
    pub time_s: f32,
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
        time_s: f32,
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
            time_s,
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
        v.push(UniformContext::f32(self.time_s, "time_s"));
        v
    }
}

fn init_exercise(gl: &RenderContext, images: &ImageCache) -> ExerciseState {
    let distance = 17.0;
    let vertical_fov_degrees = 50.0;
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
        0.0,
    );
    let program = exercise_10::get_program(gl, &params, images);

    ExerciseState { params, program }
}

pub struct RenderState<'a> {
    gl: RenderContext,
    prev_params: Cell<RenderParams>,
    exercise_state: RefCell<Box<ExerciseState>>,
    images: ImageCache<'a>,
}

fn update_params(exercise_state: &mut ExerciseState, new_params: &RenderParams) {
    let distance = f32::clamp((17.0 + new_params.mouse_scroll / 100.0) as f32, 5.0, 20.0);
    let vertical_fov_degrees = 50.0;
    let black_hole_radius = 1.5;
    let cache_width: i32 = 1024;

    let mut pos = exercise_state.params.normalized_pos;
    if new_params.mouse_pos.is_some() {
        let x_angle = std::f32::consts::TAU * (new_params.mouse_pos.unwrap().0 as f32) / 1024.;
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

    exercise_state.params = BlackHoleParams::new(
        IVec2::new(1024, 1024),
        distance,
        vertical_fov_degrees,
        black_hole_radius,
        cache_width,
        pos,
        dir,
        up,
        new_params.seconds_since_start,
    );
}
fn render_exercise(gl: &RenderContext, exercise_state: &mut ExerciseState) {
    for ele in exercise_state.params.uniform_context() {
        ele.add_to_program(gl, &mut exercise_state.program);
    }
    gl.run_program(&exercise_state.program, None);
}

impl<'a> RenderState<'a> {
    fn render(&self, params: &RenderParams) -> Result<(), JsValue> {
        console_log!("params: {:?}", params);
        let gl = &self.gl;

        update_params(&mut *self.exercise_state.borrow_mut(), params);
        render_exercise(gl, &mut *self.exercise_state.borrow_mut());
        self.prev_params.set(*params);
        Ok(())
    }
}

const DEFAULT_DISC_FUNC: &str = "float random(in vec2 _st) {
    return fract(sin(dot(_st.xy, vec2(312.12,1.*TAU)))*42.5453123);
}

// Based on Morgan McGuire @morgan3d
// https://www.shadertoy.com/view/4dS3Wd
float noise(in vec2 _st) {
    vec2 i = floor(_st);
    vec2 f = fract(_st);
    
    // Four corners in 2D of a tile
    float a = random(i);
    float b = random(i + vec2(1.0, 0.0));
    float c = random(i + vec2(0.0, 1.0));
    float d = random(i + vec2(1.0, 1.0));
    
    vec2 u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) +
    (c - a)* u.y * (1.0 - u.x) +
    (d - b) * u.x * u.y;
}

vec4 disc_color(float dist_01,float theta_01){
    float n = noise(vec2(dist_01,theta_01)*vec2(42.3,1.));
    return vec4(n,n,n,1.0);
    float offset=5.*TAU*dist_01+n+time_s;
    float white=clamp((.5+sin(theta_01*TAU+offset)),0.,1.);
    return vec4(n,n,n,1.0);
}";

impl<'a> RenderState<'a> {
    pub async fn new(width: u32, height: u32) -> Result<RenderState<'a>, JsValue> {
        let gl = RenderContext::new(width, height);

        let images = ImageCache::new(&gl).await?;
        let exercise_state = init_exercise(&gl, &images);
        Ok(RenderState {
            gl,
            prev_params: Cell::default(),
            exercise_state: RefCell::new(Box::new(exercise_state)),
            images,
        })
    }

    pub fn update_disc_shader(shader_func: &str) {
        if shader_func.is_empty() {
        } else {
        }
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
    pub mouse_scroll: f64,
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

    pub fn update_mouse_scroll(&self, delta: f64) -> RenderParams {
        let mut c = self.clone();
        c.mouse_scroll += delta;
        c
    }
}

pub async fn fetch_url_binary(url: String) -> Result<Uint8Array, JsValue> {
    let window = web_sys::window().unwrap(); // Browser window
    let promise = JsFuture::from(window.fetch_with_str(&url)); // File fetch promise
    let result = promise.await?; // Await fulfillment of fetch
    let response: web_sys::Response = result.dyn_into().unwrap(); // Type casting
    let image_data = JsFuture::from(response.array_buffer()?).await?; // Get text
    Ok(Uint8Array::new(&image_data))
}

pub async fn fetch_rgb_texture<'a>(
    gl: &RenderContext,
    url: &str,
    name: &str,
) -> UniformContext<'a> {
    let image = to_image(fetch_url_binary(url.to_string()).await.unwrap());
    let image_tex = generate_texture_from_u8(
        &gl.gl,
        &image.as_rgb8().unwrap().as_raw(),
        image.width() as i32,
        Format::RGB,
    );
    UniformContext::new_from_allocated_val(
        image_tex,
        name,
        image.width() as i32,
        image.height() as i32,
    )
}

const GALAXY_URL: &str = "http://localhost:8080/galaxy.jpg";
const CONSTELLATIONS_URL: &str = "http://localhost:8080/constellations.jpg";
const STARS_URL: &str = "http://localhost:8080/stars.jpg";
const RAY_CACHE_URL: &str = "http://localhost:8080/cache.png";
const Z_MAX_CACHE_URL: &str = "http://localhost:8080/z_max_cache.png";
const RAY_CACHE_2_URL: &str = "http://localhost:8080/ray_cache.txt";
const ANGLE_CACHE_URL: &str = "http://localhost:8080/angle_cache.txt";
const FIXED_DISTANCE_ANGLE_CACHE_URL: &str =
    "http://localhost:8080/fixed_distance_distance_cache.txt";

fn to_image(u8: Uint8Array) -> DynamicImage {
    image::load_from_memory_with_format(&u8.to_vec(), image::ImageFormat::Jpeg).unwrap()
}
pub struct ImageCache<'a> {
    galaxy_tex: UniformContext<'a>,
    stars_tex: UniformContext<'a>,
    constellations_tex: UniformContext<'a>,
    ray_cache_tex: UniformContext<'a>,
    max_z_tex: UniformContext<'a>,
    angle_cache_tex: UniformContext<'a>,
    angle_min_z_tex: UniformContext<'a>,
}

impl<'a> ImageCache<'a> {
    fn to_rgba(rgb: &Vec<u8>) -> Vec<u8> {
        let mut rgba = Vec::new();
        for i in 0..rgb.len() / 3 {
            rgba.push(rgb[3 * i]);
            rgba.push(rgb[3 * i + 1]);
            rgba.push(rgb[3 * i + 2]);
            rgba.push(255);
        }
        rgba
    }

    pub async fn new(gl: &RenderContext) -> Result<ImageCache<'a>, JsValue> {
        let galaxy_tex = fetch_rgb_texture(gl, GALAXY_URL, "galaxy").await;
        let stars_tex = fetch_rgb_texture(gl, STARS_URL, "stars").await;
        let constellations_tex = fetch_rgb_texture(gl, CONSTELLATIONS_URL, "constellations").await;

        let ray_cache_2 = fetch_url_binary(RAY_CACHE_2_URL.to_string()).await?;
        let ray_cache_2 = serde_json::from_slice::<PathRayCache>(&ray_cache_2.to_vec()).unwrap();
        console_log!("Deserialized: {:?}", ray_cache_2);
        let (ray_width, ray_height) = (ray_cache_2.caches[0].cache.len(), ray_cache_2.caches.len());
        let mut ray_vec_2 = Vec::new();
        let mut z_max_vec = Vec::new();
        for y in 0..ray_height {
            let cache = &ray_cache_2.caches[y];
            z_max_vec.push(cache.max_z);
            z_max_vec.push(cache.max_z);
            z_max_vec.push(cache.max_z);
            z_max_vec.push(cache.max_z);
            for x in 0..ray_width {
                let final_dir = cache.cache[x].final_dir;
                ray_vec_2.push(final_dir[0]);
                ray_vec_2.push(final_dir[1]);
                ray_vec_2.push(final_dir[2]);
                ray_vec_2.push(1.0);
            }
        }
        let ray_cache_tex =
            generate_texture_from_f32(&gl.gl, &ray_vec_2, ray_width as i32, Format::RGBA);
        let ray_cache_tex = UniformContext::new_from_allocated_val(
            ray_cache_tex,
            "cache",
            ray_width as i32,
            ray_height as i32,
        );
        let z_max_cache_tex =
            generate_texture_from_f32(&gl.gl, &z_max_vec, ray_height as i32, Format::RGBA);
        let z_max_cache_tex = UniformContext::new_from_allocated_val(
            z_max_cache_tex,
            "z_max_cache",
            ray_height as i32,
            1 as i32,
        );

        let angle_cache = fetch_url_binary(FIXED_DISTANCE_ANGLE_CACHE_URL.to_string()).await?;
        let angle_cache =
            serde_json::from_slice::<FixedDistanceDistanceCache>(&angle_cache.to_vec()).unwrap();

        let mut v = Vec::new();
        let mut min_z = Vec::new();

        for x in 0..angle_cache.angle_to_z_to_distance.len() {
            let cache = &angle_cache.angle_to_z_to_distance[x];
            min_z.push(cache.z_bounds.0 as f32);
            min_z.push(cache.z_bounds.1 as f32);
            min_z.push(0.);
            min_z.push(1.);
            for c in &cache.z_to_distance {
                v.push(*c as f32);
                v.push(0.);
                v.push(0.);
                v.push(1.);
            }
        }
        console_log!("min_z: {:?}", min_z);
        console_log!("min_z length: {:?}", min_z.len());

        console_log!("angle_cache: {:?}", v);
        console_log!("angle_cache length: {:?}", v.len());

        let angle_height = (min_z.len() / 4) as i32;
        let angle_width = (v.len() / 4) as i32 / angle_height;

        let angle_cache_tex = generate_texture_from_f32(&gl.gl, &v, angle_width, Format::RGBA);
        let angle_cache_tex = UniformContext::new_from_allocated_val(
            angle_cache_tex,
            "angle_cache",
            angle_width as i32,
            angle_height as i32,
        );
        let angle_min_z_tex = generate_texture_from_f32(&gl.gl, &min_z, angle_height, Format::RGBA);

        let angle_min_z_tex = UniformContext::new_from_allocated_val(
            angle_min_z_tex,
            "angle_z_max_cache",
            angle_height as i32,
            1 as i32,
        );
        Ok(ImageCache {
            galaxy_tex,
            stars_tex,
            constellations_tex,
            ray_cache_tex,
            max_z_tex: z_max_cache_tex,
            angle_cache_tex,
            angle_min_z_tex,
        })
    }
}

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let document = document();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    let start_time = Rc::new(Cell::new(SystemTime::now()));
    let params = Rc::new(Cell::new(RenderParams::default()));
    let renderer = Rc::new(RenderState::new(1024, 1024).await?);
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
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::WheelEvent| {
            params.set(params.get().update_mouse_scroll(_event.delta_y()));
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    let render_func: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
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
