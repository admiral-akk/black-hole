

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
precision highp sampler3D;
#else
precision highp float;
precision highp sampler3D;
#endif
uniform sampler2D stars;
uniform ivec2 stars_dim;
uniform sampler2D constellations;
uniform ivec2 constellations_dim;
uniform sampler2D galaxy;
uniform ivec2 galaxy_dim;
uniform sampler3D distance_cache_tex;
uniform ivec3 distance_cache_tex_dim;
uniform sampler2D distance_cache_z_bounds;
uniform ivec2 distance_cache_z_bounds_dim;
uniform sampler2D direction_cache;
uniform ivec2 direction_cache_dim;
uniform sampler2D direction_z_max_cache;
uniform ivec2 direction_z_max_cache_dim;
uniform sampler2D disc_noise;
uniform ivec2 disc_noise_dim;
out vec4 outColor;

uniform float min_angle;

uniform ivec2 dimensions;
uniform vec2 disc_dim;
uniform float vertical_fov_degrees;
uniform vec3 normalized_dir;
uniform vec3 normalized_up;
uniform vec3 normalized_pos;
uniform mat3x3 observer_mat;
uniform mat3x3 inv_observer_mat;
uniform float distance;
uniform vec2 distance_bounds;
uniform float time_s;
uniform float vertical_fov_magnitude;

#define PI_2 1.5707963269
#define PI 3.1415926538
#define TAU 6.2831853076

#define X_POINTS 41.
#define Y_POINTS 21.
#define COORD_SCALE vec2(X_POINTS,Y_POINTS)
#define AA_LEVEL 3.

float random(in vec2 _st){
    return fract(cos(dot(_st.xy,TAU/COORD_SCALE))*421.5453123);
}

// Based on Morgan McGuire @morgan3d
// https://www.shadertoy.com/view/4dS3Wd
float noise(in vec2 _st){
    vec2 i=floor(_st);
    vec2 f=fract(_st);
    
    // Four corners in 2D of a tile
    float a=random(i);
    float b=random(i+vec2(1.,0.));
    float c=random(i+vec2(0.,1.));
    float d=random(i+vec2(1.,1.));
    
    vec2 u=f*f*(3.-2.*f);
    
    return mix(a,b,u.x)+
    (c-a)*u.y*(1.-u.x)+
    (d-b)*u.x*u.y;
}
#define OCTAVES 3
float fbm(in vec2 st){
    // Initial values
    float value=0.;
    float amplitude=.5;
    float frequency=0.;
    //
    // Loop of octaves
    for(int i=0;i<OCTAVES;i++){
        value+=amplitude*noise(st);
        st*=2.;
        amplitude*=.5;
    }
    return value;
}

vec3 temp_color(vec2 coord){
    return vec3(fbm(coord),0.,0.);
}

void main(){
    // Sample
    vec2 delta=1./vec2(dimensions);
    vec2 coord2=(gl_FragCoord.xy)*delta+time_s/10.;
    vec3 color_t=vec3(0.);
    float aa_level3=AA_LEVEL+1.;
    float aa_half_delta2=delta.x/(2.*aa_level3);
    for(float x=0.;x<aa_level3;x=x+1.){
        for(float y=0.;y<aa_level3;y=y+1.){
            vec2 tar=coord2+aa_half_delta2*vec2(1.+2.*x,1.+2.*y);
            color_t+=temp_color(COORD_SCALE*tar);
        }
    }
    color_t/=aa_level3*aa_level3;
    
    outColor=vec4(clamp(color_t,0.,1.),1.);
}