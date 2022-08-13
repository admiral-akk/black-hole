
#define SPEED_UP 1.*.1
#define DIST_POINTS 14.
#define REVOLUTION_COUNT 1.
#define ARMS_COUNT 12.

#define THETA_POINTS 35.*(1.+SPEED_UP)

#define AA_LEVEL 2.
#define SEARCH_RANGE 10.
vec3 temp_color(vec2 coord){
    
    vec3 stars_c=texture(stars,coord+.5/vec2(stars_dim)).xyz;
    vec3 constellations_c=texture(constellations,coord+.5/vec2(constellations_dim)).xyz;
    vec3 galaxy_c=texture(galaxy,coord+.5/vec2(galaxy_dim)).xyz;
    return stars_c+constellations_c+galaxy_c;
}

void main(){
    // Sample
    vec2 delta=1./vec2(dimensions);
    vec2 coord2=(gl_FragCoord.xy)*delta;
    vec3 color_t=vec3(0.);
    float aa_level3=AA_LEVEL+1.;
    float aa_half_delta2=delta.x/(2.*aa_level3);
    for(float x=0.;x<aa_level3;x=x+1.){
        for(float y=0.;y<aa_level3;y=y+1.){
            vec2 tar=coord2+aa_half_delta2*vec2(1.+2.*x,1.+2.*y);
            color_t+=temp_color(tar);
        }
    }
    color_t/=aa_level3*aa_level3;
    float dist_to_constellation=1.;
    for(float x=1.-SEARCH_RANGE;x<SEARCH_RANGE;x++){
        for(float y=1.-SEARCH_RANGE;y<SEARCH_RANGE;y++){
            vec2 test_coord=coord2+delta*vec2(x,y);
            float constellations_c=texture(constellations,test_coord+.5/vec2(constellations_dim)).r;
            if(constellations_c>.5){
                dist_to_constellation=min(dist_to_constellation,length(vec2(x,y))/10.);
            }
        }
    }
    
    outColor=vec4(clamp(color_t,0.,1.),dist_to_constellation);
}