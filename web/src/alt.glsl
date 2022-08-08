
#define SPEED_UP 1.*.1
#define DIST_POINTS 14.
#define REVOLUTION_COUNT 1.
#define ARMS_COUNT 12.

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
#define OCTAVES 20

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

// https://stackoverflow.com/questions/141855/programmatically-lighten-a-color
vec3 scale_color(vec3 color,float scale){
    vec3 color_n=scale*color;
    float m=max(max(color_n.x,color_n.y),color_n.z);
    
    if(m<=1.){
        return color_n;
    }
    float total=color_n.x+color_n.y+color_n.z;
    if(total>=3.){
        return vec3(1.);
    }
    
    float x=(3.*1.-total)/(3.*m-total);
    float gray=1.-x*m;
    return vec3(gray+x*color_n.x,gray+x*color_n.y,gray+x*color_n.z);
}
#define ARM_DIST_SCALE 2.5
#define INNER_SPEED_SCALE.03
#define ARM_DIST_NORMALIZATION pow(TAU,ARM_DIST_SCALE)
#define CLOUD_DENSITY.1

vec4 disc_color(float dist_01,float theta_01){
    float dist_rescaled=((dist_01+INNER_SPEED_SCALE/(.99-dist_01))-INNER_SPEED_SCALE/.99)/(1.98-INNER_SPEED_SCALE/.99);
    float arm=mod(ARMS_COUNT*(theta_01+dist_rescaled*REVOLUTION_COUNT),ARMS_COUNT);
    float theta_start=arm/ARMS_COUNT;
    float theta_offset=mod(TAU*(1.+theta_01-theta_start),TAU);
    float arm_dist=pow(theta_offset,ARM_DIST_SCALE)/ARM_DIST_NORMALIZATION;
    vec2 show=vec2(1.,1.)*vec2(.9,dist_01/1.5);
    
    float density=clamp(1.-dist_rescaled/1.1,0.,1.);
    
    float noi=smoothstep(0.,.25,dist_01)*clamp((1./density)*(fbm(vec2(arm,arm_dist*ARM_DIST_NORMALIZATION*CLOUD_DENSITY+time_s*CLOUD_DENSITY*20.))-(1.-density)),0.,1.);
    
    vec3 hard_red=vec3(.9,0.,0.)/2.;
    vec3 orange=vec3(1.,.5176,0.)/2.;
    vec3 white=vec3(1.);
    float brightness=2.*clamp(1.-density,0.,1.);
    
    float alpha=smoothstep(.3,.55,dist_01)-smoothstep(.95,.99,dist_01)-noi;
    vec3 color=smoothstep(.05,.1,brightness)*hard_red+smoothstep(.1,.5,brightness)*(orange-hard_red)+smoothstep(.95,1.,brightness)*(white-orange);
    
    return vec4(scale_color(vec3(show,0.),3.*(1.-density)),clamp(alpha,0.,1.));
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