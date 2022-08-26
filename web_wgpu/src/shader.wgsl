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
var close_theta_f_t: texture_2d<f32>;
@group(0) @binding(7)
var close_theta_f_s: sampler;
@group(0) @binding(8)
var close_theta_1_t: texture_2d<f32>;
@group(0) @binding(9)
var close_theta_1_s: sampler;
@group(0) @binding(10)
var close_dist_1_t: texture_2d<f32>;
@group(0) @binding(11)
var close_dist_1_s: sampler;
@group(0) @binding(12)
var far_theta_f_t: texture_2d<f32>;
@group(0) @binding(13)
var far_theta_f_s: sampler;
@group(0) @binding(14)
var far_theta_1_t: texture_2d<f32>;
@group(0) @binding(15)
var far_theta_1_s: sampler;
@group(0) @binding(16)
var far_dist_1_t: texture_2d<f32>;
@group(0) @binding(17)
var far_dist_1_s: sampler;
@group(0) @binding(18)
var view_bounds_t: texture_1d<f32>;
@group(0) @binding(19)
var view_bounds_s: sampler;

fn to_float(v: vec2<f32>) -> f32 {
    return v.x + v.y/2048.0;
}

fn to_vec2(v: vec4<f32>) -> vec2<f32> {
return vec2(to_float(v.xy),to_float(v.zw));
}

fn to_high_p_float(v: vec4<f32>) -> f32 {
     return v.w/(2048.0*2048.0*2048.0) +v.z/(2048.0*2048.0) + v.y/2048.0+v.x;
}

// https://stackoverflow.com/questions/141855/programmatically-lighten-a-color
fn scale_color( color:vec3<f32>, scale: f32) -> vec3<f32>{
    let color_n=scale*color;
    let m=max(max(color_n.x,color_n.y),color_n.z);
    
    let total=color_n.x+color_n.y+color_n.z;
    let saturated = step(1., m);
    let over_saturated = step(3.,total);
    
    let x=(3.-total)/(3.*m-total);
    let gray=1.-x*m;
    let scaled= vec3(gray+x*color_n.x,gray+x*color_n.y,gray+x*color_n.z);
    return color_n * (1.-saturated) + saturated * ((1.-over_saturated)*scaled
    +vec3(1.) * over_saturated);
}


fn disc_color( dist_01:f32, theta_01:f32)-> vec4<f32>{

let REVOLUTION_COUNT =1.;
let ARMS_COUNT = 2.0;
let ARM_DIST_SCALE = 3.0;
let INNER_SPEED_SCALE = 0.03;
let  ARM_DIST_NORMALIZATION =pow(TAU,ARM_DIST_SCALE);
let CLOUD_DENSITY = 0.1;
let dist_01 =1.0-dist_01;
    let dist_rescaled=((dist_01+INNER_SPEED_SCALE/(.97-dist_01))-INNER_SPEED_SCALE/.97)/(1.98-INNER_SPEED_SCALE/.99);
    let arm=ARMS_COUNT*fract(theta_01+dist_rescaled*REVOLUTION_COUNT);
    let theta_start=arm/ARMS_COUNT;
    let theta_offset=TAU*fract(10.+theta_01-theta_start);
    let arm_dist=pow(theta_offset,ARM_DIST_SCALE)/ARM_DIST_NORMALIZATION;
    let show=vec2(1.,1.)*vec2(.9,dist_01/1.5);
    
    let density=clamp(1.-dist_rescaled/1.1,0.,1.);
    
    let x =arm ;
    let y = arm_dist+render_params.time_s*CLOUD_DENSITY;
    let noi_tex=textureSample(noise_t,noise_s,fract(vec2(x,y))).r*1.1;
    let noi=smoothstep(0.,.25,dist_01)*clamp((1./density)*(noi_tex -(1.-density)),0.,1.);
    let brightness=2.*clamp(1.-density,0.,1.);
    let alpha=smoothstep(.3,.55,dist_01)-smoothstep(.9,.94,dist_01)-noi;
    
    let color = clamp(vec4(scale_color(vec3(show,0.),3.*(1.-density)),alpha),vec4(0.),vec4(1.));

    return color*color;
}




fn calculate_view_01_dim(coord:vec2<f32>) -> f32 {
    let v = length(coord);
   let bound = to_high_p_float( textureSample(view_bounds_t,view_bounds_s,v));
 let low_bound = step(0.,bound-v,);
let low_v = sqrt(low_bound*(bound - v) / bound);
  
  let high_bound = 1.-low_bound;
  let high_v = sqrt(high_bound*(v-bound ) / (1.-bound));
   return low_v + high_v;

}

 fn get_disc_color( start_dir: vec3<f32>, coord:vec2<f32>, d_01:f32) -> vec4<f32>{
   
let normalized_pos = 
-vec3(render_params.observer_matrix[2][0],render_params.observer_matrix[2][1], render_params.observer_matrix[2][2]);
    let true_start_dir = (render_params.observer_matrix * vec4(start_dir,0.)).xyz;
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
    
    var total_disc_color=vec4(0.,0.,0.,0.);
    let other_angle_01=angle_01+.5;


   return total_disc_color;
}
fn background_color(start_dir: vec3<f32>, d_01: f32,coords:vec2<f32>) -> vec3<f32> {
    let hit_black_hole = 0.;
    return (1.-hit_black_hole)*textureSample(galaxy_t,galaxy_s,coords).xyz;
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
    let background_color=background_color(start_dir,d_01,coords);
    let disc_color = get_disc_color(start_dir, coords, d_01);
    let final_color =disc_color.w* disc_color.xyz + (1.-disc_color.w)*background_color;
    if(render_params.resolution.x > render_params.resolution.y){
        if(pixel_xy.x < lower || pixel_xy.x > upper){
            return vec4(vec3(0.),1.);
        } 
    }else{
        if(pixel_xy.y < lower || pixel_xy.y > upper){
            return vec4(vec3(0.),1.);
        } 
    }
    return vec4(vec3(calculate_view_01_dim(coords)), 1.0);
}
 