pub struct Texture {
    pub arr: Vec<u8>,
    pub width: i32,
    pub name: String,
}

impl Texture {
    pub fn new(arr: &[u8], width: i32, name: &str) -> Texture {
        Texture {
            arr: arr.to_vec(),
            width,
            name: name.to_string(),
        }
    }
}
