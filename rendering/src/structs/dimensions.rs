#[derive(Clone)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

impl Dimensions {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn to_index(&self, x: usize, y: usize) -> usize {
        self.width * (self.height - y - 1) + x
    }

    pub fn get_buffer(&self) -> Vec<u8> {
        vec![0; 4 * self.width * self.height]
    }
}
