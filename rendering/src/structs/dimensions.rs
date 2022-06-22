pub struct Dimensions {
    width: usize,
    height: usize,
}

impl Dimensions {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}
