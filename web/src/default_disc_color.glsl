
#define SPEED_UP 1.*.1
#define DIST_POINTS 14.
#define REVOLUTION_COUNT 1.
#define ARMS_COUNT 12.

#define THETA_POINTS 35.*(1.+SPEED_UP)

#define AA_LEVEL 4.

//
/* Background color calculations */
//

// Pretty sure this is just sampling from (x,y,1.)?
vec3 star_sample(vec2 phi_theta ){
    return texture(stars,phi_theta +0.5/vec2(stars_dim)).xyz;
}
vec3 constellation_sample(vec2 phi_theta ){
    return texture(constellations,phi_theta +0.5/vec2(constellations_dim)).xyz;
}
vec3 galaxy_sample(vec2 phi_theta ){
    return texture(galaxy,phi_theta +0.5/vec2(galaxy_dim)).xyz;
}


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
    float arm=ARMS_COUNT*mod(theta_01+dist_rescaled*REVOLUTION_COUNT,1.);
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


//
/* Main methods */
//

vec4 get_disc_color(vec2 coord){
    // todo(CPU pre-compute)
    vec3 forward=observer_mat*normalized_dir;
    // todo(CPU pre-compute)
    vec3 up=observer_mat*normalized_up;
    // todo(CPU pre-compute)
    vec3 right=cross(forward,up);
    // todo(CPU pre-compute)
    float view_width=2.*tan(PI*vertical_fov_degrees/360.);
    vec3 start_dir=normalize(view_width*((coord.x-.5)*right+(coord.y-.5)*up)+forward);
    vec3 forward2=normalized_dir;
    // todo(CPU pre-compute)
    vec3 up2=normalized_up;
    // todo(CPU pre-compute)
    vec3 right2=cross(forward2,up2);
    // todo(CPU pre-compute)
    float camera_dist_01=(distance-distance_bounds.x)/(distance_bounds.y-distance_bounds.x);
    float view_width2=2.*tan(PI*vertical_fov_degrees/360.);
    vec2 offset=.5/vec2(float(distance_cache_z_bounds_dim.x),float(distance_cache_z_bounds_dim.y));
    float min_z=texture(distance_cache_z_bounds,vec2(.25,camera_dist_01)+offset).x;
    float z=start_dir.z;
    if(z<min_z){
        return vec4(0.);
    }
    
    vec3 true_start_dir=normalize(view_width2*((coord.x-.5)*right2+(coord.y-.5)*up2)+forward2);
    float is_top=1.;
    if(normalized_pos.y<0.){
        is_top=0.;
    }
    vec3 close_color=vec3(is_top,1.-is_top,0.);
    vec3 far_color=vec3(1.-is_top,is_top,0.);
    
    vec3 travel_normal=normalize(cross(normalized_dir,true_start_dir));
    vec3 intersection=normalize(cross(travel_normal,vec3(0.,1.,0.)));
    float dist=dot(intersection,-normalized_pos);
    
    // there are two angles that matter;
    // which to use depends on whether the ray is going "under" or "over"
    float temp_angle_01=acos(clamp(dist,-1.,1.))/TAU;
    float alt_angle_01=.5-temp_angle_01;
    bool above=normalized_pos.y>0.;
    bool top_coord=coord.y>.5;
    float theta_01=atan(intersection.z,intersection.x)/TAU+.5;
    vec2 angle_01;
    if(above==top_coord){
        angle_01=vec2(max(temp_angle_01,alt_angle_01),theta_01);
    }else{
        angle_01=vec2(min(temp_angle_01,alt_angle_01),theta_01);
    }
    
    vec3 dist_offset=.5/vec3(float(distance_cache_tex_dim.x),float(distance_cache_tex_dim.y),float(distance_cache_tex_dim.z));
    
    vec2 z_bounds=texture(distance_cache_z_bounds,vec2(angle_01.x,camera_dist_01)+offset).xy;
    float z_index=(z-z_bounds.x)/(z_bounds.y-z_bounds.x);
    
    vec4 total_disc_color=vec4(0.);
    if(z_index>=0.&&z_index<=1.){
        float dist=texture(distance_cache_tex,vec3(z_index,angle_01.x,camera_dist_01)+dist_offset).x;
        if(dist>disc_dim.x&&dist<disc_dim.y){
            float dist_01=(disc_dim.y-dist)/(disc_dim.y-disc_dim.x);
            total_disc_color=disc_color(dist_01,angle_01.y);
        }
    }
    vec2 other_angle_01=angle_01+.5;
    offset=.5/vec2(float(distance_cache_z_bounds_dim.x),float(distance_cache_z_bounds_dim.y));
    z_bounds=texture(distance_cache_z_bounds,vec2(other_angle_01.x,camera_dist_01)+offset).xy;
    z_index=(z-z_bounds.x)/(z_bounds.y-z_bounds.x);
    if(z_index>=0.&&z_index<=1.){
        float dist=texture(distance_cache_tex,vec3(z_index,other_angle_01.x,camera_dist_01)+dist_offset).x;
        if(dist>disc_dim.x&&dist<disc_dim.y){
            float dist_01=(disc_dim.y-dist)/(disc_dim.y-disc_dim.x);
            float alpha=1.-total_disc_color.w;
            total_disc_color+=alpha*disc_color(dist_01,other_angle_01.y);
        }
    }
    return total_disc_color;
    
}

vec3 get_color(vec2 coord){
    // todo(CPU pre-compute)
    vec3 forward=observer_mat*normalized_dir;
    // todo(CPU pre-compute)
    vec3 up=observer_mat*normalized_up;
    // todo(CPU pre-compute)
    vec3 right=cross(forward,up);
    // todo(CPU pre-compute)
    float view_width=2.*tan(PI*vertical_fov_degrees/360.);
    vec3 start_dir=normalize(view_width*((coord.x-.5)*right+(coord.y-.5)*up)+forward);
    // todo(CPU pre-compute)
    vec3 forward2=normalized_dir;
    // todo(CPU pre-compute)
    vec3 up2=normalized_up;
    // todo(CPU pre-compute)
    vec3 right2=cross(forward2,up2);
    // todo(CPU pre-compute)
    float view_width2=2.*tan(PI*vertical_fov_degrees/360.);
    
    vec3 true_start_dir=normalize(view_width2*((coord.x-.5)*right2+(coord.y-.5)*up2)+forward2);
    vec3 background_color=vec3(0.,0.,0.);
    float max_z=texture(direction_z_max_cache,vec2((distance-distance_bounds.x)/(distance_bounds.y-distance_bounds.x),.0)+.5/vec2(direction_z_max_cache_dim)).y;
    
    if(start_dir.z<max_z){
        float val_1=(start_dir.z+1.)/(max_z+1.);
        float i_0=val_1;
        for(int i=0;i<4;i++){
            i_0=i_0*i_0;
        }
        float i_1=val_1/20.;
        float index=clamp(max(i_0,i_1),0.,1.);
        vec3 cached_dir=texture(direction_cache,vec2(index,(distance-distance_bounds.x)/(distance_bounds.y-distance_bounds.x))+.5/vec2(direction_cache_dim)).xzy;
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
        vec3 final_dir=inv*cached_dir;
        float horizontal_len=sqrt(final_dir.x*final_dir.x+final_dir.z*final_dir.z);
        float phi=4.*PI+atan(final_dir.z,final_dir.x);
        
        float theta=atan(final_dir.y,horizontal_len)+3.*PI/2.;
        
        vec2 phi_theta=vec2(mod(phi,TAU)/TAU,mod(theta,PI)/PI);
        background_color=clamp(
            texture(stars,phi_theta+.5/vec2(stars_dim)).xyz+
            texture(constellations,phi_theta+.5/vec2(constellations_dim)).xyz+
            texture(galaxy,phi_theta+.5/vec2(galaxy_dim)).xyz,0.,1.);
        }
        return background_color.xyz;
    }
    
    void main(){
        // Sample
        
        vec2 delta=vec2(1./float(dimensions.x),1./float(dimensions.y));
        vec2 coord=gl_FragCoord.xy*delta;
        
        vec3 color=vec3(0.,0.,0.);
        float view_width=2.*tan(PI*vertical_fov_degrees/360.);
        float z=normalize(vec3(view_width*(coord-.5),1.)).z;
        vec2 z_bounds=texture(direction_z_max_cache,vec2((distance-distance_bounds.x)/(distance_bounds.y-distance_bounds.x),.0)+.5/vec2(direction_z_max_cache_dim)).xy;
        
        vec4 disc_color_f=get_disc_color(coord);
        if(z<z_bounds.x||AA_LEVEL==1.){
            color=get_color(coord+.5*delta);
        }else{
            bool hit=false;
            bool miss=false;
            float aa_half_delta=delta.x/(2.*AA_LEVEL);
            vec2 s=coord+vec2(aa_half_delta);
            z=normalize(vec3(view_width*(s-.5),1.)).z;
            hit=hit||(z>=z_bounds.y);
            miss=miss||(z<z_bounds.y);
            s=coord+vec2(aa_half_delta,1.-aa_half_delta);
            z=normalize(vec3(view_width*(s-.5),1.)).z;
            hit=hit||(z>=z_bounds.y);
            miss=miss||(z<z_bounds.y);
            s=coord+vec2(1.-aa_half_delta,aa_half_delta);
            z=normalize(vec3(view_width*(s-.5),1.)).z;
            hit=hit||(z>=z_bounds.y);
            miss=miss||(z<z_bounds.y);
            s=coord+vec2(1.-aa_half_delta);
            z=normalize(vec3(view_width*(s-.5),1.)).z;
            hit=hit||(z>=z_bounds.y);
            miss=miss||(z<z_bounds.y);
            if(hit&&(!miss)){
                color=get_color(coord+.5*delta);
            }else{
                for(float x=0.;x<AA_LEVEL;x=x+1.){
                    for(float y=0.;y<AA_LEVEL;y=y+1.){
                        color+=get_color(coord+aa_half_delta*vec2(1.+2.*x,1.+2.*y));
                    }
                }
                color/=AA_LEVEL*AA_LEVEL;
            }
            
        }
        color=disc_color_f.xyz*disc_color_f.w+(1.-disc_color_f.w)*color;
        outColor=vec4(color.xyz,1.);
    }