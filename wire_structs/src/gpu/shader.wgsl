// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Also licensed under MIT license, at your choice.


struct Particle {
    p: vec2<f32>,
    v: vec2<f32>,
    index: u32,
    black_hole_magnitude: f32,
    black_hole_radius: f32,
    max_distance: f32,
}

struct Particles {
    data: array<Particle>,
}

struct AngleLine {
    direction: vec2<f32>,
    filler: vec2<f32>,
}
struct AngleLines {
    lines: array<AngleLine>,
}

struct AnglePaths {
    dist: array<f32>,
}

@group(0) @binding(0) 
var<storage, read_write> out_path: AnglePaths;
@group(0) @binding(1) 
var<storage, read_write> particles: Particles;
@group(0) @binding(2) 
var<storage, read> lines: AngleLines;


fn crossed_line(particle: Particle, dir: vec2<f32>) -> bool {
 return particle.p.x * dir.y - particle.p.y * dir.x <= 0.;
}

fn intersection(start: Particle, end: Particle, dir: vec2<f32>) -> f32 {
    let start = start.p;
    let diff = end.p-start;
    return -(dir.x*start.y-dir.y*start.x) / (diff.x*dir.y - diff.y*dir.x);
}

fn stop(particle: Particle) -> bool {
    let dist = length(particle.p);
    // We add some error so that the geodesics that are on the edge of the schwarzchild radius don't get pulled in accidentally.
    return dist <= 0.9 * particle.black_hole_radius || dist > particle.max_distance;
}

fn step_size(particle: Particle) -> f32 {
    return 0.005;
} 


fn force(p: vec2<f32>, magnitude: f32) -> vec2<f32> {
    let diff = -p;
    let len = dot(diff,diff);
    let len_6 = len * len*len;
    return diff * magnitude / len_6;
}

fn rk4(particle: Particle, h: f32, magnitude: f32) -> Particle {
    let k_0 = h * particle.v;
    let l_0 = h * force(particle.p, particle.black_hole_magnitude);

    let k_1 = h * (particle.v + 0.5 * l_0);
    let l_1 = h * force(particle.p + 0.5 * k_0, particle.black_hole_magnitude);

    let k_2 = h * (particle.v + 0.5 * l_1);
    let l_2 = h * force(particle.p + 0.5 * k_1, particle.black_hole_magnitude);

    let k_3 = h * (particle.v + l_2);
    let l_3 = h * force(particle.p + k_2, particle.black_hole_magnitude);

    let pv = 0.16666666 * vec4(
        (k_0 + 2.0 * k_1 + 2.0 * k_2 + k_3),
        (l_0 + 2.0 * l_1 + 2.0 * l_2 + l_3),
    );

    var out = particle;
    out.p = out.p+pv.xy;
    out.v = out.v+pv.zw;

    return out;
}

fn passes_through(pos1: vec2<f32>, pos2: vec2<f32>, radius: f32) -> vec2<f32> {
    let diff = pos2-pos1;
    let step = normalize(diff);
    let dot_ps = dot(pos1, step);
    let rad_sq = radius*radius;
    let delta = dot_ps * dot_ps + rad_sq - dot(pos1,pos1);
    if delta < 0. {
        return pos2;
    }
    let d = -dot_ps - sqrt(delta);
    let t = d / length(diff);
    if (t >= 0.) {
        return pos1+ clamp(t,0.,1.) * diff;
    }
    return pos2;
}

fn step_particle(particle: Particle, magnitude: f32) -> Particle {
    var h = step_size(particle);
    var delta_pv = particle;
    loop {
        delta_pv = rk4(particle, h, magnitude);
        var delta_pv2 = rk4(particle, 0.5 * h, magnitude);
        delta_pv2 = rk4(delta_pv2, 0.5 * h, magnitude);
        let diff = vec4(delta_pv.p,delta_pv.v) - vec4(delta_pv2.p,delta_pv2.v);
        let delta_p = delta_pv.p-particle.p;
        continuing {
            h *= 0.5;
            // If it crosses the next line, then we've gone too far. We only
            // want to cross one line at a time.

            // If the half-step RK4 approximation differs too much from the full step,
            // we've accumulated too much error.

            // if we're close to the black hole, we want to step much slower.
            break if !crossed_line(delta_pv, lines.lines[particle.index + 1u].direction) && dot(diff,diff) < 0.000001 && dot(delta_p,delta_p) < 0.00000001; 
        }
    }

    var end = particle;
    end.p = passes_through(particle.p, delta_pv.p, particle.black_hole_radius);
    end.v = delta_pv.v;
    return end;
}

@compute @workgroup_size(256, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dim_y = arrayLength(&particles.data);
    let start_index = global_id.x;
    var iterations = dim_y / 256u;
    // If the particle count isn't divisible evenly between the workgroups, then
    // we allocate an additional particle to the lower workgroups.
    if (dim_y % 256u > global_id.x) {
        iterations++;
    }
    for (var i = 0u; i < iterations; i++) {
        let index = start_index + i * 256u;
        var particle = particles.data[index];
        if (stop(particle)) { continue; }
        if (particle.index == arrayLength(&lines.lines) - 1u) {
            continue;
        }
        var i = 0u;
        let l = lines.lines[particle.index].direction;
        loop {
            var next = step_particle(particle, particle.black_hole_magnitude);
            let t = intersection(particle, next, l);
            if (crossed_line(next,l)) {
                let pos = particle.p + (next.p-particle.p)*t;
                out_path.dist[index + particle.index*dim_y] = length(pos); 
                next.index = next.index + 1u;
            } 
            particle = next;
            continuing {
                break if true;
            }
        }
        particles.data[index] = particle;
    }
}