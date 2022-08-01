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
out vec4 outColor;

uniform ivec2 dimensions;
uniform float vertical_fov_degrees;
uniform vec3 normalized_dir;
uniform vec3 normalized_up;
uniform vec3 normalized_pos;
uniform mat3x3 observer_mat;
uniform float distance;
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
    return texture(cache,vec2(index-.5/float(cache_dim.x),(distance-5.)/15.)).xyz;
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

// vec4 get_disc_color(vec3 start_dir,vec3 true_start_dir,vec2 coord){
    
    //     float index=get_angle_cache_index(start_dir)+.5;
    //     vec3 travel_normal=normalize(cross(normalized_dir,true_start_dir));
    //     vec3 intersection=normalize(cross(travel_normal,vec3(0.,1.,0.)));
    
    //     float dist=dot(intersection,-normalized_pos);
    
    //     // there are two angles that matter;
    //     // which to use depends on whether the ray is going "under" or "over"
    
    //     float angle=acos(dist);
    //     if(normalized_pos.y>0.){
        //         // top half should be >= PI/2.
        //         if(coord.y>.5){
            //             angle=max(angle,PI-angle);
        //         }else{
            //             angle=min(angle,PI-angle);
        //         }
    //     }else{
        //         if(coord.y<.5){
            //             angle=max(angle,PI-angle);
        //         }else{
            //             angle=min(angle,PI-angle);
        //         }
    //     }
    
    //     float other_angle=angle+PI;
    
    //     float dist_1=texture(angle_cache_tex,vec2(angle/(2.*PI),index/float(angle_cache_tex_dim.y))).y;
    //     float dist_2=texture(angle_cache_tex,vec2(other_angle/(2.*PI),index/float(angle_cache_tex_dim.y))).y;
    
    //     if(dist_1>3.&&dist_1<6.){
        //         float d=(6.-dist_1)/3.;
        //         return vec4(1.-d,d,0.,.8);
    //     }
    //     if(dist_2>3.&&dist_2<6.){
        //         float d=(6.-dist_2)/3.;
        //         return vec4(1.-d,d,0.,.8);
    //     }
    //     return vec4(1.,1.,1.,.0);
// }

vec3 get_color(vec2 coord){
    vec3 start_dir=get_start_dir(coord);
    vec3 true_start_dir=get_true_start_dir(coord);
    vec3 background_color=get_background_color(start_dir);
    return background_color.xyz;
    //vec4 disc_color=get_disc_color(start_dir,true_start_dir,coord);
    //return disc_color.w*disc_color.xyz+(1.-disc_color.w)*background_color.xyz;
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