#version 300 es

precision highp float;
out vec4 outColor;

void main(){
    gl_FragColor=vec4(
        gl_FragCoord.x<256.?1:0,
        gl_FragCoord.y<256.?1:0,
        mod(floor(gl_FragCoord.y/8.),2.),
    1);
}