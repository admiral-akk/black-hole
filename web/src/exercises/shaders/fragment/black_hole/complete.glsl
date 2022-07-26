#version 300 es

precision mediump float;
uniform sampler2D start_ray_tex;
uniform sampler2D ray_cache_tex;
out vec4 outColor;

uniform ivec2 dimensions;
uniform float vertical_fov_degrees;
uniform vec3 normalized_dir;
uniform vec3 normalized_up;
uniform mat3x3 observer_mat;
uniform float max_z;
uniform float ray_cache_length;
#define PI 3.1415926538

void main(){
    
    // Sample
    vec2 delta=vec2(1./float(dimensions.x),1./float(dimensions.y));
    vec2 coord=(gl_FragCoord.xy)*delta;
    
    // Start dir
    
    vec3 forward=observer_mat*normalized_dir;
    vec3 up=observer_mat*normalized_up;
    vec3 right=cross(forward,up);
    
    float view_width=2.*tan(PI*vertical_fov_degrees/360.);
    
    vec3 v=view_width*((coord.x-.5)*right+(coord.y-.5)*up)+forward;
    
    // Final dir
    
    vec3 start_ray=v;
    float z=start_ray.z/length(start_ray);
    
    if(z>=max_z){
        outColor=vec4(0.,0.,0.,1.);
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
    
    float lat=0.;
    float lon=0.;
    
    float horizontal_len=sqrt(final_dir.x*final_dir.x+final_dir.z*final_dir.z);
    float phi=atan(final_dir.z,final_dir.x);
    if(phi<0.){
        phi=4.*PI+phi;
    }
    float theta=atan(final_dir.y,horizontal_len)+PI;
    
    phi=mod(180.*phi/PI,10.);
    theta=mod(180.*theta/PI,10.);
    
    float r=1.-(smoothstep(0.,1.,phi)-smoothstep(9.,10.,phi));
    float g=1.-(smoothstep(0.,1.,theta)-smoothstep(9.,10.,theta));
    
    outColor=vec4(r,g,1.,1.);
}