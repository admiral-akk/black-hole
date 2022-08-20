use anyhow::*;
use half::f16;
use std::mem::size_of;
use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, BindingResource, SamplerBindingType, TextureFormat,
};

use super::{float_texture::Dimensions, variable::Variable};
use std::marker::PhantomData;
pub struct FullFloatTexture<U: Dimensions> {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    dimension_type: PhantomData<U>,
}

impl<U: Dimensions> Variable for FullFloatTexture<U> {
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

const SCALE_VAL: f32 = 2048.0;

fn to_f16(v: f32) -> [f16; 2] {
    let high = (v * SCALE_VAL).floor() / SCALE_VAL;
    let low = ((v - high) * SCALE_VAL * SCALE_VAL).floor() / SCALE_VAL;
    [f16::from_f32(high), f16::from_f32(low)]
}

fn to_f32(v: [f16; 2]) -> f32 {
    v[0].to_f32() + v[1].to_f32() / SCALE_VAL
}
pub trait FullFloatFormat<T> {
    fn format() -> TextureFormat;
    fn to_f16_vec(&self) -> Vec<f16>;
}

impl FullFloatFormat<f32> for f32 {
    fn format() -> TextureFormat {
        TextureFormat::Rg16Float
    }
    fn to_f16_vec(&self) -> Vec<f16> {
        to_f16(*self).to_vec()
    }
}

impl FullFloatFormat<[f32; 2]> for [f32; 2] {
    fn format() -> TextureFormat {
        TextureFormat::Rgba16Float
    }
    fn to_f16_vec(&self) -> Vec<f16> {
        let mut first = to_f16(self[0]).to_vec();
        first.extend_from_slice(&to_f16(self[1]));
        first
    }
}

impl<U: Dimensions + std::fmt::Debug> FullFloatTexture<U> {
    pub fn from_f32<T>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vec: &[T],
        dimensions: U,
        label: &str,
    ) -> Result<Self>
    where
        T: FullFloatFormat<T> + bytemuck::Pod,
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

        let vec = vec
            .iter()
            .map(|f| f.to_f16_vec())
            .fold(Vec::new(), |mut acc: Vec<f16>, b| {
                acc.extend_from_slice(&b);
                acc
            });

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
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
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
    use super::{to_f16, to_f32};

    const ITERATIONS: u32 = 100000000;
    #[test]
    fn max_error() {
        let mut max_error = 0.;
        let mut max_val = 0.;
        for i in 0..ITERATIONS {
            let f = (2. * i as f32 / (ITERATIONS - 1) as f32) - 1.;
            let f_1 = to_f32(to_f16(f));
            let err = (f - f_1).abs();
            if err > max_error {
                max_val = f;
                max_error = err;
            }
        }
        println!(
            "Max error: {}\nMax val: {}\n Approx val: {}",
            max_error,
            max_val,
            to_f32(to_f16(max_val))
        );
    }
}
