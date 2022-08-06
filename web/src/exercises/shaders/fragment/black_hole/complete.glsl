#version 300 es

precision mediump float;
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
uniform sampler2D angle_cache;
uniform ivec2 angle_cache_dim;
uniform sampler2D angle_z_max_cache;
uniform ivec2 angle_z_max_cache_dim;
out vec4 outColor;

uniform ivec2 dimensions;
uniform float vertical_fov_degrees;
uniform vec3 normalized_dir;
uniform vec3 normalized_up;
uniform vec3 normalized_pos;
uniform mat3x3 observer_mat;
uniform float distance;
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
    return texture(stars,vec2(phi/(2.*PI),theta/PI)).xyz;
}
vec3 constellation_sample(vec3 final_dir){
    return vec3(0.,0.,0.);
    float horizontal_len=sqrt(final_dir.x*final_dir.x+final_dir.z*final_dir.z);
    float phi=4.*PI+atan(final_dir.z,final_dir.x);
    
    float theta=atan(final_dir.y,horizontal_len)+3.*PI/2.;
    
    phi=mod(phi,2.*PI);
    theta=mod(theta,PI);
    return texture(constellations,vec2(phi/(2.*PI),theta/PI)).xyz;
}
vec3 galaxy_sample(vec3 final_dir){
    float horizontal_len=sqrt(final_dir.x*final_dir.x+final_dir.z*final_dir.z);
    float phi=4.*PI+atan(final_dir.z,final_dir.x);
    
    float theta=atan(final_dir.y,horizontal_len)+3.*PI/2.;
    
    phi=mod(phi,2.*PI);
    theta=mod(theta,PI);
    return texture(galaxy,vec2(phi/(2.*PI),theta/PI)).xyz;
}
bool black_hole_hit(vec3 start_dir){
    float z=texture(z_max_cache,vec2((distance-5.)/15.,.5)).x;
    return start_dir.z>=z;
}

float get_cache_index(vec3 start_dir){
    // todo(CPU pre-compute)
    float z=texture(z_max_cache,vec2((distance-5.)/15.,.5)).x;
    float val=(start_dir.z+1.)/(z+1.);
    return val*val;
}
vec3 get_cached_dir(vec3 start_dir){
    float index=get_cache_index(start_dir);
    return texture(cache,vec2(index-.5/float(cache_dim.x),(distance-5.)/15.)).xzy;
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

float random(in vec2 _st) {
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
}

//
/* Disc distance/angle calculations */
//

vec3 get_true_start_dir(vec2 coord){
    // todo(CPU pre-compute)
    vec3 forward=normalized_dir;
    // todo(CPU pre-compute)
    vec3 up=normalized_up;
    // todo(CPU pre-compute)
    vec3 right=cross(forward,up);
    // todo(CPU pre-compute)
    float view_width=2.*tan(PI*vertical_fov_degrees/360.);
    
    return normalize(view_width*((coord.x-.5)*right+(coord.y-.5)*up)+forward);
}

float to_angle_index(float angle,float z){
    vec2 z_bounds=texture(angle_z_max_cache,vec2(angle,.5)).xy;
    float z_01=(z-z_bounds.x)/(z_bounds.y-z_bounds.x);
    if(z_01>.5){
        z_01=2.*(z_01-.5);
        z_01=z_01*z_01;
        z_01=z_01/2.+.5;
    }else{
        z_01=2.*(.5-z_01);
        z_01=z_01*z_01;
        z_01=.5-z_01/2.;
    }
    return z_01;
}

vec2 get_disc_angle(vec3 true_start_dir,vec2 coord){
    vec3 travel_normal=normalize(cross(normalized_dir,true_start_dir));
    vec3 intersection=normalize(cross(travel_normal,vec3(0.,1.,0.)));
    float dist=dot(intersection,-normalized_pos);
    
    // there are two angles that matter;
    // which to use depends on whether the ray is going "under" or "over"
    float angle_01=acos(dist)/TAU;
    float alt_angle_01=.5-angle_01;
    bool above=normalized_pos.y>0.;
    bool top_coord=coord.y>.5;
    float theta_01=atan(intersection.z,intersection.x)/TAU+.5;
    if(above==top_coord){
        return vec2(max(angle_01,alt_angle_01),theta_01);
    }else{
        return vec2(min(angle_01,alt_angle_01),theta_01);
    }
}


vec4 get_disc_color(vec3 start_dir,vec3 true_start_dir,vec2 coord){
    
    float is_top=1.;
    if(normalized_pos.y<0.){
        is_top=0.;
    }
    vec3 close_color=vec3(is_top,1.-is_top,0.);
    vec3 far_color=vec3(1.-is_top,is_top,0.);
    vec2 angle_01=get_disc_angle(true_start_dir,coord);
    
    float z=start_dir.z;
    float z_index=to_angle_index(angle_01.x,z);
    if(z_index>=0.&&z_index<=1.){
        // return vec4(angle/(2.*PI),1.,0.,0.);
        float dist=texture(angle_cache,vec2(z_index,angle_01.x)).x;
        if(dist>3.&&dist<6.){
            float dist_01=(6.-dist)/3.;
            return disc_color(dist_01,angle_01.y);
        }
    }
    vec2 other_angle_01=angle_01+.5;
    z_index=to_angle_index(other_angle_01.x,z);
    if(z_index>=0.&&z_index<=1.){
        float dist=texture(angle_cache,vec2(z_index,other_angle_01.x)).x;
        if(dist>3.&&dist<6.){
            float dist_01=(6.-dist)/3.;
            return disc_color(dist_01,other_angle_01.y);
        }
    }
    return vec4(0.,0.,0.,0.);
}

//
/* Main methods */
//

vec3 get_color(vec2 coord){
    vec3 start_dir=get_start_dir(coord);
    vec3 true_start_dir=get_true_start_dir(coord);
    vec3 background_color=get_background_color(start_dir);
    vec4 disc_color=get_disc_color(start_dir,true_start_dir,coord);
    return disc_color.w*disc_color.xyz+(1.-disc_color.w)*background_color.xyz;
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