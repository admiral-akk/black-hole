// Vertex shader

 
 let PI2: f32 =1.5707963269;
 let PI : f32= 3.1415926538;
 let TAU: f32 = 6.2831853076;
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct RenderParams {
    observer_matrix: mat4x4<f32>,
    cursor_pos: vec2<f32>,
    resolution: vec2<f32>,
    distance: f32,
    time_s: f32,
    view_width: f32,
}

struct BlackHole {
    disc_bounds: vec2<f32>,
    distance_bounds: vec2<f32>,
    radius: f32,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}
@group(0) @binding(0)
var galaxy_t: texture_2d<f32>;
@group(0) @binding(1)
var galaxy_s: sampler;
@group(0) @binding(2)
var<uniform> black_hole: BlackHole;
@group(0) @binding(3)
var<uniform> render_params: RenderParams;
@group(0) @binding(4)
var noise_t: texture_2d<f32>;
@group(0) @binding(5)
var noise_s: sampler;
@group(0) @binding(6)
var sin_t: texture_2d<f32>;
@group(0) @binding(7)
var sin_s: sampler;
@group(0) @binding(8)
var dir_z_bounds_t: texture_1d<f32>;
@group(0) @binding(9)
var dir_z_bounds_s: sampler;
@group(0) @binding(10)
var final_dir_t: texture_2d<f32>;
@group(0) @binding(11)
var final_dir_s: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let s = textureSample(sin_t,sin_s, in.tex_coords);
    let pixel_xy = render_params.resolution * in.tex_coords;
    let min_dim=min(render_params.resolution.x,render_params.resolution.y);
    let max_dim=max(render_params.resolution.x,render_params.resolution.y);
    let diff=max_dim-min_dim;
    let lower=diff/2.;
    let upper=min_dim+lower;
    let delta=1./vec2(min_dim);
    var offset=vec2(lower);
    let distance = render_params.distance;
    let distance_bounds = black_hole.distance_bounds;
    let z_bounds=textureSample(dir_z_bounds_t,dir_z_bounds_s,
    (distance-distance_bounds.x)/(distance_bounds.y-distance_bounds.x)).xy;
    if(render_params.resolution.x > render_params.resolution.y){
        if(pixel_xy.x < lower || pixel_xy.x > upper){
            return vec4(vec3(0.),1.);
        } else {
            offset.y=0.;
        }
    }else{
        if(pixel_xy.y < lower || pixel_xy.y > upper){
            return vec4(vec3(0.),1.);
        } else {
            offset.x=0.;
        }
    }
    let coords = (pixel_xy - offset)*delta - 0.5;
    let start_dir = normalize(vec3(render_params.view_width*coords, 1.));
    var final_dir = start_dir;
    let z_01=(start_dir.z-z_bounds.x)/(z_bounds.y-z_bounds.x);
    if (z_01 > 1. || z_01< 0.) {
        return vec4(vec3(0.), 1.);
    } else if (z_01 >= 0.)  {
        var z_pow = z_01;
        for (var i = 0; i < 4; i += 1) {
            z_pow = z_pow*z_pow;
        }
        z_pow = clamp(max(z_pow, z_01/20.),0.,1.);
    } 

    let v = (render_params.observer_matrix*vec4(start_dir,0.)).xyz;
    return vec4(v.xyz,1.);
}
 