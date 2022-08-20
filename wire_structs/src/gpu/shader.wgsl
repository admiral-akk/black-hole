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

struct DataBuf {
   data:  array<f32>,
};

struct Field {
    magnitude: f32,
    radius:f32,
}

struct Particles {
    data: array<vec4<f32>>,
}

@group(0) @binding(0) 
var<storage, read_write> v_indices: DataBuf;
@group(0) @binding(1) 
var<storage, read> field: Field;
@group(0) @binding(2) 
var<storage, read_write> particles: Particles;

fn stop(particle: vec2<f32>) -> bool {
    let dist = length(particle.xy);
    // We add some error so that the geodesics that are on the edge of the schwarzchild radius don't get pulled in accidentally.
  return dist <  0.99* field.radius || dist > 35.;
}



    
fn step_size(particle: vec4<f32>) -> f32 {
    let v = length(particle.zw);
    let r = length(particle.xy);
    let m_4 = 6.0 * field.radius;
    if r < m_4{
    return 0.005;
    }
    if (dot(particle.xy,particle.zw) < 0.) {
        return 0.1 * r + 0.005;
    }
    return 0.1 * (r - m_4) + 0.005;
} 


   fn force( p: vec2<f32>) -> vec2<f32> {
        let diff = -p;
        let len = length(diff);
        var len_5 = len*len;
        len_5 = len_5*len_5*len;
      return normalize(diff) * field.magnitude  / len_5;
    }

fn rk4(particle: vec4<f32>, h: f32) -> vec4<f32> {
    let k_0 = h * particle.zw;
    let l_0 = h * force(particle.xy);

    let k_1 = h * (particle.zw + 0.5 * l_0);
    let l_1 = h * force(particle.xy + 0.5 * k_0);

    let k_2 = h * (particle.zw + 0.5 * l_1);
    let l_2 = h * force(particle.xy + 0.5 * k_1);

    let k_3 = h * (particle.zw + l_2);
    let l_3 = h * force(particle.xy + k_2);

    return 0.16666666 *vec4(
         (k_0 + 2.0 * k_1 + 2.0 * k_2 + k_3),
        (l_0 + 2.0 * l_1 + 2.0 * l_2 + l_3),
    );
}

fn closest(pos1: vec2<f32>,pos2: vec2<f32>) -> vec2<f32> {
    let step = pos2 - pos1;
    let d = dot(step, -pos1) / length(step);
    return d*step - pos1;
}
fn step_particle(particle: vec4<f32>) -> vec4<f32> {
    var h = step_size(particle);


    var delta_pv = rk4(particle, h);
    var delta_pv2 = rk4(particle, 0.5*h);
    delta_pv2 = delta_pv2+rk4(particle+delta_pv2, 0.5*h);
    loop {
        delta_pv = rk4(particle, h);
        delta_pv2 = rk4(particle, 0.5*h);
        delta_pv2 = delta_pv2+rk4(particle+delta_pv2, 0.5*h);
        continuing {
            h *= 0.125;
            break if length(delta_pv.xy) < 0.05 && length(delta_pv.xy - delta_pv2.xy) < 0.001 && length(delta_pv.zw - delta_pv2.zw) < 0.0001 ;
        }
    }
    
    // let intersection = closest(particle.xy, (particle+delta_pv).xy);
    // if (stop(intersection)) {
    //     return vec4(intersection, particle.zw);
    // }
    // let h = step_size(particle+delta_pv);
    // let delta_pv = rk4(particle, h);
    return particle + delta_pv;
}



@compute @workgroup_size(256,1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let iterations = u32(arrayLength(&particles.data)/(256u));
    let start_index = global_id.x;
    for (var i = 0u; i < iterations; i++) {
        let index = start_index + i*256u;
        var next_particle = particles.data[index];
        var start_pos = next_particle.xy;
        if (stop(start_pos) ) { continue; }
        var i = 0u;
        loop {
            next_particle = step_particle(next_particle);
            continuing {
                break if stop(next_particle.xy) || length(start_pos-next_particle.xy) > 0.1;
            }
        }
        particles.data[index] = next_particle;
    }
}