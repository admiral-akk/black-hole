
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