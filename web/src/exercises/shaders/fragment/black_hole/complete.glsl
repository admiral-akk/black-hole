#version 300 es

precision mediump float;
uniform sampler2D start_ray_tex;
uniform sampler2D ray_cache_tex;
uniform sampler2D stars;
uniform ivec2 stars_dim;
uniform sampler2D constellations;
uniform ivec2 constellations_dim;
uniform sampler2D galaxy;
uniform ivec2 galaxy_dim;
out vec4 outColor;

uniform ivec2 dimensions;
uniform float vertical_fov_degrees;
uniform vec3 normalized_dir;
uniform vec3 normalized_up;
uniform vec3 normalized_pos;
uniform mat3x3 observer_mat;
uniform float max_z;
uniform float ray_cache_length;
#define PI 3.1415926538
#define AA_LEVEL 4.

vec3 uv_grid(vec3 final_dir){
    float horizontal_len=sqrt(final_dir.x*final_dir.x+final_dir.z*final_dir.z);
    float phi=atan(final_dir.z,final_dir.x);
    if(phi<0.){
        phi=4.*PI+phi;
    }
    
    float theta=atan(final_dir.y,horizontal_len)+PI;
    
    phi=mod(phi,2.*PI);
    theta=mod(theta,PI);
    
    float phi_d=mod(180.*phi/PI,10.);
    float theta_d=mod(180.*theta/PI,10.);
    
    float r=(1.-(smoothstep(0.,1.,phi_d)-smoothstep(9.,10.,phi_d)))*(.5+phi/(4.*PI));
    float g=(1.-(smoothstep(0.,1.,theta_d)-smoothstep(9.,10.,theta_d)))*(.5+theta/(2.*PI));
    float b=.25+r+g;
    return vec3(r,g,b);
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

bool black_hole_hit(vec3 start_dir){
    return start_dir.z>=max_z;
}

vec3 get_cached_dir(vec3 start_dir){
    // todo(CPU pre-compute)
    float z_to_index_multiple=((ray_cache_length-1.)/((max_z+1.)*(max_z+1.)));
    
    float index=(z_to_index_multiple*(start_dir.z+1.)*(start_dir.z+1.));
    return texture(ray_cache_tex,vec2((index+.5)/ray_cache_length,.5)).xyz;
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
    
    // todo(figure out more idiomatic representation of rotation)
    mat3x3 rot=mat3x3(vec2(cos_val,sin_val),0.,vec2(-sin_val,cos_val),0.,vec2(0.,0.),1.);
    cached_dir=rot*cached_dir;
    
    // todo(CPU pre-compute)
    mat3x3 inv=inverse(observer_mat);
    return inv*cached_dir;
}

vec3 get_final_color(vec3 final_dir){
    return clamp(star_sample(final_dir)+constellation_sample(final_dir)+galaxy_sample(final_dir),0.,1.);
}

vec3 get_color(vec2 coord){
    vec3 start_dir=get_start_dir(coord);
    if(black_hole_hit(start_dir)){
        return vec3(0.,0.,0.);
    }
    vec3 cached_dir=get_cached_dir(start_dir);
    vec3 final_dir=get_final_dir(start_dir,cached_dir);
    return get_final_color(final_dir);
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