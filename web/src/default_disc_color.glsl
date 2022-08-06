float random(in vec2 _st){
    return fract(sin(dot(_st.xy,vec2(312.12,1.*TAU)))*42.5453123);
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

vec4 disc_color(float dist_01,float theta_01){
    float n=noise(vec2(dist_01,theta_01)*vec2(42.3,1.));
    return vec4(n,n,n,1.);
    float offset=5.*TAU*dist_01+n+time_s;
    float white=clamp((.5+sin(theta_01*TAU+offset)),0.,1.);
    return vec4(n,n,n,1.);
}