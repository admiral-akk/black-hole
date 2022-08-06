extern crate cfg_if;
extern crate wasm_bindgen;

mod exercises;
mod framework;
use exercises::exercise_10;

use framework::program_context::ProgramContext;
use framework::source_context::SourceContext;
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

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use cfg_if::cfg_if;
use framework::render_context::RenderContext;
use framework::uniform_context::UniformContext;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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

impl Default for BlackHoleParams {
    fn default() -> Self {
        let distance = 17.0;
        let vertical_fov_degrees = 50.0;
        let black_hole_radius = 1.5;
        let cache_width: i32 = 1024;
        let pos = distance * (Vec3::Z + 0.5 * Vec3::X);

        let (dir, up) = (-pos.normalize(), Vec3::Y);
        BlackHoleParams::new(
            IVec2::new(1024, 1024),
            distance,
            vertical_fov_degrees,
            black_hole_radius,
            cache_width,
            pos,
            dir,
            up,
            0.0,
        )
    }
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
    pub fn update(&mut self, render_params: &RenderParams) {
        self.distance = f32::clamp(
            (17.0 + render_params.mouse_scroll / 100.0) as f32,
            5.0,
            20.0,
        );

        let mut pos = self.normalized_pos;
        if render_params.mouse_pos.is_some() {
            let x_angle =
                std::f32::consts::TAU * (render_params.mouse_pos.unwrap().0 as f32) / 1024.;
            let y_angle =
                std::f32::consts::PI * (render_params.mouse_pos.unwrap().1 as f32 - 512.) / 1024.;

            pos = self.distance
                * (y_angle.cos() * x_angle.cos() * Vec3::Z
                    + y_angle.cos() * x_angle.sin() * Vec3::X
                    + y_angle.sin() * Vec3::Y);
        }

        self.normalized_pos = pos.normalize();
        self.normalized_dir = -self.normalized_pos;
        let right = Vec3::cross(Vec3::Y, self.normalized_dir).normalize();
        self.normalized_up = Vec3::cross(right, self.normalized_dir);
        let observer_quat = Quat::from_rotation_arc(pos.normalize(), -Vec3::Z);
        let euler = Quat::to_euler(observer_quat, glam::EulerRot::XYZ);
        self.observer_mat = Mat3::from_euler(glam::EulerRot::XYZ, euler.0, euler.1, euler.2);
        self.time_s = render_params.seconds_since_start;
    }
}

fn compile_shader_program(
    gl: &RenderContext,
    frag: &SourceContext,
    images: &ImageCache,
) -> ProgramContext {
    gl.get_program(None, frag, &images.textures)
}

pub struct RenderState {
    gl: RenderContext,
    source: SourceContext,
    program: ProgramContext,
    black_hole_params: BlackHoleParams,
    images: ImageCache,
}

fn update_params(black_hole_params: &mut BlackHoleParams, new_params: &RenderParams) {
    let distance = f32::clamp((17.0 + new_params.mouse_scroll / 100.0) as f32, 5.0, 20.0);
    let vertical_fov_degrees = 50.0;
    let black_hole_radius = 1.5;
    let cache_width: i32 = 1024;

    let mut pos = black_hole_params.normalized_pos;
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

    *black_hole_params = BlackHoleParams::new(
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
fn render(render_state: &mut RenderState, params: &RenderParams) -> Result<(), JsValue> {
    console_log!("params: {:?}", params);
    let gl = &render_state.gl;
    update_params(&mut render_state.black_hole_params, params);
    for ele in render_state.black_hole_params.uniform_context() {
        ele.add_to_program(gl, &mut render_state.program);
    }
    gl.run_program(&render_state.program, None);
    Ok(())
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

impl RenderState {
    pub async fn new(width: u32, height: u32) -> Result<RenderState, JsValue> {
        let gl = RenderContext::new(width, height);

        let images = ImageCache::new(&gl).await?;
        let source = SourceContext::new(include_str!(
            "exercises/shaders/fragment/black_hole/complete.glsl"
        ));
        let black_hole_params = BlackHoleParams::default();
        let program = compile_shader_program(&gl, &source, &images);

        Ok(RenderState {
            gl,
            source,
            black_hole_params,
            program,
            images,
        })
    }

    pub fn update_disc_shader(&mut self, shader_func: &str) {
        let shader_code = match shader_func.is_empty() {
            true => DEFAULT_DISC_FUNC,
            false => shader_func,
        }
        .replace("\n", "");
        self.source.add_code(shader_code);
        self.program = compile_shader_program(&self.gl, &self.source, &self.images);
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderParams {
    pub seconds_since_start: f32,
    pub mouse_pos: Option<(i32, i32)>,
    pub mouse_scroll: f64,
}

impl RenderParams {
    pub fn update_time(&mut self, seconds_since_start: f32) {
        self.seconds_since_start = seconds_since_start;
    }

    pub fn update_mouse_pos(&mut self, mouse_pos: Option<(i32, i32)>) {
        self.mouse_pos = mouse_pos;
    }

    pub fn update_mouse_scroll(&mut self, delta: f64) {
        self.mouse_scroll += delta;
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

pub async fn fetch_rgb_texture(gl: &RenderContext, url: &str, name: &str) -> UniformContext {
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
const RAY_CACHE_2_URL: &str = "http://localhost:8080/ray_cache.txt";
const FIXED_DISTANCE_ANGLE_CACHE_URL: &str =
    "http://localhost:8080/fixed_distance_distance_cache.txt";

fn to_image(u8: Uint8Array) -> DynamicImage {
    image::load_from_memory_with_format(&u8.to_vec(), image::ImageFormat::Jpeg).unwrap()
}
pub struct ImageCache {
    textures: [UniformContext; 7],
}

impl ImageCache {
    pub async fn new(gl: &RenderContext) -> Result<ImageCache, JsValue> {
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
            for x in 0..ray_width {
                let final_dir = cache.cache[x].final_dir;
                ray_vec_2.push(final_dir[0]);
                ray_vec_2.push(final_dir[2]);
            }
        }
        let ray_cache_tex =
            generate_texture_from_f32(&gl.gl, &ray_vec_2, ray_width as i32, Format::RG);
        let ray_cache_tex = UniformContext::new_from_allocated_val(
            ray_cache_tex,
            "cache",
            ray_width as i32,
            ray_height as i32,
        );
        let z_max_cache_tex =
            generate_texture_from_f32(&gl.gl, &z_max_vec, ray_height as i32, Format::R);
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
            for c in &cache.z_to_distance {
                v.push(*c as f32);
            }
        }
        console_log!("min_z: {:?}", min_z);
        console_log!("min_z length: {:?}", min_z.len());

        console_log!("angle_cache: {:?}", v);
        console_log!("angle_cache length: {:?}", v.len());

        let angle_height = (min_z.len() / 2) as i32;
        let angle_width = v.len() as i32 / angle_height;

        let angle_cache_tex = generate_texture_from_f32(&gl.gl, &v, angle_width, Format::R);
        let angle_cache_tex = UniformContext::new_from_allocated_val(
            angle_cache_tex,
            "angle_cache",
            angle_width as i32,
            angle_height as i32,
        );
        let angle_min_z_tex = generate_texture_from_f32(&gl.gl, &min_z, angle_height, Format::RG);

        let angle_min_z_tex = UniformContext::new_from_allocated_val(
            angle_min_z_tex,
            "angle_z_max_cache",
            angle_height as i32,
            1 as i32,
        );
        Ok(ImageCache {
            textures: [
                galaxy_tex,
                stars_tex,
                constellations_tex,
                ray_cache_tex,
                z_max_cache_tex,
                angle_cache_tex,
                angle_min_z_tex,
            ],
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
    let shader_text_box = document
        .get_element_by_id("shader")
        .unwrap()
        .dyn_into::<web_sys::HtmlTextAreaElement>()?;
    let compile_button = document
        .get_element_by_id("recompile")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()?;

    let start_time = Rc::new(Cell::new(SystemTime::now()));
    let params = Rc::new(RefCell::new(RenderParams::default()));
    let render_state = Rc::new(RefCell::new(RenderState::new(1024, 1024).await?));
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            params.borrow_mut().mouse_pos = Some((event.offset_x(), event.offset_y()));
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
            params.borrow_mut().mouse_pos = None;
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseleave", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::WheelEvent| {
            params.borrow_mut().mouse_scroll += _event.delta_y();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    let render_func: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = render_func.clone();
    {
        let render_state = render_state.clone();
        let start_time = start_time.clone();
        let params = params.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let seconds_since_start = SystemTime::now()
                .duration_since(start_time.get())
                .unwrap()
                .as_secs_f32();
            params.borrow_mut().seconds_since_start = seconds_since_start;
            render(&mut render_state.borrow_mut(), &params.borrow()).unwrap();
            requestAnimationFrame(render_func.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
        window()
            .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
    }

    Ok(())
}
