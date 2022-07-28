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

//https://www.shadertoy.com/view/4dS3Wd
#define NUM_NOISE_OCTAVES 5

// Precision-adjusted variations of https://www.shadertoy.com/view/4djSRW
float hash(float p){p=fract(p*.011);p*=p+7.5;p*=p+p;return fract(p);}

float hash(vec2 p){vec3 p3=fract(vec3(p.xyx)*.13);p3+=dot(p3,p3.yzx+3.333);return fract((p3.x+p3.y)*p3.z);}

float noise(vec2 x){
    vec2 i=floor(x);
    vec2 f=fract(x);
    
    // Four corners in 2D of a tile
    float a=hash(i);
    float b=hash(i+vec2(1.,0.));
    float c=hash(i+vec2(0.,1.));
    float d=hash(i+vec2(1.,1.));
    
    // Simple 2D lerp using smoothstep envelope between the values.
    // return vec3(mix(mix(a, b, smoothstep(0.0, 1.0, f.x)),
    //			mix(c, d, smoothstep(0.0, 1.0, f.x)),
    //			smoothstep(0.0, 1.0, f.y)));
    
    // Same code, with the clamps in smoothstep and common subexpressions
    // optimized away.
    vec2 u=f*f*(3.-2.*f);
    return mix(a,b,u.x)+(c-a)*u.y*(1.-u.x)+(d-b)*u.x*u.y;
}

float noise(vec3 x){
    const vec3 step=vec3(110,241,171);
    
    vec3 i=floor(x);
    vec3 f=fract(x);
    
    // For performance, compute the base input to a 1D hash from the integer part of the argument and the
    // incremental change to the 1D based on the 3D -> 1D wrapping
    float n=dot(i,step);
    
    vec3 u=f*f*(3.-2.*f);
    return mix(mix(mix(hash(n+dot(step,vec3(0,0,0))),hash(n+dot(step,vec3(1,0,0))),u.x),
    mix(hash(n+dot(step,vec3(0,1,0))),hash(n+dot(step,vec3(1,1,0))),u.x),u.y),
    mix(mix(hash(n+dot(step,vec3(0,0,1))),hash(n+dot(step,vec3(1,0,1))),u.x),
    mix(hash(n+dot(step,vec3(0,1,1))),hash(n+dot(step,vec3(1,1,1))),u.x),u.y),u.z);
}

float fbm(vec2 x){
    float v=0.;
    float a=.5;
    vec2 shift=vec2(100);
    // Rotate to reduce axial bias
    mat2 rot=mat2(cos(.5),sin(.5),-sin(.5),cos(.50));
    for(int i=0;i<NUM_NOISE_OCTAVES;++i){
        v+=a*noise(x);
        x=rot*x*2.+shift;
        a*=.5;
    }
    return v;
}

float fbm(vec3 x){
    float v=0.;
    float a=.5;
    vec3 shift=vec3(100);
    for(int i=0;i<NUM_NOISE_OCTAVES;++i){
        v+=a*noise(x);
        x=x*2.+shift;
        a*=.5;
    }
    return v;
}

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

float rand(vec2 xy,vec2 seed){
    return fbm(xy+seed);
}

vec3 voronoi(vec2 final_dir_2d,vec2 seed){
    vec2 delta=vec2(.05,.05);
    vec2 node=delta*floor(final_dir_2d/delta);
    float best=10.;
    vec2 best_node=node;
    for(int x=-1;x<=1;x++){
        for(int y=-1;y<=1;y++){
            vec2 test_node=node+delta*vec2(float(x),float(y));
            test_node=test_node+delta*(rand(test_node/delta,seed)-.5);
            float len=length(test_node-final_dir_2d);
            if(len<best){
                best_node=test_node;
                best=len;
            }
        }
    }
    float dist=pow(clamp(1.-best,0.,1.),20.);
    return smoothstep(0.,1.,2.*(rand(final_dir_2d,seed.yx)-.5))*vec3(dist,dist,dist);
}

vec3 triplanar_voronoi(vec3 final_dir){
    vec3 up=voronoi(final_dir.xz,vec2(12.2,4.3));
    vec3 right=voronoi(final_dir.yz,vec2(2.2,10.3));
    vec3 forward=voronoi(final_dir.xy,vec2(22.2,40.3));
    vec3 normalized=normalize(pow(normalize(final_dir),vec3(3.,3.,3.)));
    return up*abs(normalized.y)+right*abs(normalized.x)+forward*abs(normalized.z);
}

vec3 get_color(vec2 coord){
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
        return vec3(0.,0.,0.);
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
    mat3x3 inv=inverse(observer_mat);
    final_dir=inv*final_dir;
    return clamp(star_sample(final_dir)+constellation_sample(final_dir)+galaxy_sample(final_dir),0.,1.);
}

void main(){
    // Sample
    vec2 delta=vec2(1./float(dimensions.x),1./float(dimensions.y));
    vec2 coord=(gl_FragCoord.xy+.5)*delta;
    vec2 aa_delta=vec2(1./(3.*float(dimensions.x)),1./(3.*float(dimensions.y)));
    vec3 rgb=(get_color(coord+vec2(aa_delta.x,aa_delta.y))+
    get_color(coord+vec2(-aa_delta.x,aa_delta.y))+
    get_color(coord+vec2(aa_delta.x,-aa_delta.y))+
    get_color(coord+vec2(-aa_delta.x,-aa_delta.y)))/4.;
    
    outColor=vec4(rgb.xyz,1.);
}