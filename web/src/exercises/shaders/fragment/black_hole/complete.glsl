#version 300 es

#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
precision highp sampler3D;
#else
precision mediump float;
precision mediump sampler3D;
#endif
uniform sampler2D stars;
uniform ivec2 stars_dim;
uniform sampler2D constellations;
uniform ivec2 constellations_dim;
uniform sampler2D galaxy;
uniform ivec2 galaxy_dim;
uniform sampler3D distance_cache_tex;
uniform ivec3 distance_cache_tex_dim;
uniform sampler2D distance_cache_z_bounds;
uniform ivec2 distance_cache_z_bounds_dim;
uniform sampler2D direction_cache;
uniform ivec2 direction_cache_dim;
uniform sampler2D direction_z_max_cache;
uniform ivec2 direction_z_max_cache_dim;
out vec4 outColor;

uniform float min_angle;

uniform ivec2 dimensions;
uniform vec2 disc_dim;
uniform float vertical_fov_degrees;
uniform vec3 normalized_dir;
uniform vec3 normalized_up;
uniform vec3 normalized_pos;
uniform mat3x3 observer_mat;
uniform mat3x3 inv_observer_mat;
uniform float distance;
uniform vec2 distance_bounds;
uniform float time_s;
uniform float vertical_fov_magnitude;

#define PI_2 1.5707963269
#define PI 3.1415926538
#define TAU 6.2831853076

//
/* Disc color calculations */
//

// this is a temporary marker to get the code to appear here.
// Disc code here!
