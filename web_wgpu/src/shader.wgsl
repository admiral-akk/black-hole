// Vertex shader

 
 let PI2: f32 = 1.5707963269;
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
    cache_dim: vec2<f32>,
    distance: f32,
    time_s: f32,
    view_width: f32,
    temp: f32,
    temp2: f32,
    temp3: f32,
}

struct BlackHole {
    disc_bounds: vec2<f32>,
    distance_bounds: vec2<f32>,
    radius: f32,
    temp: f32,
    temp1: f32,
    temp2: f32,
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
var dir_z_bounds_t: texture_1d<f32>;
@group(0) @binding(7)
var dir_z_bounds_s: sampler;
@group(0) @binding(8)
var final_dir_t: texture_2d<f32>;
@group(0) @binding(9)
var final_dir_s: sampler;
@group(0) @binding(10)
var dist_z_t: texture_2d<f32>;
@group(0) @binding(11)
var dist_z_s: sampler;
@group(0) @binding(12)
var dist_t: texture_3d<f32>;
@group(0) @binding(13)
var dist_s: sampler;

fn to_float(in:vec2<f32>) -> f32 {
   return (255.*in.x + in.y) / 128. - 1.;
}
fn to_vec2(in:vec4<f32>) -> vec2<f32> {
   return vec2(to_float(in.xy),to_float(in.zw));
}

 fn get_disc_color( start_dir: vec3<f32>, coord:vec2<f32>, d_01:f32) -> vec4<f32>{
   
let normalized_pos = 
-vec3(render_params.observer_matrix[2][0],render_params.observer_matrix[2][1], render_params.observer_matrix[2][2]);
    let true_start_dir = (render_params.observer_matrix * vec4(start_dir,0.)).xyz;
    let z_bounds=textureSample(dist_z_t,dist_z_s,vec2(.25,d_01));
    let min_z = to_float(z_bounds.xy);
    let z = start_dir.z;
    let color = vec4(0.);
    let is_top = step(0.,normalized_pos.y);

    let close_color=vec3(is_top,1.-is_top,0.);
    let far_color=vec3(1.-is_top,is_top,0.);
    
    let travel_normal=normalize(cross(-normalized_pos,true_start_dir));
    let intersection=normalize(cross(travel_normal,vec3(0.,1.,0.)));
    let cos_val=clamp(dot(intersection,-normalized_pos),-1.,1.);
    
    // there are two angles that matter;
    // which to use depends on whether the ray is going "under" or "over"
    let temp_angle_01=acos(cos_val)/TAU;
    let alt_angle_01=.5-temp_angle_01;

    let theta_01=atan2(intersection.z,intersection.x)/TAU+.5;
    let top_half = step(0.,coord.y);
    let neq = step(0.5,abs((top_half - is_top)));
    let angle_01 = neq*vec2(min(temp_angle_01,alt_angle_01),theta_01) + (1.-neq) * vec2(max(temp_angle_01,alt_angle_01),theta_01);
    
    let alpha_mod=smoothstep(.03*(1.-d_01),.06*(1.-d_01),angle_01.x);
    
    var total_disc_color=vec4(0.);
    let other_angle_01=angle_01+.5;
    let z_bounds=to_vec2(textureSample(dist_z_t,dist_z_s,vec2(other_angle_01.x,d_01)));
    let z_index=(z-z_bounds.x)/(z_bounds.y-z_bounds.x);

   let in_bounds = step(0.,z_index)- step(1.,z_index);
   let dist=in_bounds*to_float(textureSample(dist_t,dist_s,vec3(z_index,other_angle_01.x,d_01)).xy);

   total_disc_color+=vec4(dist,0.,0.,in_bounds*dist + 1. - in_bounds );

   let z_bounds=to_vec2(textureSample(dist_z_t,dist_z_s,vec2(angle_01.x,d_01)));
   let z_index=(z-z_bounds.x)/(z_bounds.y-z_bounds.x);
   let in_bounds = step(0.,z_index)- step(1.,z_index);
   let dist=in_bounds*to_float(textureSample(dist_t,dist_s,vec3(z_index,angle_01.x,d_01)).xy);

   total_disc_color+= total_disc_color.w*vec4(dist,0.,0.,in_bounds*dist + 1. - in_bounds);

    let z_bounds = to_vec2(textureSample(dist_z_t,dist_z_s,vec2(coord+0.5)));
    let d_tex = to_float(textureSample(dist_t,dist_s,vec3(coord+0.5,d_01)).xy);
    return vec4(total_disc_color);
}
fn background_color(start_dir: vec3<f32>, d_01: f32) -> vec3<f32> {
    let u8_z_bounds = textureSample(dir_z_bounds_t,dir_z_bounds_s,d_01);
    let z_bounds = vec2(to_float(u8_z_bounds.xy), to_float(u8_z_bounds.zw));
    let z_01=clamp((start_dir.z-z_bounds.x)/(z_bounds.y-z_bounds.x),0.,1.1);
    var z_pow = z_01;
    for (var i = 0; i < 4; i += 1) {
        z_pow = z_pow*z_pow;
    }
    z_pow = clamp(max(z_pow, z_01/10.),0.,1.);

    let z_left = floor(z_pow*render_params.cache_dim.x)/render_params.cache_dim.x;
    let z_right = z_left+1./render_params.cache_dim.x;

    let z_left_weight = 1.0-(z_pow-z_left)/(z_right-z_left);
    let angle=-atan2(-start_dir.y,start_dir.x);
    
    let sin_val=sin(angle);
    let cos_val=cos(angle);
    let rot=mat3x3(vec3(cos_val,sin_val,0.),vec3(-sin_val,cos_val,0.),vec3(0.,0.,1.));
    let t = smoothstep(0.,0.04,z_pow);
    let u8_average_dir = textureSample(final_dir_t,final_dir_s,vec2(z_pow, d_01));
    let average_dir = vec3(to_float(u8_average_dir.xy),0.,to_float(u8_average_dir.zw));
    let cached_dir = rot* normalize(average_dir);
  
    let temp_dir = t*cached_dir+(1.-t)*start_dir;
    let final_dir=(render_params.observer_matrix*vec4(temp_dir,0.)).xyz;
    

    let horizontal_len=sqrt(final_dir.x*final_dir.x+final_dir.z*final_dir.z);
    let phi=5.*PI+atan2(final_dir.z,final_dir.x);
    
    let theta=atan2(final_dir.y,horizontal_len)+5.*PI/2.;
    
    let phi_theta=vec2(fract(phi/TAU),fract(theta/PI));
    let hit = step(z_01, 1.);
    return hit*textureSample(galaxy_t,galaxy_s,phi_theta).xyz;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_xy = render_params.resolution * in.tex_coords;
    let resolution = render_params.resolution;
    let min_dim=min(render_params.resolution.x,render_params.resolution.y);
    let max_dim=max(render_params.resolution.x,render_params.resolution.y);
    let diff=max_dim-min_dim;
    let lower=diff/2.;
    let upper=min_dim+lower;
    let delta=1./vec2(min_dim);
    var offset=vec2(lower)*vec2(step(resolution.y, resolution.x),step(resolution.x,resolution.y));
  
    let distance = render_params.distance;
    let distance_bounds = black_hole.distance_bounds;
    let d_01 =clamp((distance-distance_bounds.x)/(distance_bounds.y-distance_bounds.x),0.,1.);
    
    let coords = (pixel_xy - offset)*delta - 0.5;
    let start_dir = normalize(vec3(render_params.view_width*coords, 1.));
    let background_color=background_color(start_dir,d_01);
    let disc_color = get_disc_color(start_dir, coords, d_01);
    if(render_params.resolution.x > render_params.resolution.y){
        if(pixel_xy.x < lower || pixel_xy.x > upper){
            return vec4(vec3(0.),1.);
        } 
    }else{
        if(pixel_xy.y < lower || pixel_xy.y > upper){
            return vec4(vec3(0.),1.);
        } 
    }
    return vec4(disc_color.xyz,1.);
}
 