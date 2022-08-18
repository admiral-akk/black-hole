use anyhow::*;
use half::f16;
use std::mem::size_of;
use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, BindingResource, SamplerBindingType, TextureFormat,
};

use super::{float_texture::Dimensions, variable::Variable};
use std::marker::PhantomData;
pub struct HalfFloatTexture<U: Dimensions> {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    dimension_type: PhantomData<U>,
}

impl<U: Dimensions> Variable for HalfFloatTexture<U> {
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

pub trait HalfFloatFormat<T> {
    fn format() -> TextureFormat;
    fn to_f16_vec(&self) -> Vec<f16>;
}

impl HalfFloatFormat<f32> for f32 {
    fn format() -> TextureFormat {
        TextureFormat::R16Float
    }
    fn to_f16_vec(&self) -> Vec<f16> {
        [f16::from_f32(*self)].to_vec()
    }
}

impl HalfFloatFormat<[f32; 2]> for [f32; 2] {
    fn format() -> TextureFormat {
        TextureFormat::Rg16Float
    }
    fn to_f16_vec(&self) -> Vec<f16> {
        [f16::from_f32(self[0]), f16::from_f32(self[1])].to_vec()
    }
}

impl<U: Dimensions + std::fmt::Debug> HalfFloatTexture<U> {
    pub fn from_f32<T>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vec: &[T],
        dimensions: U,
        label: &str,
    ) -> Result<Self>
    where
        T: HalfFloatFormat<T> + bytemuck::Pod,
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
                    dimensions.row_length() * size_of::<T>() as u32 / 2,
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
