#version 300 es

precision mediump float;
precision mediump sampler3D;
uniform sampler2D stars;
uniform ivec2 stars_dim;
uniform sampler2D constellations;
uniform ivec2 constellations_dim;
uniform sampler2D galaxy;
uniform ivec2 galaxy_dim;
uniform sampler2D cache;
uniform ivec2 cache_dim;
uniform sampler2D z_max_cache;
uniform ivec2 z_max_cache_dim;
uniform sampler3D distance_cache_tex;
uniform ivec3 distance_cache_tex_dim;
uniform sampler2D distance_cache_z_bounds;
uniform ivec2 distance_cache_z_bounds_dim;
uniform sampler2D direction_cache;
uniform ivec2 direction_cache_dim;
uniform sampler2D direction_z_max_cache;
uniform ivec2 direction_z_max_cache_dim;
out vec4 outColor;

uniform float min_angle;

uniform ivec2 dimensions;
uniform vec2 disc_dim;
uniform float vertical_fov_degrees;
uniform vec3 normalized_dir;
uniform vec3 normalized_up;
uniform vec3 normalized_pos;
uniform mat3x3 observer_mat;
uniform float distance;
uniform vec2 distance_bounds;
uniform float time_s;

#define PI_2 1.5707963269
#define PI 3.1415926538
#define TAU 6.2831853076
#define AA_LEVEL 4.

//
/* Background color calculations */
//

// Pretty sure this is just sampling from (x,y,1.)?
vec3 get_start_dir(vec2 coord){
    // todo(CPU pre-compute)
    vec3 forward=observer_mat*normalized_dir;
    // todo(CPU pre-compute)
    vec3 up=observer_mat*normalized_up;
    // todo(CPU pre-compute)
    vec3 right=cross(forward,up);
    // todo(CPU pre-compute)
    float view_width=2.*tan(PI*vertical_fov_degrees/360.);
    return normalize(view_width*((coord.x-.5)*right+(coord.y-.5)*up)+forward);
}
vec3 star_sample(vec3 final_dir){
    float horizontal_len=sqrt(final_dir.x*final_dir.x+final_dir.z*final_dir.z);
    float phi=4.*PI+atan(final_dir.z,final_dir.x);
    
    float theta=atan(final_dir.y,horizontal_len)+3.*PI/2.;
    
    phi=mod(phi,2.*PI);
    theta=mod(theta,PI);
    return texture(stars,vec2(phi/(2.*PI),theta/PI)+0.5/vec2(stars_dim)).xyz;
}
vec3 constellation_sample(vec3 final_dir){
    float horizontal_len=sqrt(final_dir.x*final_dir.x+final_dir.z*final_dir.z);
    float phi=4.*PI+atan(final_dir.z,final_dir.x);
    
    float theta=atan(final_dir.y,horizontal_len)+3.*PI/2.;
    
    phi=mod(phi,2.*PI);
    theta=mod(theta,PI);
    return texture(constellations,vec2(phi/(2.*PI),theta/PI)+0.5/vec2(constellations_dim)).xyz;
}
vec3 galaxy_sample(vec3 final_dir){
    float horizontal_len=sqrt(final_dir.x*final_dir.x+final_dir.z*final_dir.z);
    float phi=4.*PI+atan(final_dir.z,final_dir.x);
    
    float theta=atan(final_dir.y,horizontal_len)+3.*PI/2.;
    
    phi=mod(phi,2.*PI);
    theta=mod(theta,PI);
    return texture(galaxy,vec2(phi/(2.*PI),theta/PI)+0.5/vec2(galaxy_dim)).xyz;
}
bool black_hole_hit(vec3 start_dir){
    float z=texture(z_max_cache,vec2((distance-5.)/15.,.0) + 0.5/vec2(z_max_cache_dim)).x;
    return start_dir.z>=z;
}

float get_cache_index(vec3 start_dir){
    // todo(CPU pre-compute)
    float max_z=texture(direction_z_max_cache,vec2((distance-5.)/15.,.0)+ 0.5/vec2(z_max_cache_dim)).x;
    float val_1=(start_dir.z+1.)/(max_z+1.);
    float i_0 = val_1;
    for (int i =0; i <5; i++) {
        i_0 = i_0 * i_0;
    }
    float i_1 = val_1 / 20.;
    return clamp(max(i_0,i_1),0.,1.);
}
vec3 get_cached_dir(vec3 start_dir){
    float index=get_cache_index(start_dir);
    return texture(direction_cache,vec2(index,(distance-5.)/15.)+.5/vec2(cache_dim)).xzy;
}
vec3 get_final_dir(vec3 start_dir,vec3 cached_dir){
    float angle=PI/2.;
    if(start_dir.x!=0.){
        angle=atan(start_dir.y,start_dir.x);
    }else if(start_dir.y<0.){
        angle=-PI/2.;
    }
    
    float sin_val=sin(angle);
    float cos_val=cos(angle);
    mat3x3 rot=mat3x3(vec2(cos_val,sin_val),0.,vec2(-sin_val,cos_val),0.,vec2(0.,0.),1.);
    cached_dir=rot*cached_dir;
    
    // todo(CPU pre-compute)
    mat3x3 inv=inverse(observer_mat);
    return inv*cached_dir;
}
vec3 get_final_color(vec3 final_dir){
    return clamp(star_sample(final_dir)+constellation_sample(final_dir)+galaxy_sample(final_dir),0.,1.);
}

vec3 get_background_color(vec3 start_dir){
    if(black_hole_hit(start_dir)){
        return vec3(0.,0.,0.);
    }
    vec3 cached_dir=get_cached_dir(start_dir);
    vec3 final_dir=get_final_dir(start_dir,cached_dir);
    return get_final_color(final_dir);
}

//
/* Disc color calculations */
//

// this is a temporary marker to get the code to appear here.
// Disc code here!

//
/* Main methods */
//

vec3 get_color(vec2 coord){
    vec3 start_dir=get_start_dir(coord);
    vec3 true_start_dir=get_true_start_dir(coord);
    vec3 background_color=get_background_color(start_dir);
    vec4 disc_color_f=get_disc_color(start_dir,true_start_dir,coord);
    return disc_color_f.w*disc_color_f.xyz+(1.-disc_color_f.w)*background_color.xyz;
}

void main(){
    // Sample
    vec2 delta=vec2(1./float(dimensions.x),1./float(dimensions.y));
    vec2 coord=gl_FragCoord.xy*delta;
    float aa_half_delta=delta.x/(2.*AA_LEVEL);
    vec3 color=vec3(0.,0.,0.);
    for(float x=0.;x<AA_LEVEL;x=x+1.){
        for(float y=0.;y<AA_LEVEL;y=y+1.){
            color+=get_color(coord+aa_half_delta*vec2(1.+2.*x,1.+2.*y));
        }
    }
    color/=AA_LEVEL*AA_LEVEL;
    
    outColor=vec4(color.xyz,1.);
}