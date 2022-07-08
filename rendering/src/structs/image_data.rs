use glam::Vec3;
use std::path::Path;

use super::data::Data;

pub struct ImageData {
    width: usize,
    pub samples: Vec<Data>,
    image: Vec<Vec3>,
    buf: Vec<u8>,
}
const SAMPLES_PER_DIMENSION: usize = 1;
const SAMPLES_PER_PIXEL: usize = SAMPLES_PER_DIMENSION * SAMPLES_PER_DIMENSION;
const PIXEL_AVERAGING: f32 = 255.0 * (SAMPLES_PER_PIXEL as f32);

fn color_correction(v: f32) -> f32 {
    v.sqrt()
}

fn set_samples_init(width: usize, half_sample_delta: f32, data: &mut Vec<Data>) {
    for x in 0..width {
        for y in 0..width {
            let index = to_index_init(x, y, width);
            let (base_x, base_y) = (x as f32 / width as f32, y as f32 / width as f32);
            for i_x in 0..SAMPLES_PER_DIMENSION {
                for i_y in 0..SAMPLES_PER_DIMENSION {
                    let (view_x, view_y) = (
                        base_x + half_sample_delta * ((2 * i_x + 1) as f32),
                        base_y + half_sample_delta * ((2 * i_y + 1) as f32),
                    );
                    data[(width * y + x) * SAMPLES_PER_PIXEL + i_x * SAMPLES_PER_DIMENSION + i_y] =
                        Data::Sample(index, view_x, view_y);
                }
            }
        }
    }

    data.sort_by(|d1, d2| {
        let i1 = match d1 {
            Data::Sample(i, _, _) => i,
            _ => {
                panic!("data isn't sample");
            }
        };
        let i2 = match d2 {
            Data::Sample(i, _, _) => i,
            _ => {
                panic!("data isn't sample");
            }
        };
        i1.cmp(i2)
    });
}

fn to_index_init(x: usize, y: usize, width: usize) -> usize {
    width * y + x
}

impl ImageData {
    pub fn new(width: usize, height: usize) -> Self {
        let image = vec![Vec3::ZERO; width * height];
        let buf = vec![255; 4 * width * height];
        let mut samples = vec![Data::None; SAMPLES_PER_PIXEL * width * height];
        let half_sample_delta = 0.5 / ((SAMPLES_PER_DIMENSION as f32) * (width as f32));
        set_samples_init(width, half_sample_delta, &mut samples);
        Self {
            width,
            samples,
            image,
            buf,
        }
    }

    pub fn load_colors(&mut self, data: &Vec<Data>) {
        for sample in data.iter() {
            match sample {
                Data::RGBA(index, c) => {
                    self.add_sample(*index, c);
                }
                _ => {}
            }
        }
    }

    pub fn add_sample(&mut self, index: usize, c: &[u8; 4]) {
        self.image[index].x += c[0] as f32;
        self.image[index].y += c[1] as f32;
        self.image[index].z += c[2] as f32;
    }

    pub fn get_image(&mut self) -> &[u8] {
        for i in 0..self.image.len() {
            let c = &self.image[i];
            let buffer_index = 4 * i;
            self.buf[buffer_index] = (255.0 * color_correction(c.x / PIXEL_AVERAGING)) as u8;
            self.buf[buffer_index + 1] = (255.0 * color_correction(c.y / PIXEL_AVERAGING)) as u8;
            self.buf[buffer_index + 2] = (255.0 * color_correction(c.z / PIXEL_AVERAGING)) as u8;
            self.image[i] = Vec3::ZERO;
        }

        &self.buf
    }

    pub fn get_dimensions(&self) -> (usize, usize) {
        (self.width, self.width)
    }

    pub fn get_sample_count(&self) -> usize {
        self.width * self.width * (SAMPLES_PER_DIMENSION * SAMPLES_PER_DIMENSION) as usize
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
