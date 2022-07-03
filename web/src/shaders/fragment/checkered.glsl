#version 300 es

precision mediump float;

out vec4 outColor;
void main(){
    float cx=mod(floor(gl_FragCoord.x/16.),2.);
    float cy=mod(floor(gl_FragCoord.y/16.),2.);
    float chex=mod(cx+cy,2.);
    outColor=vec4(.5,.5,.5,.5);
}