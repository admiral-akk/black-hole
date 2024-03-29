extern crate cfg_if;
extern crate wasm_bindgen;

mod exercises;
mod framework;

use framework::program_context::ProgramContext;
use framework::source_context::SourceContext;
use framework::texture_utils::generate_3d_texture_from_f32;
use framework::texture_utils::generate_texture_from_f32;
use framework::texture_utils::Format;
use generate_artifacts::black_hole_cache::BlackHoleCache;
use glam::IVec2;
use glam::Mat3;
use glam::Quat;
use glam::Vec2;
use glam::Vec3;

use image::DynamicImage;
use js_sys::Uint8Array;

use wasm_bindgen_futures::JsFuture;
use wasm_timer::SystemTime;
use web_sys::MouseEvent;
use web_sys::Touch;
use web_sys::TouchEvent;
use web_sys::Url;
use web_sys::WheelEvent;

use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts::PI;
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
        let vertical_fov_degrees = 90.0;
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
            2. * f32::tan(PI * self.vertical_fov_degrees / 360.),
            "vertical_fov_magnitude",
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
        v.push(UniformContext::mat3x3(
            self.observer_mat.inverse(),
            "inv_observer_mat",
        ));
        v.push(UniformContext::f32(self.time_s, "time_s"));
        v
    }
    pub fn update(&mut self, render_params: &RenderParams) {
        self.distance = 27. * render_params.ui_params.zoom_sum as f32 + 3.0;
        let mut pos = self.normalized_pos;
        let x_angle = std::f32::consts::TAU * (render_params.ui_params.mouse_position.0 as f32);
        let y_angle =
            std::f32::consts::PI * (render_params.ui_params.mouse_position.1 as f32 - 0.5);

        pos = self.distance
            * (y_angle.cos() * x_angle.cos() * Vec3::Z
                + y_angle.cos() * x_angle.sin() * Vec3::X
                + y_angle.sin() * Vec3::Y);

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

pub struct RenderState {
    gl: RenderContext,
    source: SourceContext,
    program: ProgramContext,
    black_hole_params: BlackHoleParams,
    images: ImageCache,
}

fn update_params(black_hole_params: &mut BlackHoleParams, new_params: &RenderParams) {
    let target_distance = 27. * new_params.ui_params.zoom_sum as f32 + 3.0;
    let current_distance = black_hole_params.distance;
    let distance = 0.9 * current_distance + 0.1 * target_distance;
    let vertical_fov_degrees = 90.0;
    let black_hole_radius = 1.5;
    let cache_width: i32 = 1024;

    let mut pos = black_hole_params.normalized_pos;
    let x_angle = std::f32::consts::TAU * (new_params.ui_params.mouse_position.0 as f32);
    let y_angle = std::f32::consts::PI * (new_params.ui_params.mouse_position.1 as f32 - 0.5);

    pos = distance
        * (y_angle.cos() * x_angle.cos() * Vec3::Z
            + y_angle.cos() * x_angle.sin() * Vec3::X
            + y_angle.sin() * Vec3::Y);

    pos = pos.normalize();
    let dir = -pos;
    let right = Vec3::cross(Vec3::Y, dir).normalize();
    let up = Vec3::cross(right, dir);

    *black_hole_params = BlackHoleParams::new(
        IVec2::new(
            new_params.dimensions.0 as i32,
            new_params.dimensions.1 as i32,
        ),
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
    let gl = &render_state.gl;
    update_params(&mut render_state.black_hole_params, params);
    gl.update_dimensions(params.dimensions.0, params.dimensions.1);
    for ele in render_state.black_hole_params.uniform_context() {
        ele.add_to_program(gl, &mut render_state.program);
    }
    gl.run_program(&render_state.program, None);
    Ok(())
}

const DEFAULT_DISC_FUNC: &str = include_str!("default_disc_color.glsl");

impl RenderState {
    pub async fn new(width: u32, height: u32) -> Result<RenderState, JsValue> {
        let gl = RenderContext::new(width, height);

        let images = ImageCache::new(&gl).await?;
        let mut source = SourceContext::new(include_str!(
            "exercises/shaders/fragment/black_hole/complete.glsl"
        ));
        source.add_code(DEFAULT_DISC_FUNC.to_string());
        let black_hole_params = BlackHoleParams::default();
        let program = gl.get_program(&source, &images.textures);

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
        .to_string();
        self.source.add_code(shader_code);
        self.program = self.gl.get_program(&self.source, &self.images.textures);
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

#[derive(Debug, Default)]
pub struct RenderParams {
    pub seconds_since_start: f32,
    pub dimensions: (u32, u32),
    pub ui_params: UIParams,
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
    console_log!("image:{}\ndim: {:?}", name, (image.width(), image.height()));
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
const GALAXY_URL: &str = "galaxy.jpg";
const CONSTELLATIONS_URL: &str = "constellations.jpg";
const STARS_URL: &str = "stars.jpg";
const COMBINED_URL: &str = "combined.jpg";
const BLACK_HOLE_CACHE_URL: &str = "black_hole_cache.txt";
const NOISE_URL: &str = "noise.jpg";

fn to_image(u8: Uint8Array) -> DynamicImage {
    image::load_from_memory_with_format(&u8.to_vec(), image::ImageFormat::Jpeg).unwrap()
}
fn to_image_png(u8: Uint8Array) -> DynamicImage {
    image::load_from_memory_with_format(&u8.to_vec(), image::ImageFormat::Png).unwrap()
}
pub struct ImageCache {
    textures: Vec<UniformContext>,
}

impl ImageCache {
    pub async fn new(gl: &RenderContext) -> Result<ImageCache, JsValue> {
        let galaxy_tex = fetch_rgb_texture(gl, GALAXY_URL, "galaxy").await;
        let stars_tex = fetch_rgb_texture(gl, STARS_URL, "stars").await;
        let combined_tex = fetch_rgb_texture(gl, COMBINED_URL, "combined").await;
        let constellations_tex = fetch_rgb_texture(gl, CONSTELLATIONS_URL, "constellations").await;
        let disc_noise_tex = fetch_rgb_texture(gl, NOISE_URL, "disc_noise").await;

        let black_hole_cache = fetch_url_binary(BLACK_HOLE_CACHE_URL.to_string()).await?;
        let black_hole_cache =
            serde_json::from_slice::<BlackHoleCache>(&black_hole_cache.to_vec()).unwrap();
        let direction_cache = black_hole_cache.direction_cache;
        let distance_cache = black_hole_cache.distance_cache;

        let disc_dim = UniformContext::vec2(
            Vec2::new(
                distance_cache.disc_bounds.0 as f32,
                distance_cache.disc_bounds.1 as f32,
            ),
            "disc_dim",
        );

        let mut distance_cache_vec = Vec::new();
        let mut z_bounds_vec = Vec::new();
        let mut min_z_vec = Vec::new();

        let min_angle = UniformContext::f32(
            distance_cache.distance_angle_to_z_to_distance[0].min_angle as f32,
            "min_angle",
        );
        let distance_bounds = UniformContext::vec2(
            Vec2::new(
                distance_cache.distance_bounds.0 as f32,
                distance_cache.distance_bounds.1 as f32,
            ),
            "distance_bounds",
        );
        for fixed_distance in distance_cache.distance_angle_to_z_to_distance {
            min_z_vec.push(fixed_distance.min_z);
            for fixed_angle in fixed_distance.angle_to_z_to_distance {
                let z_bounds = fixed_angle.z_bounds;
                z_bounds_vec.push(z_bounds.0 as f32);
                z_bounds_vec.push(z_bounds.1 as f32);
                for fixed_z in fixed_angle.z_to_distance {
                    distance_cache_vec.push(fixed_z as f32);
                }
            }
        }

        let (width, height, depth) = distance_cache.cache_size;
        let distance_cache_tex = generate_3d_texture_from_f32(
            &gl.gl,
            &distance_cache_vec,
            width as i32,
            height as i32,
            depth as i32,
            Format::R,
        );
        let distance_cache_tex = UniformContext::texture_3d(
            distance_cache_tex,
            "distance_cache_tex",
            width as i32,
            height as i32,
            depth as i32,
        );
        let distance_cache_z_bounds =
            generate_texture_from_f32(&gl.gl, &z_bounds_vec, height as i32, Format::RG);
        let distance_cache_z_bounds = UniformContext::new_from_allocated_val(
            distance_cache_z_bounds,
            "distance_cache_z_bounds",
            height as i32,
            depth as i32,
        );
        let mut direction_vec = Vec::new();
        let mut direction_z_max_vec = Vec::new();

        let direction_height = direction_cache.distance_angle_to_z_to_distance.len();
        let direction_width = direction_cache.distance_angle_to_z_to_distance[0]
            .z_to_final_dir
            .len();
        for y in 0..direction_height {
            let cache = &direction_cache.distance_angle_to_z_to_distance[y];
            direction_z_max_vec.push(cache.min_z as f32);
            direction_z_max_vec.push(cache.max_z as f32);
            for x in 0..direction_width {
                let final_dir = cache.z_to_final_dir[x].1;
                direction_vec.push(final_dir.0 as f32);
                direction_vec.push(final_dir.1 as f32);
            }
        }
        let direction_tex =
            generate_texture_from_f32(&gl.gl, &direction_vec, direction_width as i32, Format::RG);
        let direction_tex = UniformContext::new_from_allocated_val(
            direction_tex,
            "direction_cache",
            direction_width as i32,
            direction_height as i32,
        );
        let direction_z_max_tex = generate_texture_from_f32(
            &gl.gl,
            &direction_z_max_vec,
            direction_height as i32,
            Format::RG,
        );
        let direction_z_max_tex = UniformContext::new_from_allocated_val(
            direction_z_max_tex,
            "direction_z_max_cache",
            direction_height as i32,
            1 as i32,
        );

        Ok(ImageCache {
            textures: Vec::from([
                galaxy_tex,
                stars_tex,
                combined_tex,
                constellations_tex,
                disc_noise_tex,
                disc_dim,
                distance_cache_z_bounds,
                distance_cache_tex,
                min_angle,
                distance_bounds,
                direction_tex,
                direction_z_max_tex,
            ]),
        })
    }
}

#[derive(Debug)]
pub struct UIParams {
    pub seconds_since_start: f32,
    pub pixel_dimensions: (u32, u32),
    pub css_dimensions: (u32, u32),
    pub touch_id_to_last_pos: HashMap<i32, IVec2>,
    pub curr_touch_center: IVec2,
    pub mouse_position: (f64, f64),
    pub touch_diff: Option<f64>,
    pub zoom_sum: f64,
}

impl Default for UIParams {
    fn default() -> Self {
        let window = window();
        let pixel_ratio = window.device_pixel_ratio();
        let (width, height) = (
            window.inner_width().unwrap().as_f64().unwrap(),
            window.inner_height().unwrap().as_f64().unwrap(),
        );
        Self {
            seconds_since_start: Default::default(),
            pixel_dimensions: ((width * pixel_ratio) as u32, (height * pixel_ratio) as u32),
            css_dimensions: (width as u32, height as u32),
            touch_id_to_last_pos: HashMap::new(),
            curr_touch_center: Default::default(),
            mouse_position: (0.5, 0.5),
            touch_diff: Default::default(),
            zoom_sum: 0.5,
        }
    }
}
impl UIParams {
    fn move_view(&mut self, movement: (i32, i32)) {
        let delta = (
            (movement.0 as f64 / self.css_dimensions.0 as f64),
            (movement.1 as f64 / self.css_dimensions.1 as f64),
        );
        self.mouse_position.0 += delta.0;
        self.mouse_position.1 = (self.mouse_position.1 + delta.1).clamp(0., 1.);
    }

    fn zoom_view(&mut self, delta: f64) {
        self.zoom_sum = (self.zoom_sum + delta).clamp(0., 1.);
    }

    pub fn mouse_move(&mut self, move_event: &MouseEvent) {
        if move_event.buttons() & 1 == 0 {
            return;
        }

        self.move_view((move_event.movement_x(), move_event.movement_y()));
    }
    pub fn mouse_scroll(&mut self, event: &WheelEvent) {
        self.zoom_view(event.delta_y() / 3000.);
    }

    pub fn handle_touch(&mut self, event: &TouchEvent) {
        let touch_list: Vec<Touch> = (0..event.touches().length())
            .map(|i| event.touches().get(i).unwrap())
            .collect();

        let mut changes = false;
        // Check whether any touches were added
        for touch in &touch_list {
            if !self.touch_id_to_last_pos.contains_key(&touch.identifier()) {
                self.touch_id_to_last_pos.insert(
                    touch.identifier(),
                    IVec2::new(touch.client_x(), touch.client_y()),
                );
                changes = true;
            }
        }

        // Check whether any touches were removed
        let existing_touch_ids: Vec<i32> = self.touch_id_to_last_pos.keys().map(|k| *k).collect();
        for touch_id in existing_touch_ids {
            if touch_list
                .iter()
                .find(|touch| touch.identifier() == touch_id)
                .is_none()
            {
                self.touch_id_to_last_pos.remove(&touch_id);
                changes = true;
            }
        }

        // Panning
        let new_center = self
            .touch_id_to_last_pos
            .values()
            .map(|v| *v)
            .fold(IVec2::ZERO, |a, b| a + b);

        // adding/removing touches doesn't change the view
        let center_diff = new_center - self.curr_touch_center;
        if !changes {
            // Pan
            self.move_view((center_diff.x, center_diff.y));

            // Zoom
            if self.touch_id_to_last_pos.len() >= 2 {
                let old_average_dist = self
                    .touch_id_to_last_pos
                    .values()
                    .map(|touch_pos| (*touch_pos - new_center).as_dvec2().length())
                    .fold(0., |a, b| a + b);
                let new_average_dist = touch_list
                    .iter()
                    .map(|touch| IVec2::new(touch.client_x(), touch.client_y()))
                    .map(|touch_pos| (touch_pos - new_center).as_dvec2().length())
                    .fold(0., |a, b| a + b);

                self.zoom_view(
                    -(new_average_dist - old_average_dist)
                        / (100. * self.touch_id_to_last_pos.len() as f64),
                );
            }
        }

        // Update map positions
        for touch in &touch_list {
            self.touch_id_to_last_pos.insert(
                touch.identifier(),
                IVec2::new(touch.client_x(), touch.client_y()),
            );
        }
        self.curr_touch_center = new_center;
    }
}

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    let d = document();
    let canvas = d
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let shader_text_box = d
        .get_element_by_id("shader")
        .unwrap()
        .dyn_into::<web_sys::HtmlTextAreaElement>()?;
    shader_text_box.set_value(DEFAULT_DISC_FUNC);
    let compile_button = d
        .get_element_by_id("recompile")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()?;

    let fps_counter = d
        .get_element_by_id("fps-counter")
        .unwrap()
        .dyn_into::<web_sys::HtmlDivElement>()?;

    let canvas_ref = Rc::new(RefCell::new(
        d.get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()?,
    ));
    let query_params = Url::new(&document().url().unwrap())
        .unwrap()
        .search_params();
    let q_debug = query_params.get("debug");
    if q_debug.is_some() {
        // unhide
        compile_button.set_hidden(false);
        shader_text_box.set_hidden(false);
        // enabled scrolling down?
        d.body().unwrap().remove_attribute("overflow-x").unwrap();
    }
    let last_200_frame_times = Rc::new(RefCell::new(Vec::from([0.0_f32])));
    let start_time = Rc::new(RefCell::new(SystemTime::now()));
    let params = Rc::new(RefCell::new(RenderParams::default()));
    let render_state = Rc::new(RefCell::new(RenderState::new(1024, 1024).await?));
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            params.borrow_mut().ui_params.mouse_move(&event);
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
            params.borrow_mut().ui_params.mouse_scroll(&event);
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::TouchEvent| {
            params.borrow_mut().ui_params.handle_touch(&event);
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::TouchEvent| {
            params.borrow_mut().ui_params.handle_touch(&event);
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::TouchEvent| {
            params.borrow_mut().ui_params.handle_touch(&event);
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let params = params.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::TouchEvent| {
            params.borrow_mut().ui_params.handle_touch(&event);
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("touchcancel", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    {
        let start_time = start_time.clone();
        let render_state = render_state.clone();
        let last_200_frame_times = last_200_frame_times.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
            *start_time.borrow_mut() = SystemTime::now();
            last_200_frame_times.borrow_mut().clear();
            render_state
                .borrow_mut()
                .update_disc_shader(&shader_text_box.value());
        }) as Box<dyn FnMut(_)>);
        compile_button
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    let render_func: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = render_func.clone();
    {
        let render_state = render_state.clone();
        let start_time = start_time.clone();
        let params = params.clone();
        let last_200_frame_times = last_200_frame_times.clone();
        let canvas_ref = canvas_ref.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            {
                let window = window();
                let mut pixel_ratio = window.device_pixel_ratio();
                let (mut width, mut height) = (
                    window.inner_width().unwrap().as_f64().unwrap(),
                    window.inner_height().unwrap().as_f64().unwrap(),
                );

                let (q_width, q_height) = (query_params.get("width"), query_params.get("height"));
                if q_width.is_some() {
                    width = q_width.unwrap().parse::<f64>().unwrap();
                    pixel_ratio = 1.0;
                }
                if q_height.is_some() {
                    height = q_height.unwrap().parse::<f64>().unwrap();
                    pixel_ratio = 1.0;
                }

                // let pixel_ratio = 1.;
                // let (width, height) = (1024., 1024.);
                {
                    params.borrow_mut().dimensions =
                        ((width * pixel_ratio) as u32, (height * pixel_ratio) as u32);
                }
                let mut_canvas = canvas_ref.borrow();
                mut_canvas
                    .style()
                    .set_property("width", &(width as u32).to_string())
                    .unwrap();
                mut_canvas
                    .style()
                    .set_property("height", &(height as u32).to_string())
                    .unwrap();
                params.borrow_mut().seconds_since_start = SystemTime::now()
                    .duration_since(*start_time.borrow())
                    .unwrap()
                    .as_secs_f32();
            }
            let mut frame_times = last_200_frame_times.borrow_mut();
            {
                frame_times.push(params.borrow().seconds_since_start);
            }
            if frame_times.len() > 200 {
                frame_times.remove(0);
            }
            {
                fps_counter.set_inner_text(&format!(
                    "FPS: {:.1}\n{:?}",
                    (frame_times.len() as f32)
                        / (frame_times.last().unwrap() - frame_times.first().unwrap()),
                    params.borrow().dimensions,
                ));
            }
            render(&mut render_state.borrow_mut(), &params.borrow()).unwrap();
            requestAnimationFrame(render_func.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));
        window()
            .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .expect("should register `requestAnimationFrame` OK");
    }

    Ok(())
}
