pub enum Exercise {
    Exercise1,
    Exercise2,
    Exercise3,
}

const VERT_1: &str = include_str!("../shaders/1/vertex.glsl");
const FRAG_1: &str = include_str!("../shaders/1/fragment.glsl");
const VERT_2: &str = include_str!("../shaders/2/vertex.glsl");
const FRAG_2: &str = include_str!("../shaders/2/fragment.glsl");
const VERT_3: &str = include_str!("../shaders/3/vertex.glsl");
const FRAG_3: &str = include_str!("../shaders/3/fragment.glsl");

pub fn get_shaders(exercise: &Exercise) -> (String, String) {
    match exercise {
        Exercise::Exercise1 => {
            return (VERT_1.to_string(), FRAG_1.to_string());
        }
        Exercise::Exercise2 => {
            return (VERT_2.to_string(), FRAG_2.to_string());
        }
        Exercise::Exercise3 => {
            return (VERT_3.to_string(), FRAG_3.to_string());
        }
    }
    return (VERT_1.to_string(), FRAG_1.to_string());
}
