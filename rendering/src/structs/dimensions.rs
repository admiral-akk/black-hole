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

    pub fn size(&self) -> usize {
        self.width * self.height
    }
}
