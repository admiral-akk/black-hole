
#define SPEED_UP .1
#define DIST_POINTS 14.
#define REVOLUTION_COUNT 1.
#define ARMS_COUNT 12.

#define THETA_POINTS(1.+SPEED_UP)*35.
float random(in vec2 _st){
    return fract(cos(dot(_st.xy,vec2(112.4,1./ARMS_COUNT)))*421.5453123);
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
#define OCTAVES 4
#define AMP_DROP.5

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

vec4 disc_color(float dist_01,float theta_01){
    float arm=mod(ARMS_COUNT*(theta_01+time_s/15.+dist_01*REVOLUTION_COUNT),ARMS_COUNT);
    float arm_weight=2.*abs(fract(arm)-.5);
    float arm_dist_01=pow(dist_01,18.);
    vec2 st=vec2(arm_dist_01-time_s,arm);
    
    for(int i=0;i<0;i++){
        st=st+vec2(fbm(st),fbm(st));
    }
    float n=fbm(st);
    vec2 q=vec2(0.,0.);
    q.x=fbm(st+0.*time_s);
    q.y=fbm(st+vec2(1.,1.));
    
    vec2 r=vec2(0.);
    r.x=fbm(st+1.*q+vec2(1.7,9.2)+.15*time_s);
    r.y=fbm(st+1.*q+vec2(8.3,2.8)+.126*time_s);
    float f=fbm(st+r);
    vec2 show=vec2(1.,1.)*n;
    return vec4(show.x,show.y,0.,1.);
}