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
        self.height * self.width
    }

    pub fn to_xy(&self, index: usize) -> (usize, usize) {
        let _x = index % self.width;
        let _y = index / self.width;
        (index % self.width, self.height - 1 - index / self.width)
    }

    pub fn get_buffer(&self) -> Vec<u8> {
        vec![0; 4 * self.width * self.height]
    }

    pub fn aspect_ratio(&self) -> f64 {
        (self.width as f64) / (self.height as f64)
    }
}
