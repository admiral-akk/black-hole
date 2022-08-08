
#define SPEED_UP 1.*.1
#define DIST_POINTS 14.
#define REVOLUTION_COUNT 1.
#define ARMS_COUNT 22.

#define THETA_POINTS 35.*(1.+SPEED_UP)
float random(in vec2 _st){
    return fract(cos(dot(_st.xy,vec2(TAU/ARMS_COUNT,1./1.)))*421.5453123);
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
#define OCTAVES 10
#define AMP_DROP 1.*.5

float fbm(in vec2 st){
    // Initial values
    float value=0.;
    float amplitude=1.-AMP_DROP;
    float frequency=0.;
    //
    // Loop of octaves
    for(int i=0;i<OCTAVES;i++){
        value+=amplitude*noise(st);
        st*=2.;
        amplitude*=AMP_DROP;
    }
    return value;
}

#define ARM_DIST_SCALE 1.5
#define ARM_DIST_NORMALIZATION pow(TAU,ARM_DIST_SCALE)

vec4 disc_color(float dist_01,float theta_01){
    
    float arm=mod(ARMS_COUNT*(theta_01+mod((dist_01+.01/(.99-dist_01)),2./ARM_DIST_SCALE)*REVOLUTION_COUNT),ARMS_COUNT);
    float theta_start=arm/ARMS_COUNT;
    float theta_offset=mod(TAU*(1.+theta_01-theta_start),TAU);
    float arm_dist=pow(theta_offset,ARM_DIST_SCALE)/ARM_DIST_NORMALIZATION+3.*time_s/ARM_DIST_NORMALIZATION;
    vec2 show=vec2(1.,1.)*vec2(.9,dist_01/1.5);
    
    float noi=clamp(2.*(fbm(vec2(arm,arm_dist*ARM_DIST_NORMALIZATION))-.5),0.,1.);
    float alpha=smoothstep(0.,.35,dist_01)-smoothstep(.95,.99,dist_01)-noi;
    
    return vec4(show.x,show.y,0.,alpha);
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
    return(z-z_bounds.x)/(z_bounds.y-z_bounds.x);
}

vec2 get_disc_angle(vec3 true_start_dir,vec2 coord){
    vec3 travel_normal=normalize(cross(normalized_dir,true_start_dir));
    vec3 intersection=normalize(cross(travel_normal,vec3(0.,1.,0.)));
    if(abs(travel_normal.y)>.999999){
        intersection=normalized_pos;
    }
    float dist=dot(intersection,-normalized_pos);
    
    // there are two angles that matter;
    // which to use depends on whether the ray is going "under" or "over"
    float angle_01=acos(clamp(dist,-1.,1.))/TAU;
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

float to_z_index(float camera_dist_01,float angle_01,float z){
    vec2 z_bounds=texture(distance_cache_z_bounds,vec2(angle_01,camera_dist_01)).xy;
    return(z-z_bounds.x)/(z_bounds.y-z_bounds.x);
}

vec4 get_disc_color(vec3 start_dir,vec3 true_start_dir,vec2 coord){
    float is_top=1.;
    if(normalized_pos.y<0.){
        is_top=0.;
    }
    vec3 close_color=vec3(is_top,1.-is_top,0.);
    vec3 far_color=vec3(1.-is_top,is_top,0.);
    float camera_dist_01=(distance-distance_bounds.x)/(distance_bounds.y-distance_bounds.x);
    vec2 angle_01=get_disc_angle(true_start_dir,coord);
    
    float z=start_dir.z;
    float z_index=to_z_index(camera_dist_01,angle_01.x,z);
    vec4 total_disc_color=vec4(0.);
    if(z_index>=0.&&z_index<=1.){
        float dist=texture(angle_cache,vec2(z_index,angle_01.x)).x;
        if(dist>disc_dim.x&&dist<disc_dim.y){
            float dist_01=(disc_dim.y-dist)/(disc_dim.y-disc_dim.x);
            total_disc_color=disc_color(dist_01,angle_01.y);
        }
    }
    vec2 other_angle_01=angle_01+.5;
    z_index=to_z_index(camera_dist_01,other_angle_01.x,z);
    if(z_index>=0.&&z_index<=1.){
        float dist=texture(angle_cache,vec2(z_index,other_angle_01.x)).x;
        if(dist>disc_dim.x&&dist<disc_dim.y){
            float dist_01=(disc_dim.y-dist)/(disc_dim.y-disc_dim.x);
            float alpha=1.-total_disc_color.w;
            total_disc_color+=alpha*disc_color(dist_01,other_angle_01.y);
        }
    }
    return total_disc_color;
}