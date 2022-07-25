#version 300 es

precision mediump float;
uniform sampler2D start_ray_tex;
uniform sampler2D ray_cache_tex;
out vec4 outColor;

uniform ivec2 dimensions;
uniform float distance;
uniform float vertical_fov_degrees;
uniform float black_hole_radius;
uniform int cache_width;
uniform float max_z;
uniform vec3 normalized_pos;
uniform vec3 normalized_dir;
uniform vec3 normalized_up;
uniform mat3x3 observer_mat;
uniform float ray_cache_length;
#define PI 3.1415926538

void main(){
    vec2 delta=vec2(1./float(dimensions.x),1./float(dimensions.y));
    vec3 start_ray=texture(start_ray_tex,gl_FragCoord.xy*delta).xyz;
    float z=start_ray.z/length(start_ray);
    
    if(z>=max_z){
        outColor=vec4(start_ray.xyz,0.);
        return;
    }
    float z_to_index_multiple=((ray_cache_length-1.)/((max_z+1.)*(max_z+1.)));
    float index=(z_to_index_multiple*(z+1.)*(z+1.));
    vec3 final_dir=texture(ray_cache_tex,vec2((index+.5)/ray_cache_length,.5)).xyz;
    
    float angle=PI/2.;
    if(start_ray.x!=0.){
        angle=atan(start_ray.y,start_ray.x);
    }else if(start_ray.y<0.){
        angle=-PI/2.;
    }
    float sin_val=sin(angle);
    float cos_val=cos(angle);
    
    float x=final_dir.x;
    
    float y=final_dir.y;
    final_dir.x=x*cos_val-y*sin_val;
    final_dir.y=x*sin_val+y*cos_val;
    outColor=vec4(final_dir.xyz,1.);
}