use anyhow::*;
use std::mem::size_of;
use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, BindingResource, SamplerBindingType, TextureFormat,
};

use super::{float_texture::Dimensions, variable::Variable};
use std::marker::PhantomData;
pub struct HackyFloatTexture<U: Dimensions> {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    dimension_type: PhantomData<U>,
}

impl<U: Dimensions> Variable for HackyFloatTexture<U> {
    fn entry(&self, index: u32) -> Vec<BindGroupEntry> {
        [
            wgpu::BindGroupEntry {
                binding: index,
                resource: BindingResource::TextureView(&self.view),
            },
            wgpu::BindGroupEntry {
                binding: index + 1,
                resource: BindingResource::Sampler(&self.sampler),
            },
        ]
        .to_vec()
    }

    fn layout_entry(&self, index: u32) -> Vec<BindGroupLayoutEntry> {
        [
            wgpu::BindGroupLayoutEntry {
                binding: index,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: U::texture_view_dimension(),
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: index + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ]
        .to_vec()
    }
}

fn float_to_u8(f: f32) -> [u8; 2] {
    let x = (256. * (f + 1.) / 2.).floor() as u8;
    let remainder = ((f + 1.) / 2.) - (x as f32) / 256.;
    let y = (256. * 255. * remainder).floor() as u8;
    [x, y]
}

fn u8_to_float(u: [u8; 2]) -> f32 {
    2. * (u[0] as f32 + u[1] as f32 / 255.) / 256. - 1.
}

pub trait FloatFormat<T> {
    fn format() -> TextureFormat;
    fn to_u8_vec(vec: &[T]) -> Vec<u8>;
}

impl FloatFormat<f32> for f32 {
    fn format() -> TextureFormat {
        TextureFormat::Rg8Unorm
    }
    fn to_u8_vec(vec: &[f32]) -> Vec<u8> {
        vec.iter()
            .map(|f| {
                let f = float_to_u8(*f);
                [f[0], f[1]]
            })
            .fold(Vec::new(), |mut a: Vec<u8>, b| {
                a.extend_from_slice(&b);
                a
            })
            .iter()
            .map(|f| *f)
            .collect()
    }
}

impl FloatFormat<[f32; 2]> for [f32; 2] {
    fn format() -> TextureFormat {
        TextureFormat::Rgba8Unorm
    }
    fn to_u8_vec(vec: &[[f32; 2]]) -> Vec<u8> {
        vec.iter()
            .map(|f| {
                let (f1, f2) = (float_to_u8(f[0]), float_to_u8(f[1]));
                [f1[0], f1[1], f2[0], f2[1]]
            })
            .fold(Vec::new(), |mut a: Vec<u8>, b| {
                a.extend_from_slice(&b);
                a
            })
            .iter()
            .map(|f| *f)
            .collect()
    }
}

impl<U: Dimensions> HackyFloatTexture<U> {
    pub fn from_f32<T>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vec: &[T],
        dimensions: U,
        label: &str,
    ) -> Result<Self>
    where
        T: FloatFormat<T> + bytemuck::Pod,
    {
        let size = dimensions.size();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: U::texture_dimension(),
            format: T::format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        let vec = T::to_u8_vec(vec);

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &bytemuck::cast_slice(&vec),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(
                    dimensions.row_length() * size_of::<T>() as u32,
                ),
                rows_per_image: std::num::NonZeroU32::new(dimensions.row_count()),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            dimension_type: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{float_to_u8, u8_to_float};

    const MAX_ERROR: f32 = 2. / (256. * 255.);
    const ITERATIONS: u32 = 10000;
    #[test]
    fn test_map_from_float_to_u8() {
        let f = 1.;
        let u = float_to_u8(f);
        let f_1 = u8_to_float(u);
        assert_eq!(f, f_1);
        let f = -1.;
        let u = float_to_u8(f);
        let f_1 = u8_to_float(u);
        assert_eq!(f, f_1);
        for i in 0..ITERATIONS {
            let f = 2. * (i as f32 / (ITERATIONS - 1) as f32) - 1.;
            let u = float_to_u8(f);
            let f_1 = u8_to_float(u);
            let error = (f - f_1).abs();
            assert!(error < MAX_ERROR, "Error: {}\nf: {}\nf': {}", error, f, f_1);
        }
    }
}
