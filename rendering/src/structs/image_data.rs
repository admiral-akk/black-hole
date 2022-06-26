use glam::IVec4;
use std::path::Path;

pub struct ImageData {
    width: usize,
    half_sample_delta: f64,
    image: Vec<IVec4>,
    buf: Vec<u8>,
}
const SAMPLES_PER_DIMENSION: u32 = 3;

impl ImageData {
    pub fn new(width: usize, height: usize) -> Self {
        let image = vec![IVec4::ZERO; width * height];
        let buf = vec![255; 4 * width * height];
        Self {
            width,
            half_sample_delta: 0.5 / ((SAMPLES_PER_DIMENSION as f64) * (width as f64)),
            image,
            buf,
        }
    }

    pub fn get_samples(&self, x: usize, y: usize) -> Vec<(f64, f64)> {
        let mut samples = Vec::new();

        let (base_x, base_y) = (x as f64 / self.width as f64, y as f64 / self.width as f64);
        for i_x in 0..SAMPLES_PER_DIMENSION {
            for i_y in 0..SAMPLES_PER_DIMENSION {
                let (view_x, view_y) = (
                    base_x + self.half_sample_delta * ((2 * i_x + 1) as f64),
                    base_y + self.half_sample_delta * ((2 * i_y + 1) as f64),
                );
                samples.push((view_x, view_y));
            }
        }
        samples
    }

    fn to_index(&self, x: usize, y: usize) -> usize {
        self.width * (self.width - y - 1) + x
    }

    pub fn add_sample(&mut self, x: usize, y: usize, c: &[u8; 4]) {
        let index = self.to_index(x, y);
        self.image[index].x += c[0] as i32;
        self.image[index].y += c[1] as i32;
        self.image[index].z += c[2] as i32;
        self.image[index].w += 1;
    }

    fn get_image(&mut self) -> &[u8] {
        for i in 0..self.image.len() {
            let c = &self.image[i];
            let buffer_index = 4 * i;
            self.buf[buffer_index] = (c.x / c.w) as u8;
            self.buf[buffer_index + 1] = (c.y / c.w) as u8;
            self.buf[buffer_index + 2] = (c.z / c.w) as u8;
            self.image[i] = IVec4::ZERO;
        }

        &self.buf
    }

    pub fn get_dimensions(&self) -> (usize, usize) {
        (self.width, self.width)
    }

    pub fn write_image(&mut self, file_name: &str) {
        let (width, height) = (self.width as u32, self.width as u32);
        image::save_buffer(
            &Path::new(&format!("output/{}.png", file_name)),
            self.get_image(),
            width,
            height,
            image::ColorType::Rgba8,
        )
        .unwrap();
    }
}
